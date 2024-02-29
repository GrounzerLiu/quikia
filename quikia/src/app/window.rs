// use winapi::shared::windef::HWND__;
use std::{ffi::CString, fs, num::NonZeroU32};
use std::time::Instant;
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
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use skia_safe::{gpu::{self, backend_render_targets, gl::FramebufferInfo, SurfaceOrigin}, Color, ColorType, Surface, Paint, Point, Font, Rect, FontStyle, FontMgr, Data, Picture, ISize, AlphaType, ColorFilter, BlendMode, ImageInfo, Color4f, ImageFilter, SamplingOptions, FilterMode, MipmapMode, Image, TileMode, MaskFilter, BlurStyle};
use skia_safe::canvas::SaveLayerRec;
use skia_safe::image_filters::{blur, CropRect};
use skia_safe::svg::{Canvas, Dom};
use skia_safe::wrapper::PointerWrapper;
// use winapi::shared::minwindef::TRUE;
use winit::dpi::{LogicalPosition, PhysicalPosition};
use winit::event::{ElementState, Ime, Touch, TouchPhase};
use winit::event_loop::{EventLoopBuilder, EventLoopWindowTarget};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;

use crate::anim::Animation;
use crate::app::{ANIMATIONS, new_app, Page, PageStack, SharedApp, Theme, ThemeColor, UserEvent};
use crate::item::{ButtonState, ImeAction, ItemPath, MeasureMode, PointerType};

struct Env {
    surface: Surface,
    gl_surface: GlutinSurface<WindowSurface>,
    gr_context: gpu::DirectContext,
    gl_context: PossiblyCurrentContext,
    fb_info: FramebufferInfo,
    num_samples: usize,
    stencil_size: usize,
}

