use quikia::anim::Animation;
use quikia::app::{Page, SharedApp, ThemeColor};
use quikia::{Color, flex_layout};
//use quikia::old_item::{Item, LayoutDirection, Rectangle, TextBlock};
//use quikia::{clonify, Color, row, scroller, text_block};
use quikia::item::{Item, LayoutDirection, Rectangle};
//use quikia::old_item::Row;
use quikia::property::{BoolProperty, ColorProperty, Size, SizeProperty};
use quikia::property::Size::Fixed;

pub struct MainPage {
    rectangle1_active: BoolProperty,
    width: SizeProperty,
    color: ColorProperty,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            rectangle1_active: BoolProperty::from_value(true),
            width: SizeProperty::from_value(Fixed(200.0)),
            color: Color::RED.into(),
        }
    }
}

impl Page for MainPage {
    fn build(&mut self, app: SharedApp) -> Item {
        let c = app.lock().unwrap().theme().get_color(ThemeColor::Primary);
        let primary = app.lock().unwrap().theme().get_color(ThemeColor::Primary);
        let secondary = app.lock().unwrap().theme().get_color(ThemeColor::Secondary);
        let tertiary = app.lock().unwrap().theme().get_color(ThemeColor::Tertiary);

        flex_layout!(
            Rectangle::new()
            .color(&self.color)
            .radius(80.0)
            .use_smooth_corners(true)
            .unwrap()
            .width(&self.width)
            .height(200)
            .on_click({
                let width = self.width.clone();
                let color = self.color.clone();
                move||{
                    Animation::new({
                        let width = width.clone();
                        let color = color.clone();
                        move||{
                            let new_width = match width.lock().as_ref(){
                                    Fixed(width) => {
                                        if *width > 250.0 {
                                            color.set_value(Color::BLUE);
                                            Fixed(200.0)
                                        }else{
                                            color.set_value(Color::RED);
                                            Fixed(300.0)
                                        }
                                    }
                                    _=>{Fixed(200.0)}
                                };
                            width.set_value(new_width);
                    }}).duration(300u64).start()
                }
            })

            // Rectangle::new()
            // .color(primary)
            // .radius(80.0)
            // .use_smooth_corners(true)
            // .unwrap()
            // .width(200)
            // .height(200)
            // .on_click(||{
            //     let instant = std::time::Instant::now();
            //     println!("Hello World");
            //     println!("Now: {:?}", instant.elapsed());
            // })
            //
            // flex_layout!(
            //     Rectangle::new()
            //     .color(primary)
            //     .radius(50.0)
            //     .use_smooth_corners(true)
            //     .unwrap()
            //     .width(50)
            //     .height(50)
            //     .on_click(||{
            //         let instant = std::time::Instant::now();
            //         println!("Hello World");
            //         println!("Now: {:?}", instant.elapsed());
            //     })
            // ).unwrap().width(100).height(100)
            //
            // Rectangle::new()
            // .color(secondary)
            // .radius(60.0)
            // .unwrap()
            // .width(150)
            // .height(150)
            //
            // Rectangle::new()
            // .color(tertiary)
            // .radius(40.0)
            // .unwrap()
            // .width(100)
            // .height(100)
            //
            // Rectangle::new()
            // .color(tertiary)
            // .radius(40.0)
            // .unwrap()
            // .width(100)
            // .height(100)
            //
            // Rectangle::new()
            // .color(tertiary)
            // .radius(40.0)
            // .unwrap()
            // .width(100)
            // .height(100)
            //
            // Rectangle::new()
            // .color(tertiary)
            // .radius(40.0)
            // .unwrap()
            // .width(100)
            // .height(100)
        ).unwrap()
            // .on_click(move || {
            //     let layout_direction = app.layout_direction();
            //     app.set_layout_direction(match layout_direction {
            //         LayoutDirection::LeftToRight => LayoutDirection::RightToLeft,
            //         LayoutDirection::RightToLeft => LayoutDirection::LeftToRight,
            //     });
            //     app.request_rebuild();
            // })

        /*        stack!(
                    flex_layout!(
                        Button::new().unwrap()

                        //small image
                        Image::new()
                        .source("https://pic.nximg.cn/file/20231209/6542964_084426978127_2.jpg")
                        .item()
                        .background(Color::RED)
                        .gravity(Gravity::Center)
                        .enable_clipping(true)
                        .width(100)
                        .height(250)


                        Rectangle::new()
                        .color(c)
                        .radius(80.0)
                        //.color(Color::RED)
                        .unwrap()
                        .width(200)
                        .height(200)

                        Rectangle::new()
                        .color(Color::BLUE)
                        .unwrap()
                        .width(100)
                        .height(100)
                        .on_click(||{
                            let instant = std::time::Instant::now();
                            println!("Hello World");
                            println!("Now: {:?}", instant.elapsed());
                        })

                        TextBlock::new()
                        .text("Hello World")
                        .unwrap()
                        .background(Color::RED)
                        .on_click(||{
                            let instant = std::time::Instant::now();
                            println!("Hello World");
                            println!("Now: {:?}", instant.elapsed());
                        })

                    ).unwrap()
                    .background(Color::YELLOW)
                    .vertical_gravity(Gravity::End)
                    .horizontal_gravity(Gravity::Center)
                    .width(Fill)
                    .height(Fill)
                ).unwrap()*/
    }
}