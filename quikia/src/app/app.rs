use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use winit::window::Window;
use std::thread::ThreadId;

lazy_static!(
    pub(crate) static ref APPS:Mutex<LinkedList<(ThreadId, SharedApp)>> = Mutex::new(LinkedList::new());
    );

pub(crate) fn current_app() -> Option<SharedApp>{
    let current_thread_id = std::thread::current().id();
    let apps = APPS.lock().unwrap();
    for app in apps.iter(){
        if app.0 == current_thread_id{
            return Some(app.1.clone());
        }
    }
    None
}

pub(crate) fn new_app(app: SharedApp){
    let mut apps = APPS.lock().unwrap();
    apps.push_back((std::thread::current().id(), app));
}

#[derive(Clone, Copy, Debug)]
pub enum LayoutDirection {
    LeftToRight,
    RightToLeft,
}

pub struct App {
    window: Window,
    layout_direction: LayoutDirection,
    need_redraw: bool,
    pub(crate) named_ids:HashMap<String,usize>,
    pub(crate) unnamed_id:usize,
}

impl App {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            layout_direction: LayoutDirection::LeftToRight,
            need_redraw: false,
            named_ids: HashMap::new(),
            unnamed_id: 0,
        }
    }

    pub fn new_id(&mut self) -> usize{
        self.unnamed_id += 1;
        self.unnamed_id
    }

    pub fn id(&mut self, name: &str) -> usize{
        if let Some(id) = self.named_ids.get(name){
            *id
        }else{
            let id = self.new_id();
            self.named_ids.insert(name.to_string(), id);
            id
        }
    }

    pub fn need_redraw(&mut self) {
        self.need_redraw = true;
    }

    pub fn whether_need_redraw(&self) -> bool {
        self.need_redraw
    }

    pub fn set_need_redraw(&mut self, need_redraw: bool) {
        self.need_redraw = need_redraw;
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn content_width(&self) -> f32 {
        self.window.inner_size().width as f32 / self.window.scale_factor() as f32
    }

    pub fn content_height(&self) -> f32 {
        self.window.inner_size().height as f32 / self.window.scale_factor() as f32
    }

    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    pub fn layout_direction(&self) -> LayoutDirection {
        self.layout_direction
    }
}

pub struct SharedApp {
    app: Arc<Mutex<App>>
}

impl SharedApp {
    pub fn new(window: Window) -> Self {
        Self {
            app: Arc::new(Mutex::new(App::new(window)))
        }
    }

    pub fn app(&self) -> Arc<Mutex<App>> {
        self.app.clone()
    }
}

impl Clone for SharedApp {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone()
        }
    }
}

impl Deref for SharedApp {
    type Target = Arc<Mutex<App>>;
    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl SharedApp {
    pub fn new_id(&self) -> usize{
        self.app.lock().unwrap().new_id()
    }

    pub fn id(&self, name: &str) -> usize{
        self.app.lock().unwrap().id(name)
    }

    pub fn need_redraw(&self) {
        self.app.lock().unwrap().need_redraw();
    }

    pub fn whether_need_redraw(&self) -> bool {
        self.app.lock().unwrap().whether_need_redraw()
    }

    pub fn set_need_redraw(&self, need_redraw: bool) {
        self.app.lock().unwrap().set_need_redraw(need_redraw);
    }

    pub fn content_width(&self) -> f32 {
        self.app.lock().unwrap().content_width()
    }

    pub fn content_height(&self) -> f32 {
        self.app.lock().unwrap().content_height()
    }

    pub fn scale_factor(&self) -> f32 {
        self.app.lock().unwrap().scale_factor()
    }

    pub fn layout_direction(&self) -> LayoutDirection {
        self.app.lock().unwrap().layout_direction()
    }
}