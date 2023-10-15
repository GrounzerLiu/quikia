
use std::{
    ffi::CString,
    num::NonZeroU32,
};

use gl::types::*;
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor,
        PossiblyCurrentContext,
    },
    display::{GetGlDisplay, GlDisplay},
    prelude::GlSurface,
    surface::{Surface as GlutinSurface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use skia_safe::{gpu::{self, backend_render_targets, gl::FramebufferInfo, SurfaceOrigin}, Color, ColorType, Surface, Paint};
use crate::app::SharedApp;

struct Env {
    surface: Surface,
    gl_surface: GlutinSurface<WindowSurface>,
    gr_context: gpu::DirectContext,
    gl_context: PossiblyCurrentContext,
    fb_info: FramebufferInfo,
    num_samples: usize,
    stencil_size: usize,
}

fn run(app:SharedApp, mut env:Env, event_loop:EventLoop<()>){
    event_loop.run(move |event, _, control_flow| {
        let mut needs_redraw = false;
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
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
                }
                WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        virtual_keycode,
                        modifiers,
                        ..
                    },
                    ..
                } => {
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                needs_redraw = true;
            }
            _ => (),
        }

        needs_redraw=needs_redraw||app.whether_need_redraw();

        if needs_redraw{
            let canvas = env.surface.canvas();
            canvas.clear(Color::RED);

            canvas.draw_circle((100.0, 100.0), 50.0, &Paint::default().set_color(Color::BLUE));

            env.gr_context.flush_and_submit();
            env.gl_surface.swap_buffers(&env.gl_context).unwrap();
        }

        *control_flow = if needs_redraw {
            ControlFlow::Poll
        } else {
            ControlFlow::Wait
        };
    });
}

pub fn create_window(window_builder: WindowBuilder) {


    let event_loop = EventLoop::new();

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
    let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
        if name == "eglGetCurrentDisplay" {
            return std::ptr::null();
        }
        gl_config
            .display()
            .get_proc_address(CString::new(name).unwrap().as_c_str())
    })
        .expect("Could not create interface");

    let mut gr_context = skia_safe::gpu::DirectContext::new_gl(Some(interface), None)
        .expect("Could not create direct context");

    let fb_info = {
        let mut fboid: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

        FramebufferInfo {
            fboid: fboid.try_into().unwrap(),
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
            ..Default::default()
        }
    };


    let num_samples = gl_config.num_samples() as usize;
    let stencil_size = gl_config.stencil_size() as usize;

    let surface = create_surface(&window, fb_info, &mut gr_context, num_samples, stencil_size);



    let mut env = Env {
        surface,
        gl_surface,
        gl_context,
        gr_context,
        fb_info,
        num_samples,
        stencil_size,
    };

    let app = SharedApp::new(window);
    run(app, env, event_loop);
}

fn create_surface(
    window: &Window,
    fb_info: FramebufferInfo,
    gr_context: &mut skia_safe::gpu::DirectContext,
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