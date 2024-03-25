use std::rc::Rc;
use std::sync::Mutex;

use crate::property::SharedProperty;
use crate::ui::Item;

pub type ItemObject= Option<Rc<Mutex<Item>>>;

pub type ItemProperty = SharedProperty<Option<Item>>;

impl ItemProperty{
    pub fn none() -> Self{
        Self::from_value(None)
    }
}

// impl From<Item> for ItemProperty{
//     fn from(item: Item) -> Self {
//         Self::from(item)
//     }
// }

// impl From<Rectangle> for ItemProperty{
//     fn from(rectangle: Rectangle) -> Self {
//         let item = rectangle.unwrap();
//         Self::from(Some(item))
//     }
// }

/*impl From<u32> for ItemProperty{
    fn from(color: u32) -> Self {
        let item = Rectangle::new().color(color).unwrap();
        Self::from(Some(item))
    }
}

impl From<Color> for ItemProperty{
    fn from(color: Color) -> Self {
        let item = Rectangle::new().color(color).unwrap();
        Self::from(Some(item))
    }
}

impl From<ColorProperty> for ItemProperty{
    fn from(color: ColorProperty) -> Self {
        let color_property = color.clone();
        let item =Rectangle::new().color(color.get()).unwrap();
        let item_property = Self::from(Some(item));
        item_property.observe(&color_property);
        item_property
    }
}*/