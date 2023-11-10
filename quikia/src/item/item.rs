use std::collections::{HashMap, LinkedList};
use std::ops::{Deref, DerefMut};
use std::slice::{Iter, IterMut};
use std::sync::Mutex;
use std::time::Duration;
use skia_safe::Canvas;
use skia_safe::gpu::MipMapped::No;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton};
use crate::app::ItemMap;
use crate::item::{Rectangle, Row, /*Stack,*/ TextBlock};
use crate::property::{BoolProperty, FloatProperty, Gettable, Size, SizeProperty};

#[derive(Clone, Debug)]
pub struct ItemPath {
    path: LinkedList<usize>,
}

impl ItemPath {
    pub fn new() -> Self {
        Self {
            path: LinkedList::new()
        }
    }

    pub fn push(&mut self, id: usize) {
        self.path.push_back(id);
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.path.pop_back()
    }

    pub fn iter(&self) -> std::collections::linked_list::Iter<usize> {
        self.path.iter()
    }
}

pub enum Item {
    Rectangle(Rectangle),
    Row(Row),
    //Stack(Stack),
    TextBlock(TextBlock),
}

impl Item {
    pub fn as_ime_inputable(&mut self) -> Option<&mut dyn Inputable> {
        match self {
            Item::TextBlock(text_block) => Some(text_block),
            _ => None,
        }
    }

    pub fn as_event_input(&mut self) -> Box<&mut dyn EventInput> {
        match self {
            Item::Rectangle(rectangle) => Box::new(rectangle),
            Item::Row(row) => Box::new(row),
            //Item::Stack(stack) => Box::new(stack),
            Item::TextBlock(text_block) => Box::new(text_block),
        }
    }
}

impl Deref for Item {
    type Target = dyn ItemTrait;
    fn deref(&self) -> &Self::Target {
        match self {
            Item::Rectangle(rectangle) => rectangle,
            Item::Row(row) => row,
            //Item::Stack(stack) => stack,
            Item::TextBlock(text_block) => text_block,
        }
    }
}

impl DerefMut for Item {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Item::Rectangle(rectangle) => rectangle,
            Item::Row(row) => row,
            //Item::Stack(stack) => stack,
            Item::TextBlock(text_block) => text_block,
        }
    }
}

impl Into<Item> for Rectangle {
    fn into(self) -> Item {
        Item::Rectangle(self)
    }
}


#[derive(Clone, Copy, Debug)]
pub enum MeasureMode {
    Exactly(f32),
    AtMost(f32),
}

pub enum ImeAction {
    Enabled,
    Enter,
    Delete,
    Preedit(String, Option<(usize, usize)>),
    Commit(String),
    Disabled,
}

#[derive(Clone, Copy, Debug)]
pub enum PointerType {
    Cursor { mouse_button: MouseButton },
    Touch { id: usize },
}

#[derive(Clone, Copy, Debug)]
pub enum PointerAction {
    Down { x: f32, y: f32, pointer_type: PointerType },
    Up { x: f32, y: f32, pointer_type: PointerType },
    Move { x: f32, y: f32, pointer_type: PointerType },
    Cancel,
}

