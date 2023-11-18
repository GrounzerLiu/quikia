use quikia::app::{Page, SharedApp};
use quikia::item::{Item, LayoutDirection, Rectangle, TextBlock};
use quikia::{clonify, Color, row, scroller, text_block};
use quikia::item::Row;
use quikia::property::{BoolProperty, Gettable};
use quikia::property::Size::Fill;

pub struct MainPage {
    rectangle1_active: BoolProperty,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            rectangle1_active: BoolProperty::from_value(true),
        }
    }
}

impl Page for MainPage {
    fn build(&mut self, app: SharedApp) -> Item {
        app.set_layout_direction(LayoutDirection::RightToLeft);
        row!(
            Rectangle::new()
            .active(&self.rectangle1_active)
            .id("id1")
            .color(Color::RED)
            .width(100)
            .height(100)
            .margin_start(10)
            .margin_end(10)

/*            Rectangle::new()
            .id("id2")
            .color(0xff00ff00)
            .width(200)
            .height(100)
            .padding_start(10)
            .padding_end(10)
            .on_click(clonify!(
                |self,rectangle1_active|{
                    let rectangle1_active_value = rectangle1_active.get();
                    rectangle1_active.set_value(!rectangle1_active_value);
                })
            )*/

            Rectangle::new()
            .color(Color::YELLOW)
            .width(100)
            .height(100)

            //scroller!(
                text_block!()
                .text(r#"في عالم اليوم، يجمع الابتكار والتقنية في مجالات متعددة مثل artificial intelligence والتحليلات الضخمة لإحداث تغييرات كبيرة. يجب أن نكون open-minded ومستعدين لapplying new methodologies والتكنولوجيا لتحقيق التقدم والازدهار."#)
                //.text_color(Color::BLACK)
                .text_size(16.0)
            //)

        )
            .width(Fill)
            .height(Fill)
            .into()
    }
}