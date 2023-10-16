use std::rc::Rc;
use std::sync::Mutex;
use skia_safe::Color;
use crate::item::{Item, Rectangle};
use crate::property::{ColorProperty, SharedProperty};

pub type ItemObject= Option<Rc<Mutex<Item>>>;

pub type ItemProperty = SharedProperty<ItemObject>;

impl ItemProperty{
    pub fn from_item(item: Item) -> Self{
        let item = Rc::new(Mutex::new(item));
        Self::from_generator(Box::new(move || Some(Rc::clone(&item))))
    }

    pub fn from_none() -> Self{
        Self::from_generator(Box::new(|| None))
    }
}

impl From<Item> for ItemProperty{
    fn from(item: Item) -> Self {
        Self::from_item(item)
    }
}

impl From<Rectangle> for ItemProperty{
    fn from(rectangle: Rectangle) -> Self {
        Self::from_item(rectangle.into())
    }
}

impl From<u32> for ItemProperty{
    fn from(color: u32) -> Self {
        Self::from_item(Rectangle::new().color(color).into())
    }
}

impl From<Color> for ItemProperty{
    fn from(color: Color) -> Self {
        Self::from_item(Rectangle::new().color(color).into())
    }
}

impl From<ColorProperty> for ItemProperty{
    fn from(color: ColorProperty) -> Self {
        let color_property = color.clone();
        let item = Rc::new(Mutex::new(Rectangle::new().color(color.get()).into()));
        let item_property = Self::from_generator(Box::new(move || Some(Rc::clone(&item))));
        item_property.observe(&color_property);
        item_property
    }
}

impl From<ItemObject> for ItemProperty{
    fn from(item: ItemObject) -> Self {
        Self::from_generator(Box::new(move || item.clone()))
    }
}