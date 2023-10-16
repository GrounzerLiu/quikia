use skia_safe::{Canvas, Paint, Rect};
use macros::item;
use crate::item::{Drawable, Layout, MeasureMode};
use crate::item_init;
use crate::property::ColorProperty;

#[item]
pub struct Rectangle {
    color: ColorProperty,
}

item_init!(
            Rectangle{
                color:0x00000000.into()
            }
        );

impl Rectangle {
    pub fn color(mut self, color:impl Into<ColorProperty>) -> Self{
        self.color = color.into();
        let app = self.app.clone();
        self.color.lock().add_value_changed_listener(
            crate::property::ValueChangedListener::new_without_id(move ||{
            app.need_redraw();
        }));
        self
    }
}

impl Drawable for Rectangle {
    fn draw(&self, canvas: &Canvas) {
        let layout_params = &self.layout_params;
        canvas.draw_rect(Rect::from_xywh(layout_params.x, layout_params.y, layout_params.width, layout_params.height), &Paint::default().set_color(self.color.get()));
    }
}

impl Layout for Rectangle{
    fn measure(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        let mut layout_params = &mut self.layout_params;
        match width_measure_mode {
            MeasureMode::Exactly(width) => {
                layout_params.width = width;
            }
            MeasureMode::AtMost(_) => {
                layout_params.height = 0.0;
            }
        }
        match height_measure_mode {
            MeasureMode::Exactly(height) => {
                layout_params.height = height;
            }
            MeasureMode::AtMost(_) => {
                layout_params.height = 0.0;
            }
        }
    }

    fn layout(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let mut layout_params = &mut self.layout_params;
        layout_params.x = x;
        layout_params.y = y;
        layout_params.width = width;
        layout_params.height = height;
    }
}