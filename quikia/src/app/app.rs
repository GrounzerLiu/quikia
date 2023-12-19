use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use winit::window::Window;
use std::thread::ThreadId;
use winit::event_loop::EventLoopProxy;
use crate::anim::Animation;
use crate::app::Theme;
use crate::item::{ItemPath, LayoutDirection, PointerType};

lazy_static!(
    pub(crate) static ref ANIMATIONS:Mutex<Vec<Animation>> = Mutex::new(Vec::new());
);

lazy_static!(
    pub(crate) static ref APPS:Mutex<LinkedList<(ThreadId, SharedApp)>> = Mutex::new(LinkedList::new());
);

pub(crate) fn current_app() -> Option<SharedApp> {
    let current_thread_id = std::thread::current().id();
    let apps = APPS.lock().unwrap();
    for app in apps.iter() {
        if app.0 == current_thread_id {
            return Some(app.1.clone());
        }
    }
    None
}

pub(crate) fn new_app(app: SharedApp) {
    let mut apps = APPS.lock().unwrap();
    apps.push_back((std::thread::current().id(), app));
}

#[derive(Clone, Debug)]
pub(crate) enum UserEvent {
    Empty,
    TimerExpired(ItemPath,String)
}

pub struct App {
    window: Option<Window>,
    theme: Theme,
    pub(crate) need_redraw: bool,
    pub(crate) need_layout: bool,
    pub(crate) need_rebuild: bool,
    event_loop_proxy: EventLoopProxy<UserEvent>,
    layout_direction: LayoutDirection,
    pub(crate) focused_item_id: Option<usize>,
    pub(crate) request_focus_id: Option<usize>,

    pub(crate) pointer_catch: Option<(PointerType, usize)>,

    pub(crate) named_ids: HashMap<String, usize>,
    pub(crate) unnamed_id: usize,
}

impl App {
    pub(crate) fn new(event_loop_proxy: EventLoopProxy<UserEvent>, theme: Theme) -> Self {
        Self {
            window: None,
            theme,
            need_redraw: false,
            need_layout: false,
            need_rebuild: false,
            event_loop_proxy,
            layout_direction: LayoutDirection::LeftToRight,
            focused_item_id: None,
            request_focus_id: None,
            pointer_catch: None,
            named_ids: HashMap::new(),
            unnamed_id: 0,
        }
    }

    pub fn new_id(&mut self) -> usize {
        self.unnamed_id += 1;
        self.unnamed_id
    }

    pub fn id(&mut self, name: &str) -> usize {
        if let Some(id) = self.named_ids.get(name) {
            *id
        } else {
            let id = self.new_id();
            self.named_ids.insert(name.to_string(), id);
            id
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub(crate) fn set_window(&mut self, window: Window) {
        self.window = Some(window);
    }

    pub(crate) fn send_event(&self, event: UserEvent) {
        self.event_loop_proxy.send_event(event).unwrap();
    }

    pub fn request_redraw(&mut self) {
        if !self.need_layout {
            self.window.as_mut().unwrap().request_redraw();
        }
        self.need_redraw = true;
    }

    pub fn request_layout(&mut self) {
        self.need_layout = true;
        if !self.need_redraw {
            self.request_redraw();
        }
    }

    pub fn request_rebuild(&mut self) {
        self.need_rebuild = true;
    }

    pub fn activate_ime(&mut self){
        self.window().set_ime_allowed(true);
    }

    pub fn deactivate_ime(&mut self){
        self.window().set_ime_allowed(false);
    }

    pub(crate) fn redraw_done(&mut self) {
        self.need_redraw = false;
    }

    pub(crate) fn layout_done(&mut self) {
        self.need_layout = false;
    }

    pub(crate) fn rebuild_done(&mut self) {
        self.need_rebuild = false;
    }

    pub fn request_focus(&mut self, id: usize) {
        self.request_focus_id = Some(id);
    }

    pub fn catch_pointer(&mut self, pointer_type: PointerType, id:usize) {
        self.pointer_catch = Some((pointer_type, id));
    }

    pub fn window(&self) -> &Window {
        self.window.as_ref().unwrap()
    }

    pub fn window_mut(&mut self) -> &mut Window {
        self.window.as_mut().unwrap()
    }

    pub fn content_width(&self) -> f32 {
        self.window().inner_size().width as f32 / self.window().scale_factor() as f32
    }

    pub fn content_height(&self) -> f32 {
        self.window().inner_size().height as f32 / self.window().scale_factor() as f32
    }

    pub fn scale_factor(&self) -> f32 {
        self.window().scale_factor() as f32
    }

    pub fn layout_direction(&self) -> LayoutDirection {
        self.layout_direction
    }
    
    pub fn set_layout_direction(&mut self, layout_direction: LayoutDirection) {
        self.layout_direction = layout_direction;
    }
}

pub struct SharedApp {
    app: Arc<Mutex<App>>,
}

impl SharedApp {
    pub(crate) fn new(event_loop_proxy: EventLoopProxy<UserEvent>, theme: Theme) -> Self {
        Self {
            app: Arc::new(Mutex::new(App::new(event_loop_proxy,theme)))
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
    pub fn new_id(&self) -> usize {
        self.app.lock().unwrap().new_id()
    }

    pub fn id(&self, name: &str) -> usize {
        self.app.lock().unwrap().id(name)
    }

    pub fn set_theme(&self, theme: Theme) {
        self.app.lock().unwrap().set_theme(theme);
    }

    pub(crate) fn set_window(&self, window: Window) {
        self.app.lock().unwrap().set_window(window);
    }

    pub(crate) fn send_event(&self, event: UserEvent) {
        self.app.lock().unwrap().send_event(event);
    }

    pub fn request_focus(&self, id: usize) {
        self.app.lock().unwrap().request_focus(id);
    }

    pub fn catch_pointer(&self, pointer_type: PointerType, id: usize) {
        self.app.lock().unwrap().catch_pointer(pointer_type, id);
    }

    pub fn request_redraw(&self) {
        self.app.lock().unwrap().request_redraw();
    }

    pub fn request_layout(&self) {
        self.app.lock().unwrap().request_layout();
    }

    pub fn request_rebuild(&self) {
        self.app.lock().unwrap().request_rebuild();
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

    pub(crate) fn re_layout_done(&self) {
        self.app.lock().unwrap().layout_done();
    }

    pub(crate) fn rebuild_done(&self) {
        self.app.lock().unwrap().rebuild_done();
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
    
    pub fn set_layout_direction(&self, layout_direction: LayoutDirection) {
        self.app.lock().unwrap().set_layout_direction(layout_direction);
    }
}