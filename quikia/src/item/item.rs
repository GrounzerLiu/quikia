use std::ops::{Deref, DerefMut};
use skia_safe::Canvas;
use crate::item::{Rectangle, Row, Stack};
use crate::property::{BoolProperty, SizeProperty};

pub enum Item{
    Rectangle(Rectangle),
    Row(Row),
    Stack(Stack),
}

impl Item{
    pub fn as_item_group(&self) -> Option<&dyn ItemGroup>{
        match self{
            Item::Rectangle(_) => None,
            Item::Row(row) => Some(row),
            Item::Stack(stack) => Some(stack),
        }
    }

    pub fn as_item_group_mut(&mut self) -> Option<&mut dyn ItemGroup>{
        match self{
            Item::Rectangle(_) => None,
            Item::Row(row) => Some(row),
            Item::Stack(stack) => Some(stack),
        }
    }
}

impl Deref for Item{
    type Target = dyn ItemTrait;
    fn deref(&self) -> &Self::Target{
        match self{
            Item::Rectangle(rectangle) => rectangle,
            Item::Row(row) => row,
            Item::Stack(stack) => stack,
        }
    }
}

impl DerefMut for Item{
    fn deref_mut(&mut self) -> &mut Self::Target{
        match self{
            Item::Rectangle(rectangle) => rectangle,
            Item::Row(row) => row,
            Item::Stack(stack) => stack,
        }
    }
}

impl Into<Item> for Rectangle{
    fn into(self) -> Item{
        Item::Rectangle(self)
    }
}


#[derive(Clone, Copy, Debug)]
pub enum MeasureMode{
    Exactly(f32),
    AtMost(f32),
}

pub trait ItemGroup{
    fn get_children(&self) -> &Vec<Item>;
    fn get_children_mut(&mut self) -> &mut Vec<Item>;
    fn add_child(&mut self, child: Item);
    fn remove_child_at(&mut self, index: usize);
    fn clear_children(&mut self);
}

pub trait Layout{
    fn measure(&mut self, width_measure_mode:MeasureMode, height_measure_mode:MeasureMode);
    fn layout(&mut self, x:f32, y:f32, width:f32, height:f32);
}

pub trait Drawable{
    fn draw(&self, canvas: &Canvas);
}

pub struct LayoutParams{
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

pub trait ItemTrait:Layout+Drawable{
    fn get_id(&self) -> usize;
    fn is_item_group(&self) -> bool{
        false
    }

    fn get_enabled(&self)->BoolProperty;
    fn get_width(&self) -> SizeProperty;
    fn get_height(&self) -> SizeProperty;
    fn get_layout_params(&self) -> &LayoutParams;
    fn get_on_click(&self) -> Option<&Box<dyn Fn() + 'static>>;
    
}

#[allow(dead_code)]
#[macro_export]
macro_rules! item_init {
    ($name:ident{$($rest:ident:$value:expr),*})=>{
        impl $name {
            pub fn new() ->Self{
                let app = $crate::app::current_app().unwrap();
                Self{
                    id: app.new_id(),
                    app,
                    enabled: $crate::property::BoolProperty::from_value(true),
                    width: $crate::property::Size::Default.into(),
                    height: $crate::property::Size::Default.into(),
                    background:None.into(),
                    foreground:None.into(),
                    layout_params: $crate::item::LayoutParams{
                        width: 0.0,
                        height: 0.0,
                        x: 0.0,
                        y: 0.0,
                    },
                    on_click: None,
                    $($rest:$value,)*
                }
            }
        }
    }
}

pub enum Alignment{
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

impl Clone for Alignment{
    fn clone(&self) -> Self {
        match self{
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