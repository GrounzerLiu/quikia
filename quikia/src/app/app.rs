use std::ops::Deref;
use std::sync::{Arc, Mutex};
use winit::window::Window;

pub struct App{
    window: Window,
    need_redraw: bool,
}

impl App{
    pub fn new(window: Window) -> Self{
        Self{
            window,
            need_redraw: false,
        }
    }

    pub fn need_redraw(&mut self){
        self.need_redraw = true;
    }

    pub fn set_need_redraw(&mut self, need_redraw: bool){
        self.need_redraw = need_redraw;
    }

    pub fn window(&self) -> &Window{
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window{
        &mut self.window
    }

    pub fn content_width(&self) -> f32{
        self.window.inner_size().width as f32
    }

    pub fn content_height(&self) -> f32{
        self.window.inner_size().height as f32
    }

    pub fn scale_factor(&self) -> f32{
        self.window.scale_factor() as f32
    }
}

pub struct SharedApp{
    app: Arc<Mutex<App>>,
}

impl SharedApp{
    pub fn new(window: Window) -> Self{
        Self{
            app: Arc::new(Mutex::new(App::new(window))),
        }
    }

    pub fn app(&self) -> Arc<Mutex<App>>{
        self.app.clone()
    }
}

impl Clone for SharedApp{
    fn clone(&self) -> Self{
        Self{
            app: self.app.clone(),
        }
    }
}

impl Deref for SharedApp{
    type Target = Arc<Mutex<App>>;
    fn deref(&self) -> &Self::Target{
        &self.app
    }
}

impl SharedApp{
    pub fn need_redraw(&self){
        self.app.lock().unwrap().need_redraw();
    }

    pub fn set_need_redraw(&self, need_redraw: bool){
        self.app.lock().unwrap().set_need_redraw(need_redraw);
    }

    pub fn content_width(&self) -> f32{
        self.app.lock().unwrap().content_width()
    }

    pub fn content_height(&self) -> f32{
        self.app.lock().unwrap().content_height()
    }

    pub fn scale_factor(&self) -> f32{
        self.app.lock().unwrap().scale_factor()
    }
}