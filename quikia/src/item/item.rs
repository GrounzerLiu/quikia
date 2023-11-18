use std::collections::LinkedList;
use std::ops::{Add, AddAssign, Deref, DerefMut, Sub, SubAssign};
use std::slice::{Iter, IterMut};
use std::time::Duration;
use skia_safe::Canvas;
use skia_safe::textlayout::TextDirection;
use winit::dpi::LogicalPosition;
use winit::event::{DeviceId, ElementState, Force, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase};
use crate::app::{SharedApp};
use crate::item::{Rectangle, Row, Scroller, TextBlock};
use crate::property::{BoolProperty, FloatProperty, Gettable, Size, SizeProperty};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutDirection {
    LeftToRight,
    RightToLeft,
}

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
    Scroller(Scroller),
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
            Item::Scroller(scroller) => Box::new(scroller),
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
            Item::Scroller(scroller) => scroller,
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
            Item::Scroller(scroller) => scroller,
            Item::TextBlock(text_block) => text_block,
        }
    }
}

impl Into<Item> for Rectangle {
    fn into(self) -> Item {
        Item::Rectangle(self)
    }
}

pub struct Touch {
    device_id: DeviceId,
    phase: TouchPhase,
    location: LogicalPosition<f32>,
    force: Option<Force>,
    id: u64,
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
    Touch { id: u64 },
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

