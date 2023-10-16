use quikia::app::{Page, SharedApp};
use quikia::item::{Item, Rectangle};
use quikia::{closure, Color, row};
use quikia::item::Row;
use quikia::property::BoolProperty;

pub struct MainPage {
    rectangle1_enabled:BoolProperty
}

impl MainPage{
    pub fn new() -> Self{
        Self{
            rectangle1_enabled:BoolProperty::from_value(true),
        }
    }
}

impl Page for MainPage{
    fn build(&mut self, app: SharedApp) -> Item {
        row!(
            Rectangle::new()
            .enabled(&self.rectangle1_enabled)
            .id("id1")
            .color(0xff0000ff)
            .width(100)
            .height(100)

            Rectangle::new()
            .id("id2")
            .color(0xff00ff00)
            .width(200)
            .height(100)
            .on_click(closure!(move ||{
                let rectangle1_enabled_value = rectangle1_enabled.get();
                rectangle1_enabled.set(!rectangle1_enabled_value);
            },self,rectangle1_enabled)
            )

            Rectangle::new()
            .color(0xffff0000)
            .width(100)
            .height(100)
        ).background(Color::DARK_GRAY).into()
    }
}