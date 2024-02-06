use quikia::anim::Animation;
use quikia::app::{Page, SharedApp, ThemeColor};
use quikia::{Color, flex_layout, stack};
//use quikia::old_item::{Item, LayoutDirection, Rectangle, TextBlock};
//use quikia::{clonify, Color, row, scroller, text_block};
use quikia::item::{Gravity, Item, LayoutDirection, Rectangle};
//use quikia::old_item::Row;
use quikia::property::{BoolProperty, ColorProperty, FloatProperty, GravityProperty, Size, SizeProperty};
use quikia::property::Size::Fixed;

pub struct MainPage {
    rectangle1_active: BoolProperty,
    width: SizeProperty,
    color: ColorProperty,
    radius: FloatProperty,
    gravity: GravityProperty,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            rectangle1_active: BoolProperty::from_value(true),
            width: SizeProperty::from_value(Fixed(200.0)),
            color: Color::BLUE.into(),
            radius: FloatProperty::from_value(100.0),
            gravity: Gravity::Start.into(),
        }
    }
}

impl Page for MainPage {
    fn build(&mut self, app: SharedApp) -> Item {
        let c = app.lock().unwrap().theme().get_color(ThemeColor::Primary);
        let primary = app.lock().unwrap().theme().get_color(ThemeColor::Primary);
        let secondary = app.lock().unwrap().theme().get_color(ThemeColor::Secondary);
        let tertiary = app.lock().unwrap().theme().get_color(ThemeColor::Tertiary);

        stack!(
            flex_layout!(
                Rectangle::new()
                    .color(primary)
                    .radius(50.0)
                    .use_smooth_corners(true)
                    .unwrap()
                    .width(50)
                    .height(50)
                    .on_click(||{
                        let instant = std::time::Instant::now();
                        println!("Hello World");
                        println!("Now: {:?}", instant.elapsed());
                    })

                Rectangle::new()
                    .color(secondary)
                    .radius(60.0)
                    .unwrap()
                    .width(150)
                    .height(150)

                Rectangle::new()
                    .color(tertiary)
                    .radius(40.0)
                    .unwrap()
                    .width(100)
                    .height(100)

                Rectangle::new()
                    .color(tertiary)
                    .radius(40.0)
                    .unwrap()
                    .width(100)
                    .height(100)

                Rectangle::new()
                    .color(tertiary)
                    .radius(40.0)
                    .unwrap()
                    .width(100)
                    .height(100)

                Rectangle::new()
                    .color(tertiary)
                    .radius(40.0)
                    .unwrap()
                    .width(100)
                    .height(100)
            ).unwrap()
        ).unwrap()
            .on_click(move || {
                let layout_direction = app.layout_direction();
                app.set_layout_direction(
                    match layout_direction {
                        LayoutDirection::LeftToRight => LayoutDirection::RightToLeft,
                        LayoutDirection::RightToLeft => LayoutDirection::LeftToRight,
                    }
                );
                app.request_rebuild();
            })
    }
}