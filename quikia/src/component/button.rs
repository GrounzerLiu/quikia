use skia_safe::Color;
use crate::app::ThemeColor;
use crate::item::{Gravity, Item, Rectangle, TextBlock};


pub struct Button {
    item: TextBlock,
}

impl Button {
    pub fn new() -> Self {
        Button {
            item: TextBlock::new()
        }
    }
}

impl Button {
    pub fn unwrap(self) -> Item {
        let app = self.item.get_app();
        let text_color = app.lock().unwrap().theme().get_color(ThemeColor::OnPrimary);
        let background_color = app.lock().unwrap().theme().get_color(ThemeColor::Primary);
        drop(app);
        let item =
            self.item
                .editable(false)
                .text("Filled button")
                .color(text_color)
                .unwrap()
                .height(40)
                .padding_start(24)
                .padding_end(24)
                .vertical_gravity(Gravity::Center)
                .background(
                    Rectangle::new()
                        .color(background_color)
                        .radius(20.0)
                );
        item
    }
}