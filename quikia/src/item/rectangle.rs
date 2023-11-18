use std::collections::HashMap;
use std::sync::Mutex;
use skia_safe::{Canvas, Paint, Rect};
use winit::dpi::LogicalPosition;
use winit::event::{DeviceId, Force, MouseButton, TouchPhase};
use macros::item;
use crate::app::ItemMap;
use crate::item::{ButtonState, Drawable, EventInput, Item, ItemTrait, Layout, MeasureMode, PointerAction, PointerType};
use crate::item_init;
use crate::property::{ColorProperty, Gettable};

#[item]
pub struct Rectangle {
    pub(crate) color: ColorProperty,
}

item_init!(
            Rectangle{
                color:0x00000000.into()
            }
        );

impl Rectangle {
    pub fn color(mut self, color: impl Into<ColorProperty>) -> Self {
        self.color = color.into();
        let app = self.app.clone();
        self.color.lock().add_observer(
            crate::property::Observer::new_without_id(move || {
                app.request_redraw();
            }));
        self
    }
}

impl Drawable for Rectangle {
    fn draw(&mut self, canvas: &Canvas) {
        let layout_params = &self.layout_params;
        let rect_width = layout_params.width - layout_params.padding_start - layout_params.padding_end;
        let rect_height = layout_params.height - layout_params.padding_top - layout_params.padding_bottom;
        let rect_x = layout_params.x + layout_params.padding_start;
        let rect_y = layout_params.y + layout_params.padding_top;
        let rect = Rect::from_xywh(rect_x, rect_y, rect_width, rect_height);
        canvas.draw_rect(rect, &Paint::default().set_color(self.color.get()));
    }
}

impl Layout for Rectangle {
    fn measure(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        let mut layout_params = &mut self.layout_params;

        layout_params.padding_start = self.padding_start.get();
        layout_params.padding_top = self.padding_top.get();
        layout_params.padding_end = self.padding_end.get();
        layout_params.padding_bottom = self.padding_bottom.get();
        layout_params.margin_start = self.margin_start.get();
        layout_params.margin_top = self.margin_top.get();
        layout_params.margin_end = self.margin_end.get();
        layout_params.margin_bottom = self.margin_bottom.get();

        match width_measure_mode {
            MeasureMode::Exactly(width) => {
                layout_params.width = width;
            }
            MeasureMode::AtMost(_) => {
                layout_params.width = layout_params.padding_start + layout_params.padding_end;
            }
        }
        match height_measure_mode {
            MeasureMode::Exactly(height) => {
                layout_params.height = height;
            }
            MeasureMode::AtMost(_) => {
                layout_params.height = layout_params.padding_top + layout_params.padding_bottom;
            }
        }

        if let Some(background) = self.background.lock().as_mut() {
            background.measure(
                MeasureMode::Exactly(layout_params.width),
                MeasureMode::Exactly(layout_params.height),
            );
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.measure(
                MeasureMode::Exactly(layout_params.width),
                MeasureMode::Exactly(layout_params.height),
            );
        }
    }

    fn layout(&mut self, x: f32, y: f32) {
        let mut layout_params = &mut self.layout_params;
        layout_params.x = x;
        layout_params.y = y;

        if let Some(background) = self.background.lock().as_mut() {
            background.layout(x, y);
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.layout(x, y);
        }
    }
}

impl EventInput for Rectangle {
    fn on_pointer_input(&mut self, action: PointerAction) -> bool {
        if let Some(on_click) = self.get_on_click() {
            if let PointerAction::Up { .. } = action {
                on_click();
                return true;
            }
            return true;
        }
        false
    }
    fn on_mouse_input(&mut self, device_id: DeviceId, state: ButtonState, button: MouseButton, cursor_x: f32, cursor_y: f32) -> bool {
        let children_iter = (&mut self.children).iter_mut().rev();
        for child in children_iter {
            let child_layout_params = child.get_layout_params();
            if child_layout_params.contains(cursor_x, cursor_y) {
                if child.on_mouse_input(device_id, state, button, cursor_x, cursor_y) {
                    return true;
                }
            }
        }
        if self.on_pointer_input(PointerAction::from_mouse(state, button, cursor_x, cursor_y)) {
            if state == ButtonState::Pressed {
                self.app.catch_pointer(PointerType::Cursor { mouse_button: button }, &self.path);
            }
            return true;
        }
        false
    }
    fn on_touch(&mut self, device_id: DeviceId, phase: TouchPhase, location: LogicalPosition<f32>, force: Option<Force>, id: u64) -> bool {
        for child in self.children.iter_mut().rev() {
            let child_layout_params = child.get_layout_params();
            if child_layout_params.contains(location.x, location.y){
                if child.on_touch(device_id, phase, location, force, id) {
                    return true;
                }
            }
        }
        if self.on_pointer_input(PointerAction::from_touch(phase,location,force,id)){
            self.app.catch_pointer(PointerType::Touch{id}, &self.path);
            return true;
        }
        false
    }
}