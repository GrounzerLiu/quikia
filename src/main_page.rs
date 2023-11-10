use quikia::app::{Page, SharedApp};
use quikia::item::{Item, Rectangle, TextBlock};
use quikia::{clonify, Color, row, text_block};
use quikia::item::Row;
use quikia::property::{BoolProperty, Gettable};
use quikia::property::Size::Fill;

pub struct MainPage {
    rectangle1_active:BoolProperty
}

impl MainPage{
    pub fn new() -> Self{
        Self{
            rectangle1_active:BoolProperty::from_value(true),
        }
    }
}

impl Page for MainPage{
    fn build(&mut self, app: SharedApp) -> Item {
        row!(
            Rectangle::new()
            .active(&self.rectangle1_active)
            .id("id1")
            .color(Color::RED)
            .width(100)
            .height(100)
            .margin_left(10)
            .margin_right(10)

            Rectangle::new()
            .id("id2")
            .color(0xff00ff00)
            .width(200)
            .height(100)
            .padding_left(10)
            .padding_right(10)
            .on_click(clonify!(move ||{
                let rectangle1_active_value = rectangle1_active.get();
                rectangle1_active.set_value(!rectangle1_active_value);
            },self,rectangle1_active)
            )

            Rectangle::new()
            .color(Color::YELLOW)
            .width(100)
            .height(100)

            text_block!()
            .text(r#"Hello🙅🏽‍♀️, world! "#)
            .text_color(Color::WHITE)
            .text_size(16.0)
        )
            .width(Fill)
            .height(Fill)
            .background(Color::BLACK)
            .into()
    }
}