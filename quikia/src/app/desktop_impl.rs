use std::{
    ffi::CString,
    num::NonZeroU32,
};
use std::collections::LinkedList;
use gl::types::*;
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder,
        PossiblyCurrentContext,
    },
    display::{GetGlDisplay, GlDisplay},
    prelude::GlSurface,
    surface::{Surface as GlutinSurface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin::context::NotCurrentGlContext;
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use skia_safe::{gpu::{self, backend_render_targets, gl::FramebufferInfo, SurfaceOrigin}, Color, ColorType, Surface, Paint, Point};
use winit::dpi::{LogicalPosition, PhysicalPosition};
use winit::event::{ElementState, Ime, Touch};
use winit::event_loop::{EventLoopBuilder, EventLoopProxy};
use winit::keyboard::Key::Named;
use winit::keyboard::{Key, NamedKey};
use crate::app::{new_app, Page, PageItem, PageStack, SharedApp, UserEvent};
use crate::item::{ButtonState, ImeAction, Item, ItemPath, KeyboardInput, MeasureMode, PointerAction, PointerType};
use crate::property::Gettable;

struct Env {
    surface: Surface,
    gl_surface: GlutinSurface<WindowSurface>,
    gr_context: gpu::DirectContext,
    gl_context: PossiblyCurrentContext,
    fb_info: FramebufferInfo,
    num_samples: usize,
    stencil_size: usize,
}

fn run(app: SharedApp, mut pages: PageStack, mut env: Env, event_loop: EventLoop<UserEvent>) {
    let mut cursor_positon = LogicalPosition::new(0.0_f32, 0.0_f32);
    let mut physical_cursor_positon = PhysicalPosition::new(0.0_f32, 0.0_f32);

    let mut pointer_catch: Vec<(PointerType, ItemPath)> = Vec::new();

    event_loop.run(move |event, elwt| {
        match event {
            Event::UserEvent(user_event) => {
                match user_event {
                    UserEvent::TimerExpired(item_path, msg) => {
                        let mut page_item = pages.current_page().unwrap();
                        let mut item = page_item.find_item_mut(&item_path).unwrap().as_event_input();
                        item.on_timer_expired(msg);
                    }
                }
            }
            Event::WindowEvent { window_id, event } => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        env.surface = create_surface(
                            app.lock().unwrap().window(),
                            env.fb_info,
                            &mut env.gr_context,
                            env.num_samples,
                            env.stencil_size,
                        );
                        /* First resize the opengl drawable */
                        let (width, height): (u32, u32) = physical_size.into();

                        env.gl_surface.resize(
                            &env.gl_context,
                            NonZeroU32::new(width.max(1)).unwrap(),
                            NonZeroU32::new(height.max(1)).unwrap(),
                        );


                        let width = width as f32 / app.scale_factor();
                        let height = height as f32 / app.scale_factor();
                        pages.iter_mut().for_each(|PageItem { root_item, .. }| {
                            root_item.measure(0.0, 0.0, MeasureMode::Exactly(width), MeasureMode::Exactly(height));
                        })
                    }
                    WindowEvent::CursorMoved { device_id, position, .. } => {
                        physical_cursor_positon.x = position.x as f32;
                        physical_cursor_positon.y = position.y as f32;
                        let scale_factor = app.scale_factor();
                        cursor_positon = position.to_logical(scale_factor as f64);

                        pointer_catch.iter().for_each(|(pointer_type, path)| {
                            match pointer_type {
                                PointerType::Cursor { mouse_button } => {
                                    if let Some(item) = pages.current_page().unwrap().find_item_mut(path) {
                                        item.as_event_input().on_mouse_input(
                                            device_id,
                                            ButtonState::Moved,
                                            *mouse_button,
                                            cursor_positon.x,
                                            cursor_positon.y,
                                        );
                                    }
                                }
                                _ => {}
                            }
                        });
                    }
                    WindowEvent::MouseInput { device_id, state, button } => {
                        match state {
                            ElementState::Pressed => {
                                let PageItem { root_item, .. } = pages.current_page().unwrap();
                                root_item.as_event_input().on_mouse_input(device_id, state.into(), button, cursor_positon.x, cursor_positon.y);
                            }
                            ElementState::Released => {
                                pointer_catch.iter().for_each(|(pointer_type, path)| {
                                    match pointer_type {
                                        PointerType::Cursor { mouse_button } => {
                                            if *mouse_button == button {
                                                if let Some(item) = pages.current_page().unwrap().find_item_mut(path) {
                                                    item.as_event_input().on_mouse_input(
                                                        device_id,
                                                        ButtonState::Released,
                                                        *mouse_button,
                                                        cursor_positon.x,
                                                        cursor_positon.y,
                                                    );
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                });
                                //pointer_catch.clear();
                                pointer_catch.retain(|(pointer_type, _)| {
                                    match pointer_type {
                                        PointerType::Cursor { mouse_button } => {
                                            *mouse_button != button
                                        }
                                        _ => true
                                    }
                                });
                            }
                        }
                    }

                    WindowEvent::KeyboardInput {
                        device_id, event, is_synthetic
                    } => {
                        let app = app.lock().unwrap();
                        let focused_item_path = app.focused_item_path.clone();
                        drop(app);
                        if let Some(focused_item_path) = focused_item_path {
                            if let Some(item) = pages.current_page().unwrap().find_item_mut(&focused_item_path) {
                                let mut item = item.as_event_input();
                                item.on_keyboard_input(KeyboardInput {
                                    device_id,
                                    event: event.clone(),
                                    is_synthetic,
                                });
                            }
                        }

                        /*                        if let Some(page_item) = pages.current_page() {
                                                    let mut item_stack = LinkedList::new();
                                                    item_stack.push_back(page_item.root_id);
                                                    while let Some(id) = item_stack.pop_front() {
                                                        let item = page_item.items.get_mut(&id).unwrap();
                                                        if item.get_enabled().get() {
                                                            let consumed = item.as_event_input().on_keyboard_input(KeyboardInput {
                                                                device_id,
                                                                event: event.clone(),
                                                                is_synthetic,
                                                            });

                                                            if consumed {
                                                                break;
                                                            }

                                                            // if let Some(on_key_input) = item.get_on_key_input() {
                                                            //     on_key_input(physical_key, logical_key, text, location, state, repeat, is_synthetic);
                                                            // }
                                                            if let Some(item_group) = item.as_item_group_mut() {
                                                                for child in item_group.get_children_mut() {
                                                                    item_stack.push_back(child);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                if let ElementState::Pressed = event.state {
                                                    match event.logical_key {
                                                        Named(named_key) => {
                                                            match named_key {
                                                                NamedKey::Enter => {
                                                                    let focused_item_id = { app.lock().unwrap().focused_item_id };
                                                                    if let Some(focused_item_id) = focused_item_id {
                                                                        if let Some(item) = pages.current_page().unwrap().items.get_mut(&focused_item_id) {
                                                                            if let Some(ime_inputable) = item.as_ime_inputable() {
                                                                                ime_inputable.input(ImeAction::Enter);
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                NamedKey::Backspace => {
                                                                    let focused_item_id = { app.lock().unwrap().focused_item_id };
                                                                    if let Some(focused_item_id) = focused_item_id {
                                                                        if let Some(item) = pages.current_page().unwrap().items.get_mut(&focused_item_id) {
                                                                            if let Some(ime_inputable) = item.as_ime_inputable() {
                                                                                ime_inputable.input(ImeAction::Delete);
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                NamedKey::Space => {
                                                                    let focused_item_id = { app.lock().unwrap().focused_item_id };
                                                                    if let Some(focused_item_id) = focused_item_id {
                                                                        if let Some(item) = pages.current_page().unwrap().items.get_mut(&focused_item_id) {
                                                                            if let Some(ime_inputable) = item.as_ime_inputable() {
                                                                                ime_inputable.input(ImeAction::Commit(" ".to_string()));
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        Key::Character(char) => {
                                                            let focused_item_id = { app.lock().unwrap().focused_item_id };
                                                            if let Some(focused_item_id) = focused_item_id {
                                                                if let Some(item) = pages.current_page().unwrap().items.get_mut(&focused_item_id) {
                                                                    if let Some(ime_inputable) = item.as_ime_inputable() {
                                                                        ime_inputable.input(ImeAction::Commit(char.to_string()));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Key::Unidentified(_) => {}
                                                        Key::Dead(_) => {}
                                                    }
                                                }*/
                    }

                    WindowEvent::Ime(ime) => {
                        let app = app.lock().unwrap();
                        let focused_item_path = &app.focused_item_path;
                        if let Some(focused_item_path) = focused_item_path {
                            if let Some(item) = pages.current_page().unwrap().find_item_mut(focused_item_path) {
                                if let Some(ime_inputable) = item.as_ime_inputable() {
                                    drop(app);
                                    ime_inputable.input(match ime {
                                        Ime::Enabled => ImeAction::Enabled,
                                        Ime::Preedit(text, cursor_position) => ImeAction::Preedit(text, cursor_position),
                                        Ime::Commit(text) => ImeAction::Commit(text),
                                        Ime::Disabled => ImeAction::Disabled,
                                    });
                                }
                            }
                        }
                    }

                    WindowEvent::Touch(Touch {
                                           device_id, phase, location, force, id
                                       }) => {
                        println!("Touch {:?}", phase);
                    }

                    WindowEvent::RedrawRequested => {
                        let (width, height) = (app.content_width(), app.content_height());
                        pages.iter_mut().for_each(|PageItem { root_item, .. }| {
                            root_item.measure(0.0, 0.0, MeasureMode::Exactly(width), MeasureMode::Exactly(height));
                        });

                        let canvas = env.surface.canvas();
                        canvas.save();
                        let scale_factor = app.scale_factor();
                        canvas.scale((scale_factor, scale_factor));
                        canvas.clear(Color::WHITE);

                        if let Some(PageItem { root_item, .. }) = pages.current_page() {
                            root_item.draw(canvas);
                        }

                        canvas.restore();

                        env.gr_context.flush_and_submit();
                        env.gl_surface.swap_buffers(&env.gl_context).unwrap();
                        app.redraw_done();
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        {
            let mut app = app.lock().unwrap();
            if let Some(request_focus_path) = &app.request_focus_path {
                if let Some(focused_item_path) = &app.focused_item_path {
                    let focused_item = pages.current_page().unwrap().find_item_mut(focused_item_path).unwrap();
                    if let Some(on_blur) = focused_item.get_on_blur() {
                        on_blur();
                    }
                }

                let item = pages.current_page().unwrap().find_item_mut(request_focus_path).unwrap();
                if let Some(on_focus) = item.get_on_focus() {
                    on_focus();
                }
                app.focused_item_path = Some(request_focus_path.clone());
                app.request_focus_path = None;
            }

            if let Some(pc) = &app.pointer_catch {
                pointer_catch.push(pc.clone());
                app.pointer_catch = None;
            }
        }

        elwt.set_control_flow(ControlFlow::Wait);
    }).unwrap();
}

pub fn create_window(window_builder: WindowBuilder, launch_page: Box<dyn Page>) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build().unwrap();

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(true);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() < accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();
    let window = window.expect("Could not create window with OpenGL context");
    let raw_window_handle = window.raw_window_handle();

    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));
    let not_current_gl_context = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_config
                    .display()
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    };

    let (width, height): (u32, u32) = window.inner_size().into();

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .expect("Could not create gl window surface")
    };

    let gl_context = not_current_gl_context
        .make_current(&gl_surface)
        .expect("Could not make GL context current when setting up skia renderer");

    gl::load_with(|s| {
        gl_config
            .display()
            .get_proc_address(CString::new(s).unwrap().as_c_str())
    });
    let interface = gpu::gl::Interface::new_load_with(|name| {
        if name == "eglGetCurrentDisplay" {
            return std::ptr::null();
        }
        gl_config
            .display()
            .get_proc_address(CString::new(name).unwrap().as_c_str())
    })
        .expect("Could not create interface");

    let mut gr_context = gpu::DirectContext::new_gl(Some(interface), None)
        .expect("Could not create direct context");

    let fb_info = {
        let mut fboid: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

        FramebufferInfo {
            fboid: fboid.try_into().unwrap(),
            format: gpu::gl::Format::RGBA8.into(),
            ..Default::default()
        }
    };


    let num_samples = gl_config.num_samples() as usize;
    let stencil_size = gl_config.stencil_size() as usize;

    let surface = create_surface(&window, fb_info, &mut gr_context, num_samples, stencil_size);

    let env = Env {
        surface,
        gl_surface,
        gl_context,
        gr_context,
        fb_info,
        num_samples,
        stencil_size,
    };

    event_loop.set_control_flow(ControlFlow::Wait);

    let app = SharedApp::new(window, event_loop.create_proxy());
    new_app(app.clone());
    let mut pages = PageStack::new();
    pages.launch(launch_page, app.clone());
    run(app, pages, env, event_loop);
}

fn create_surface(
    window: &Window,
    fb_info: FramebufferInfo,
    gr_context: &mut gpu::DirectContext,
    num_samples: usize,
    stencil_size: usize,
) -> Surface {
    let size = window.inner_size();
    let size = (
        size.width.try_into().expect("Could not convert width"),
        size.height.try_into().expect("Could not convert height"),
    );
    let backend_render_target =
        backend_render_targets::make_gl(size, num_samples, stencil_size, fb_info);

    gpu::surfaces::wrap_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
        .expect("Could not create skia surface")
}