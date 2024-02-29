use quikia::anim::Animation;
use quikia::app::{Page, SharedApp, ThemeColor};
use quikia::{Color, flex_layout, stack};
use quikia::component::Button;
//use quikia::{clonify, Color, row, scroller, text_block};
use quikia::item::{Gravity, Image, Item, LayoutDirection, Rectangle, Ripple, TextBlock};
use quikia::item::additional_property::{ShadowBlur, ShadowColor, ShadowOffsetY};
use quikia::property::{BoolProperty, ColorProperty, FloatProperty, GravityProperty, SizeProperty};
use quikia::property::Size::{Fill, Fixed};

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
        let on_surface = app.lock().unwrap().theme().get_color(ThemeColor::OnSurface);

        let mut shadow_offset_y = FloatProperty::from_value(5.0);
        let mut shadow_blur = FloatProperty::from_value(5.0);

        flex_layout!(
                Ripple::new().unwrap().width(100).height(100)
                
                Button::new()
                    .unwrap()
                    .on_click(|| {
                        println!("Hello World");
                    })
                
                Image::new()
                    .source("https://www.rust-lang.org/logos/rust-logo-512x512.png")
                    .item()
                    .width(100)
                    .height(100)
                
                TextBlock::new()
                    .text("Hello, world!")
                    .color(on_surface)
                    .unwrap()
                
                TextBlock::new()
                    .text("Hello, world!")
                    .color(on_surface)
                    .unwrap()
                
                Rectangle::new()
                    .color(primary)
                    .radius(50.0)
                    .use_smooth_corners(true)
                    .unwrap()
                    .width(100)
                    .height(100)
                    .shadow_color(0x66000000)
                    .shadow_offset_y(&shadow_offset_y)
                    .shadow_blur(&shadow_blur)
                    .on_cursor_entered({
                        let shadow_offset_y = shadow_offset_y.clone();
                        let shadow_blur = shadow_blur.clone();
                        move || {
                            Animation::new({
                                let shadow_offset_y = shadow_offset_y.clone();
                                let shadow_blur = shadow_blur.clone();
                                move||{
                                    shadow_offset_y.set_value(8.0);
                                    shadow_blur.set_value(8.0);
                                }}).duration(500_u32).start();
                    }})
                    .on_cursor_exited({
                        let shadow_offset_y = shadow_offset_y.clone();
                        let shadow_blur = shadow_blur.clone();
                        move || {
                            Animation::new({
                                let shadow_offset_y = shadow_offset_y.clone();
                                let shadow_blur = shadow_blur.clone();
                                move||{
                                    shadow_offset_y.set_value(5.0);
                                    shadow_blur.set_value(5.0);
                                }}).duration(500_u32).start();
                    }})
                
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
            .width(Fill)
            .height(Fill)
            .tag("flex")
            .tag("root")
            .on_click(move || {
                println!("Hello, world!");
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