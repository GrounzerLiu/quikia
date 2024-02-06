mod item;
mod rectangle;
mod logical_x;
mod item_event;
// mod text_block;
// mod image;

pub use item::*;
pub use rectangle::*;
// pub use text_block::*;
// pub use image::*;

pub use item_event::*;
pub use logical_x::*;

use skia_safe::{Canvas, Color};
use winit::dpi::LogicalPosition;
use winit::event::{DeviceId, ElementState, Force, MouseButton, TouchPhase};
use crate::property::{Gettable, Size};
use std::collections::{HashMap, LinkedList};
use std::ops::{Add, Deref, DerefMut};

pub fn measure_child(child: &Item, parent_layout_params: &LayoutParams, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) -> (MeasureMode, MeasureMode) {
    let layout_params = child.get_layout_params();
    let max_width = match width_measure_mode {
        MeasureMode::Specified(width) => width,
        MeasureMode::Unspecified(width) => width,
    } - layout_params.margin_start - layout_params.margin_end - parent_layout_params.padding_start - parent_layout_params.margin_end;
    let max_height = match height_measure_mode {
        MeasureMode::Specified(height) => height,
        MeasureMode::Unspecified(height) => height,
    } - layout_params.margin_top - layout_params.margin_bottom - parent_layout_params.padding_top - parent_layout_params.margin_bottom;

    let child_width = child.get_width().get();
    let child_height = child.get_height().get();

    let child_width_measure_mode = match child_width {
        Size::Default => MeasureMode::Unspecified(max_width),
        Size::Fill => MeasureMode::Specified(max_width),
        Size::Fixed(width) => MeasureMode::Specified(width),
        Size::Relative(scale) => MeasureMode::Specified(max_width * scale),
    };

    let child_height_measure_mode = match child_height {
        Size::Default => MeasureMode::Unspecified(max_height),
        Size::Fill => MeasureMode::Specified(max_height),
        Size::Fixed(height) => MeasureMode::Specified(height),
        Size::Relative(percent) => MeasureMode::Specified(max_height * percent),
    };

    (child_width_measure_mode, child_height_measure_mode)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gravity {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug)]
pub enum MeasureMode {
    /// Indicates that the parent has determined an exact size for the child.
    Specified(f32),
    /// Indicates that the child can determine its own size. The value of this enum is the maximum size the child can use.
    Unspecified(f32),
}

#[derive(Clone, Debug, PartialEq)]
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
    pub max_width: f32,
    pub max_height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub float_params: HashMap<String, f32>,
    pub color_params: HashMap<String, Color>
}

impl LayoutParams {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    pub fn set_float_param(&mut self, key: impl Into<String>, value: f32) {
        self.float_params.insert(key.into(), value);
    }

    pub fn get_float_param(&self, key: impl Into<String>) -> Option<&f32> {
        self.float_params.get(&key.into())
    }

    pub fn set_color_param(&mut self, key: impl Into<String>, value: Color) {
        self.color_params.insert(key.into(), value);
    }

    pub fn get_color_param(&self, key: impl Into<String>) -> Option<&Color> {
        self.color_params.get(&key.into())
    }

    pub fn init_from_item(&mut self, item: &Item) {
        self.padding_start = item.get_padding_start().get();
        self.padding_top = item.get_padding_top().get();
        self.padding_end = item.get_padding_end().get();
        self.padding_bottom = item.get_padding_bottom().get();
        self.margin_start = item.get_margin_start().get();
        self.margin_top = item.get_margin_top().get();
        self.margin_end = item.get_margin_end().get();
        self.margin_bottom = item.get_margin_bottom().get();
        self.max_width = item.get_max_width().get();
        self.max_height = item.get_max_height().get();
        self.min_width = item.get_min_width().get();
        self.min_height = item.get_min_height().get();
    }
}


impl Default for LayoutParams {
    fn default() -> Self {
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
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            min_width: 0.0,
            min_height: 0.0,
            float_params: HashMap::new(),
            color_params: HashMap::new(),
        }
    }
}

pub type ItemPath = LinkedList<usize>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutDirection {
    LeftToRight,
    RightToLeft,
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



#[macro_export]
macro_rules! impl_item_property {
    ($struct_name:ident, $property_name:ident,$get:ident, $t:ty) => {
        impl $struct_name{
            pub fn $property_name(mut self, $property_name: impl Into<$t>) -> Self{
                self.$property_name=$property_name.into();
                let app = self.get_app();
                self.width.add_observer(
                    $crate::property::Observer::new_without_id(move ||{
                        app.request_layout();
                    })
                );
                self
            }

            pub fn $get(&self) -> $t{
                self.$property_name.clone()
            }
        }
    };
}