    pub fn from_touch(phase: TouchPhase, location: LogicalPosition<f32>, force: Option<Force>, id: u64) -> Self {
        match phase {
            TouchPhase::Started => PointerAction::Down {
                x: location.x,
                y: location.y,
                pointer_type: PointerType::Touch { id },
            },
            TouchPhase::Moved => PointerAction::Move {
                x: location.x,
                y: location.y,
                pointer_type: PointerType::Touch { id },
            },
            TouchPhase::Ended => PointerAction::Up {
                x: location.x,
                y: location.y,
                pointer_type: PointerType::Touch { id },
            },
            TouchPhase::Cancelled => PointerAction::Cancel,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LayoutParams {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub padding_start: f32,
    pub padding_top: f32,
    pub padding_end: f32,
    pub padding_bottom: f32,
    pub margin_start: f32,
    pub margin_top: f32,
    pub margin_end: f32,
    pub margin_bottom: f32,
}

impl LayoutParams {
    pub fn new() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            x: 0.0,
            y: 0.0,
            padding_start: 0.0,
            padding_top: 0.0,
            padding_end: 0.0,
            padding_bottom: 0.0,
            margin_start: 0.0,
            margin_top: 0.0,
            margin_end: 0.0,
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
    fn measure(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode);
    fn layout(&mut self, x: f32, y: f32);
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

    fn on_touch(&mut self,
                device_id: DeviceId,
                phase: TouchPhase,
                location: LogicalPosition<f32>,
                force: Option<Force>,
                id: u64) -> bool {
        false
    }

    fn on_cursor_entered(&mut self) {}

    fn on_cursor_exited(&mut self) {}

    fn on_cursor_moved(&mut self, cursor_x: f32, cursor_y: f32) {}

    fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) -> bool {
        false
    }

    fn on_keyboard_input(&mut self, keyboard_input: KeyboardInput) -> bool {
        false
    }

    fn on_blur(&mut self) {}

    fn on_focus(&mut self) {}

    fn on_timer_expired(&mut self, msg: String) {}
}

pub trait ItemTrait: EventInput + Layout + Drawable {
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
    fn get_padding_start(&self) -> FloatProperty;
    fn get_padding_top(&self) -> FloatProperty;
    fn get_padding_end(&self) -> FloatProperty;
    fn get_padding_bottom(&self) -> FloatProperty;
    fn get_margin_start(&self) -> FloatProperty;
    fn get_margin_top(&self) -> FloatProperty;
    fn get_margin_end(&self) -> FloatProperty;
    fn get_margin_bottom(&self) -> FloatProperty;
    fn get_layout_params(&self) -> &LayoutParams;
    fn get_layout_params_mut(&mut self) -> &mut LayoutParams;
    fn set_layout_params(&mut self, layout_params: LayoutParams);

    fn get_on_click(&self) -> Option<&Box<dyn Fn() + 'static>>;
    fn get_on_pointer_input(&self) -> Option<&Box<dyn Fn(PointerAction) + 'static>>;

    fn get_on_focus(&self) -> Option<&Box<dyn Fn() + 'static>>;
    fn get_on_blur(&self) -> Option<&Box<dyn Fn() + 'static>>;
}


pub(crate) struct Timer {
    inner: crate::app::Timer,
}

impl Timer {
    pub(crate) fn start(app: &SharedApp, item_path: &ItemPath, msg: &str, duration: Duration) -> Self {
        let item_path = item_path.clone();
        let msg = msg.to_string();
        let app = app.clone();

        let timer = crate::app::Timer::new();
        timer.start(duration, move || {
            app.send_event(crate::app::UserEvent::TimerExpired(item_path.clone(), msg.clone()));
        });

        Self {
            inner: timer
        }
    }

    pub(crate) fn cancel(&self) {
        self.inner.cancel();
    }
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

pub(super) fn measure_child(child: &Item, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) -> (MeasureMode, MeasureMode) {
    let max_width = match width_measure_mode {
        MeasureMode::Exactly(width) => width,
        MeasureMode::AtMost(width) => width,
    };
    let max_height = match height_measure_mode {
        MeasureMode::Exactly(height) => height,
        MeasureMode::AtMost(height) => height,
    };

    let child_width = child.get_width().get();
    let child_height = child.get_height().get().clone();

    let child_width_measure_mode = match child_width {
        Size::Default => MeasureMode::AtMost(max_width),
        Size::Fill => MeasureMode::Exactly(max_width),
        Size::Fixed(width) => MeasureMode::Exactly(width),
        Size::Relative(scale) => MeasureMode::Exactly(max_width * scale),
    };

    let child_height_measure_mode = match child_height {
        Size::Default => MeasureMode::AtMost(max_height),
        Size::Fill => MeasureMode::Exactly(max_height),
        Size::Fixed(height) => MeasureMode::Exactly(height),
        Size::Relative(percent) => MeasureMode::Exactly(max_height * percent),
    };

    (child_width_measure_mode, child_height_measure_mode)
}

#[derive(Clone, Copy, Debug)]
pub struct LogicalX {
    x: f32,
    width: f32,
    direction: LayoutDirection,
}

impl LogicalX {
    pub fn new(x: f32, width: f32, direction: LayoutDirection) -> Self {
        Self {
            x,
            width,
            direction,
        }
    }

    pub fn to_physical(&self) -> f32 {
        match self.direction {
            LayoutDirection::LeftToRight => self.x,
            LayoutDirection::RightToLeft => self.width - self.x
        }
    }

    pub fn reverse(&self) -> Self {
        match self.direction {
            LayoutDirection::LeftToRight => Self::new(self.width - self.x, self.width, self.direction),
            LayoutDirection::RightToLeft => Self::new(self.x, self.width, self.direction)
        }
    }
}

impl Into<f32> for LogicalX {
    fn into(self) -> f32 {
        self.to_physical()
    }
}

impl Add<&LogicalX> for &LogicalX {
    type Output = LogicalX;

    fn add(self, rhs: &LogicalX) -> Self::Output {
        if self.direction != rhs.direction {
            panic!("Cannot add LogicalX with different direction!");
        }
        match self.direction {
            LayoutDirection::LeftToRight => {
                LogicalX::new(self.x + rhs.x, self.width, self.direction)
            }
            LayoutDirection::RightToLeft => {
                LogicalX::new(self.x - rhs.x, self.width, self.direction)
            }
        }
    }
}

impl Sub<&LogicalX> for &LogicalX {
    type Output = LogicalX;

    fn sub(self, rhs: &LogicalX) -> Self::Output {
        if self.direction != rhs.direction {
            panic!("Cannot add LogicalX with different direction!");
        }
        match self.direction {
            LayoutDirection::LeftToRight => {
                LogicalX::new(self.x - rhs.x, self.width, self.direction)
            }
            LayoutDirection::RightToLeft => {
                LogicalX::new(self.x + rhs.x, self.width, self.direction)
            }
        }
    }
}

impl AddAssign<&LogicalX> for LogicalX {
    fn add_assign(&mut self, rhs: &LogicalX) {
        if self.direction != rhs.direction {
            panic!("Cannot add LogicalX with different direction!");
        }
        match self.direction {
            LayoutDirection::LeftToRight => {
                self.x += rhs.x;
            }
            LayoutDirection::RightToLeft => {
                self.x -= rhs.x;
            }
        }
    }
}

impl SubAssign<&LogicalX> for LogicalX {
    fn sub_assign(&mut self, rhs: &LogicalX) {
        if self.direction != rhs.direction {
            panic!("Cannot add LogicalX with different direction!");
        }
        match self.direction {
            LayoutDirection::LeftToRight => {
                self.x -= rhs.x;
            }
            LayoutDirection::RightToLeft => {
                self.x += rhs.x;
            }
        }
    }
}

impl Add<f32> for &LogicalX {
    type Output = LogicalX;

    fn add(self, rhs: f32) -> Self::Output {
        match self.direction {
            LayoutDirection::LeftToRight => {
                LogicalX::new(self.x + rhs, self.width, self.direction)
            }
            LayoutDirection::RightToLeft => {
                LogicalX::new(self.x - rhs, self.width, self.direction)
            }
        }
    }
}

impl Sub<f32> for &LogicalX {
    type Output = LogicalX;

    fn sub(self, rhs: f32) -> Self::Output {
        match self.direction {
            LayoutDirection::LeftToRight => {
                LogicalX::new(self.x - rhs, self.width, self.direction)
            }
            LayoutDirection::RightToLeft => {
                LogicalX::new(self.x + rhs, self.width, self.direction)
            }
        }
    }
}

impl AddAssign<f32> for LogicalX {
    fn add_assign(&mut self, rhs: f32) {
        match self.direction {
            LayoutDirection::LeftToRight => {
                self.x += rhs;
            }
            LayoutDirection::RightToLeft => {
                self.x -= rhs;
            }
        }
    }
}

impl SubAssign<f32> for LogicalX {
    fn sub_assign(&mut self, rhs: f32) {
        match self.direction {
            LayoutDirection::LeftToRight => {
                self.x -= rhs;
            }
            LayoutDirection::RightToLeft => {
                self.x += rhs;
            }
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
                let layout_direction = app.layout_direction();
                Self{
                    path: $crate::item::ItemPath::new(),
                    id: app.new_id(),
                    app,
                    children: std::vec::Vec::new(),
                    layout_direction,
                    active: $crate::property::BoolProperty::from_value(true),
                    focusable: $crate::property::BoolProperty::from_value(false),
                    focused: $crate::property::BoolProperty::from_value(false),
                    focusable_when_clicked: $crate::property::BoolProperty::from_value(false),
                    is_cursor_inside: false,
                    width: $crate::property::Size::Default.into(),
                    height: $crate::property::Size::Default.into(),
                    min_width: 0.0.into(),
                    min_height: 0.0.into(),
                    max_width: std::f32::INFINITY.into(),
                    max_height: std::f32::INFINITY.into(),
                    padding_start: 0.0.into(),
                    padding_top: 0.0.into(),
                    padding_end: 0.0.into(),
                    padding_bottom: 0.0.into(),
                    margin_start: 0.0.into(),
                    margin_top: 0.0.into(),
                    margin_end: 0.0.into(),
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