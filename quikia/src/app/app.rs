use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use winit::window::Window;
use std::thread::ThreadId;
use crate::item::{Item, ItemPath, PointerType};

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
    need_redraw: bool,
    layout_direction: LayoutDirection,
    pub(crate) focused_item_path: Option<ItemPath>,
    pub(crate) request_focus_path: Option<ItemPath>,

    pub(crate) pointer_catch:Option<(PointerType,ItemPath)>,

    pub(crate) named_ids:HashMap<String,usize>,
    pub(crate) unnamed_id:usize,
}

impl App {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            need_redraw: false,
            layout_direction: LayoutDirection::LeftToRight,
            focused_item_path: None,
            request_focus_path: None,
            pointer_catch: None,
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

    pub fn request_redraw(&mut self) {
        if !self.need_redraw {
            self.need_redraw = true;
            self.window.request_redraw();
        }
    }

    pub fn activate_ime(&mut self) {
        self.window.set_ime_allowed(true);
    }

    pub fn deactivate_ime(&mut self) {
        self.window.set_ime_allowed(false);
    }

    pub fn redraw_done(&mut self) {
        self.need_redraw = false;
    }

    pub fn request_focus(&mut self, path: &ItemPath){
        self.request_focus_path = Some(path.clone());
    }

    pub fn catch_pointer(&mut self, pointer_type: PointerType, path: &ItemPath){
        self.pointer_catch = Some((pointer_type, path.clone()));
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

    pub fn request_focus(&self, path: &ItemPath){
        self.app.lock().unwrap().request_focus(path);
    }

    pub fn catch_pointer(&self, pointer_type: PointerType, path: &ItemPath){
        self.app.lock().unwrap().catch_pointer(pointer_type, path);
    }

    pub fn request_redraw(&self) {
        self.app.lock().unwrap().request_redraw();
    }

    pub fn activate_ime(&self) {
        self.app.lock().unwrap().activate_ime();
    }

    pub fn deactivate_ime(&self) {
        self.app.lock().unwrap().deactivate_ime();
    }

    pub(crate) fn redraw_done(&self) {
        self.app.lock().unwrap().redraw_done();
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