fn init_env(elwt: &EventLoopWindowTarget<UserEvent>) -> (Env, Window) {
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(true);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(WindowBuilder::new().with_title("T")));
    let (window, gl_config) = display_builder
        .build(elwt, template, |configs| {
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

    (Env {
        surface,
        gl_surface,
        gl_context,
        gr_context,
        fb_info,
        num_samples,
        stencil_size,
    }, window)
}

fn run(app: SharedApp, event_loop: EventLoop<UserEvent>, launch_page: Box<dyn Page>) {
    let mut cursor_position = LogicalPosition::new(0.0_f32, 0.0_f32);
    let mut physical_cursor_position = PhysicalPosition::new(0.0_f32, 0.0_f32);

    let mut pointer_catch: Vec<(PointerType, usize)> = Vec::new();

    let mut pages = PageStack::new();
    pages.push(launch_page);

    let mut env = None;

    let mut info = String::new();

    let wallpaper = get_wallpaper();

    event_loop.run(move |event, elwt| {
        if let Event::Resumed = event {
            let (inited_env, window) = init_env(elwt);

            // match window.raw_window_handle() {
            //     RawWindowHandle::Win32(handle) => {
            //         //set_acrylic(handle.hwnd)
            //         /*let d=DWM_BLURBEHIND{
            //             dwFlags: DWM_BB_ENABLE,
            //             fEnable: TRUE,
            //             hRgnBlur: std::ptr::null_mut(),
            //             fTransitionOnMaximized: TRUE,
            //         };
            //         unsafe { DwmEnableBlurBehindWindow(handle.hwnd as *mut HWND__, std::ptr::addr_of!(d)); }
            //         winapi::um::winuser::SetWindowCom*/
            //     }
            //     _=>{}
            // }

            env = Some(inited_env);
            app.set_window(window);
            let current_page_item = pages.current_page().unwrap();
            let item = current_page_item.page.build(app.clone());
            current_page_item.page.on_create(app.clone());
            current_page_item.root_item = Some(item);
        }

        if env.is_none() {
            return;
        }

        info.clear();

        let event_clone = event.clone();

        match event {
            Event::UserEvent(user_event) => {
            }
            Event::WindowEvent { window_id, event } => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Moved(_) => {
                        app.request_redraw();
                    }
                    WindowEvent::Resized(physical_size) => {
                        let env = env.as_mut().unwrap();
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
                        pages.iter_mut().for_each(|page_item| {
                            page_item.root_item_mut().measure(MeasureMode::Specified(width), MeasureMode::Specified(height));
                            page_item.root_item_mut().layout(0.0, 0.0);
                        })
                    }
                    WindowEvent::CursorMoved { device_id, position, .. } => {
                        physical_cursor_position.x = position.x as f32;
                        physical_cursor_position.y = position.y as f32;
                        let scale_factor = app.scale_factor();
                        cursor_position = position.to_logical(scale_factor as f64);

                        pointer_catch.iter().for_each(|(pointer_type, id)| {
                            match pointer_type {
                                PointerType::Cursor { mouse_button } => {
                                    if let Some(item) = pages.current_page().unwrap().find_item_mut(*id) {
                                        item.mouse_input(
                                            device_id,
                                            ButtonState::Moved,
                                            *mouse_button,
                                            cursor_position.x,
                                            cursor_position.y,
                                        );
                                    }
                                }
                                _ => {}
                            }
                        });
                        pages.current_page().unwrap().root_item_mut().cursor_moved(cursor_position.x, cursor_position.y);
                    }
                    WindowEvent::MouseInput { device_id, state, button } => {
                        //println!("pointer_catch={:?}", pointer_catch);
                        match state {
                            ElementState::Pressed => {
                                pages.current_page().unwrap()
                                    .root_item_mut()
                                    .mouse_input(device_id, state.into(), button, cursor_position.x, cursor_position.y);
                            }
                            ElementState::Released => {
                                pointer_catch.iter().for_each(|(pointer_type, id)| {
                                    match pointer_type {
                                        PointerType::Cursor { mouse_button } => {
                                            if *mouse_button == button {
                                                if let Some(item) = pages.current_page().unwrap().find_item_mut(*id) {
                                                    item.mouse_input(
                                                        device_id,
                                                        ButtonState::Released,
                                                        *mouse_button,
                                                        cursor_position.x,
                                                        cursor_position.y,
                                                    );
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                });
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
                        let focused_item_id = app.focused_item_id;
                        drop(app);
                        if let Some(focused_item_id) = focused_item_id {
                            if let Some(item) = pages.current_page().unwrap().find_item_mut(focused_item_id) {
                                item.keyboard_input(device_id, event, is_synthetic);
                            }
                        }
                    }

                    WindowEvent::Ime(ime) => {
                        let app = app.lock().unwrap();
                        let focused_item_id = &app.focused_item_id;
                        if let Some(focused_item_path) = focused_item_id {
                            if let Some(item) = pages.current_page().unwrap().find_item_mut(*focused_item_path) {
                                drop(app);
                                item.ime_input(match ime {
                                    Ime::Enabled => ImeAction::Enabled,
                                    Ime::Preedit(text, cursor_position) => ImeAction::Preedit(text, cursor_position),
                                    Ime::Commit(text) => ImeAction::Commit(text),
                                    Ime::Disabled => ImeAction::Disabled,
                                });
                            }
                        }
                    }

                    WindowEvent::RedrawRequested => {
                        app.lock().unwrap().need_redraw = true;
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        let pc = app.lock().unwrap().pointer_catch.clone();
        if let Some(pc) = pc {
            pointer_catch.push(pc);
            app.lock().unwrap().pointer_catch = None;
        }

        {
            let mut app = app.lock().unwrap();
            if let Some(request_focus_id) = app.request_focus_id {
                if let Some(focused_item_id) = app.focused_item_id {
                    if let Some(focused_item) = pages.current_page().unwrap().find_item_mut(focused_item_id) {
                        focused_item.invoke_on_blur();
                    }
                }
                if let Some(request_focus_item) = pages.current_page().unwrap().find_item_mut(request_focus_id) {
                    request_focus_item.invoke_on_focus();
                    app.focused_item_id = Some(request_focus_id);
                    app.request_focus_id = None;
                }
            }
        }

        if app.lock().unwrap().need_rebuild {
            let mut page_item = pages.current_page().unwrap();
            let mut old_item = page_item.root_item_mut();
            let item = page_item.page.build(app.clone());
            page_item.page.on_create(app.clone());
            page_item.root_item = Some(item);
            app.rebuild_done();
            app.request_layout();
        }

        if app.lock().unwrap().need_layout {
            let (width, height): (f32, f32) = app.lock().unwrap().window().inner_size().into();
            let scale_factor = app.scale_factor();
            let width = width / scale_factor;
            let height = height / scale_factor;
            pages.iter_mut().for_each(|page_item| {
                page_item.root_item_mut().measure(MeasureMode::Specified(width), MeasureMode::Specified(height));
                page_item.root_item_mut().layout(0.0, 0.0);
            });
            app.re_layout_done();
            app.request_redraw();
        }


        if app.lock().unwrap().need_redraw {
            let env = env.as_mut().unwrap();
            let scale_factor = app.scale_factor();

            let canvas = env.surface.canvas();

            canvas.clear(app.lock().unwrap().theme().get_color(ThemeColor::Background));
            //canvas.clear(Color::from_argb(0x80,0,0,0));
            // if let Some(wallpaper) = &wallpaper {
            //     let app = app.lock().unwrap();
            //     let window = app.window();
            //     let window_client_position = window.inner_position();
            //     if let Ok(window_client_position) = window_client_position {
            //         let x = -window_client_position.x as f32 / scale_factor;
            //         let y = -window_client_position.y as f32 / scale_factor;
            //         let mut paint = Paint::default();
            //         canvas.draw_image_rect(
            //             wallpaper,
            //             None,
            //             Rect::from_xywh(x, y, wallpaper.width() as f32, wallpaper.height() as f32),
            //             &paint,
            //         );
            //     }
            // }


            canvas.save();
            canvas.scale((scale_factor, scale_factor));

            if let Some(page_item) = pages.current_page() {
                page_item.root_item_mut().draw(canvas);
            }

            canvas.restore();

            env.gr_context.flush_and_submit();
            env.gl_surface.swap_buffers(&env.gl_context).unwrap();
            app.redraw_done();
        }

        {
            let mut animations = ANIMATIONS.lock().unwrap();
            if !animations.is_empty() {
                let item = pages.current_page().unwrap().root_item_mut();
                let width = app.content_width();
                let height = app.content_height();
                for animation in animations.iter_mut() {
                    if animation.is_running() {
                        if animation.from.is_none() {
                            animation.from = Some(Animation::item_to_layout_params(item));
                            (animation.layout_transition.action)();
                            item.measure(MeasureMode::Specified(width), MeasureMode::Specified(height));
                            item.layout(0.0, 0.0);
                            app.lock().unwrap().need_layout = false;
                            animation.to = Some(Animation::item_to_layout_params(item));
                        }
                        animation.update(item, Instant::now());
                        app.request_redraw();
                        //elwt.set_control_flow(ControlFlow::Poll);
                    }
                }
                animations.retain(|animation| animation.is_running());
            } else {
                //elwt.set_control_flow(ControlFlow::Wait);
            }
        }
        println!("loop, {:?}", event_clone);
    }).unwrap();
}

#[cfg(not(target_os = "android"))]
pub fn create_window(window_builder: WindowBuilder, theme: Theme, launch_page: Box<dyn Page>) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let app = SharedApp::new(event_loop.create_proxy(), theme);
    new_app(app.clone());

    run(app, event_loop, launch_page);
}

#[cfg(target_os = "android")]
pub fn create_window(android_app: AndroidApp, window_builder: WindowBuilder, launch_page: Box<dyn Page>) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().with_android_app(android_app).build().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let app = SharedApp::new(event_loop.create_proxy());
    new_app(app.clone());

    run(app, event_loop, launch_page);
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

fn get_wallpaper() -> Option<Image> {
    /*if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Control Panel\Desktop")
    {
        let value: Result<String,_> = key.get_value("Wallpaper");
        if let Ok(value) = value {
            if let Ok(bytes) =fs::read(value){
                let data = Data::new_copy(&bytes);
                if let Some(image) = Image::from_encoded(data) {
                    return Some(image);
                }
            }
        }
        None
    } else {
        None
    }*/
    None
}