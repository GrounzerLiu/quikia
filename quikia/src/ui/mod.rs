use std::collections::{HashMap, LinkedList};


use skia_safe::Color;
use winit::dpi::LogicalPosition;
use winit::event::{ElementState, Force, MouseButton, TouchPhase};

pub use item::*;
pub use item_event::*;
pub use logical_x::*;

use crate::property::{Gettable, SharedProperty, Size};

mod item;
// mod rectangle;
mod logical_x;
mod item_event;
// mod text_block;
// mod image;
// mod ripple;
pub mod additional_property;
mod layout_params;
pub use layout_params::*;

// pub use rectangle::*;
// pub use text_block::*;
// pub use image::*;
// pub use ripple::*;

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

#[derive(Clone, Debug)]
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

pub enum AdditionalProperty{
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Color(Color),
    Item(Item),
    SharedI8(SharedProperty<i8>),
    SharedI16(SharedProperty<i16>),
    SharedI32(SharedProperty<i32>),
    SharedI64(SharedProperty<i64>),
    SharedI128(SharedProperty<i128>),
    SharedIsize(SharedProperty<isize>),
    SharedU8(SharedProperty<u8>),
    SharedU16(SharedProperty<u16>),
    SharedU32(SharedProperty<u32>),
    SharedU64(SharedProperty<u64>),
    SharedU128(SharedProperty<u128>),
    SharedUsize(SharedProperty<usize>),
    SharedF32(SharedProperty<f32>),
    SharedF64(SharedProperty<f64>),
    SharedBool(SharedProperty<bool>),
    SharedString(SharedProperty<String>),
    SharedColor(SharedProperty<Color>),
    SharedItem(SharedProperty<Item>),
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