impl PointerAction {
    pub fn from_mouse(state: ButtonState, button: MouseButton, x: f32, y: f32) -> Self {
        match state {
            ButtonState::Pressed => PointerAction::Down {
                x,
                y,
                pointer_type: PointerType::Cursor { mouse_button: button },
            },
            ButtonState::Released => PointerAction::Up {
                x,
                y,
                pointer_type: PointerType::Cursor { mouse_button: button },
            },
            ButtonState::Moved => PointerAction::Move {
                x,
                y,
                pointer_type: PointerType::Cursor { mouse_button: button },
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LayoutParams {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub margin_left: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
}

impl LayoutParams {
    pub fn new() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            x: 0.0,
            y: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            margin_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
        }
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

pub trait Inputable {
    fn input(&mut self, action: ImeAction);
}

pub trait ItemGroup {
    fn get_children(&mut self) -> &mut LinkedList<Item>;
    fn get_children_ids(&self) -> &Vec<usize>;
}

pub trait Layout {
    fn measure(&mut self, x: f32, y: f32, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode);
}

pub trait Drawable {
    fn draw(&mut self, canvas: &Canvas);
}

#[derive(Clone, Debug)]
pub struct KeyboardInput {
    pub device_id: DeviceId,
    pub event: KeyEvent,

    /// If `true`, the event was generated synthetically by winit
    /// in one of the following circumstances:
    ///
    /// * Synthetic key press events are generated for all keys pressed
    ///   when a window gains focus. Likewise, synthetic key release events
    ///   are generated for all keys pressed when a window goes out of focus.
    ///   ***Currently, this is only functional on X11 and Windows***
    ///
    /// Otherwise, this value is always `false`.
    pub is_synthetic: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonState {
    Pressed,
    Moved,
    Released,
}

impl From<ElementState> for ButtonState {
    fn from(value: ElementState) -> Self {
        match value {
            ElementState::Pressed => ButtonState::Pressed,
            ElementState::Released => ButtonState::Released,
        }
    }
}

pub trait EventInput {
    fn on_pointer_input(&mut self, action: PointerAction) -> bool {
        false
    }

    fn on_mouse_input(&mut self, device_id: DeviceId, state: ButtonState, button: MouseButton, cursor_x: f32, cursor_y: f32) -> bool {
        false
    }

    fn on_keyboard_input(&mut self, keyboard_input: KeyboardInput) -> bool {
        false
    }

    fn on_blur(&mut self) {

    }

    fn on_focus(&mut self) {}

    fn on_timer_expired(&mut self, msg:&str) {}
}

pub trait ItemTrait: EventInput + Layout + Drawable {
    fn start_timer(&self, msg:&str, duration: Duration){

    }

    fn get_id(&self) -> usize;

    fn get_path(&self) -> &ItemPath;
    fn set_path(&mut self, path: ItemPath);

    fn find_item(&self, path: &ItemPath) -> Option<&Item> {
        let mut childrens = self.get_children();
        let mut path_iter = path.iter();
        let mut next = path_iter.next();
        while let Some(index) = next {
            if let Some(child) = childrens.get(*index) {
                next = path_iter.next();
                if next.is_some() {
                    childrens = child.get_children();
                } else {
                    return Some(child);
                }
            } else {
                return None;
            }
        }
        None
    }

    fn find_item_mut(&mut self, path: &ItemPath) -> Option<&mut Item> {
        let mut childrens = self.get_children_mut();
        let mut path_iter = path.iter();
        let mut next = path_iter.next();
        while let Some(index) = next {
            if let Some(child) = childrens.get_mut(*index) {
                next = path_iter.next();
                if next.is_some() {
                    childrens = child.get_children_mut();
                } else {
                    return Some(child);
                }
            } else {
                return None;
            }
        }
        None
    }

    fn get_children(&self) -> &Vec<Item>;
    fn get_children_mut(&mut self) -> &mut Vec<Item>;
    fn has_children(&self) -> bool {
        !self.get_children().is_empty()
    }

    fn request_focus(&self);

    fn get_focusable(&self) -> BoolProperty;
    fn get_focused(&self) -> BoolProperty;
    fn get_focusable_when_clicked(&self) -> BoolProperty;

    fn get_active(&self) -> BoolProperty;
    fn get_width(&self) -> SizeProperty;
    fn get_height(&self) -> SizeProperty;
    fn get_padding_left(&self) -> FloatProperty;
    fn get_padding_top(&self) -> FloatProperty;
    fn get_padding_right(&self) -> FloatProperty;
    fn get_padding_bottom(&self) -> FloatProperty;
    fn get_margin_left(&self) -> FloatProperty;
    fn get_margin_top(&self) -> FloatProperty;
    fn get_margin_right(&self) -> FloatProperty;
    fn get_margin_bottom(&self) -> FloatProperty;
    fn get_layout_params(&self) -> &LayoutParams;

    fn get_on_click(&self) -> Option<&Box<dyn Fn() + 'static>>;
    fn get_on_pointer_input(&self) -> Option<&Box<dyn Fn(PointerAction) + 'static>>;

    fn get_on_focus(&self) -> Option<&Box<dyn Fn() + 'static>>;
    fn get_on_blur(&self) -> Option<&Box<dyn Fn() + 'static>>;
}

pub trait ForEachActive<T> {
    fn for_each_active<F: FnMut(&T)>(&mut self, f: F);
}

pub trait ForEachActiveMut<T> {
    fn for_each_active<F: FnMut(&mut T)>(&mut self, f: F);
}

impl<'a> ForEachActive<Item> for Iter<'a, Item> {
    fn for_each_active<F: FnMut(&Item)>(&mut self, mut f: F) {
        let mut next = self.next();
        while let Some(item) = next {
            if item.get_active().get() {
                f(item);
            }
            next = self.next();
        }
    }
}

impl<'a> ForEachActiveMut<Item> for IterMut<'a, Item> {
    fn for_each_active<F: FnMut(&mut Item)>(&mut self, mut f: F) {
        let mut next = self.next();
        while let Some(item) = next {
            if item.get_active().get() {
                f(item);
            }
            next = self.next();
        }
    }
}

#[allow(dead_code)]
#[macro_export]
macro_rules! item_init {
    ($name:ident{$($rest:ident:$value:expr),*})=>{
        impl $name {
            pub fn new() ->Self{
                let app = $crate::app::current_app().unwrap();
                Self{
                    path: $crate::item::ItemPath::new(),
                    id: app.new_id(),
                    app,
                    children: std::vec::Vec::new(),
                    active: $crate::property::BoolProperty::from_value(true),
                    focusable: $crate::property::BoolProperty::from_value(false),
                    focused: $crate::property::BoolProperty::from_value(false),
                    focusable_when_clicked: $crate::property::BoolProperty::from_value(false),
                    width: $crate::property::Size::Default.into(),
                    height: $crate::property::Size::Default.into(),
                    padding_left: 0.0.into(),
                    padding_top: 0.0.into(),
                    padding_right: 0.0.into(),
                    padding_bottom: 0.0.into(),
                    margin_left: 0.0.into(),
                    margin_top: 0.0.into(),
                    margin_right: 0.0.into(),
                    margin_bottom: 0.0.into(),
                    background:None.into(),
                    foreground:None.into(),
                    layout_params: $crate::item::LayoutParams::new(),
                    on_click: None,
                    on_pointer_input: None,
                    on_focus: None,
                    on_blur: None,
                    $($rest:$value,)*
                }
            }
        }
    }
}

pub enum Alignment {
    TopStart,
    Top,
    TopEnd,
    Start,
    Center,
    End,
    BottomStart,
    Bottom,
    BottomEnd,
}

impl Clone for Alignment {
    fn clone(&self) -> Self {
        match self {
            Alignment::TopStart => Alignment::TopStart,
            Alignment::Top => Alignment::Top,
            Alignment::TopEnd => Alignment::TopEnd,
            Alignment::Start => Alignment::Start,
            Alignment::Center => Alignment::Center,
            Alignment::End => Alignment::End,
            Alignment::BottomStart => Alignment::BottomStart,
            Alignment::Bottom => Alignment::Bottom,
            Alignment::BottomEnd => Alignment::BottomEnd,
        }
    }
}