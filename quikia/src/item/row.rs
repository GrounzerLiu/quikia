use skia_safe::{Canvas, Point, Rect};
use skia_safe::textlayout::TextDirection;
use winit::dpi::LogicalPosition;
use winit::event::{DeviceId, Force, MouseButton, MouseScrollDelta, TouchPhase};
use macros::item;
use crate::item::{ButtonState, Drawable, EventInput, ForEachActiveMut, Item, ItemTrait, Layout, LayoutDirection, LogicalX, MeasureMode, PointerAction, PointerType};
use crate::item::item::measure_child;
use crate::item_init;
use crate::property::{Gettable, Size};

#[item]
pub struct Row {
    point: Point,
}


item_init! {
    Row{
        point:Point::new(0.0,0.0)
    }
}

impl Drawable for Row {
    fn draw(&mut self, canvas: &Canvas) {
        canvas.save();

        let layout_params = &self.layout_params;
        canvas.clip_rect(Rect::from_xywh(layout_params.x, layout_params.y, layout_params.width, layout_params.height), None, Some(false));

        if let Some(background) = self.background.lock().as_mut() {
            background.draw(canvas);
        }

        self.children.iter_mut().for_each_active(|child| {
            child.draw(canvas);
        });

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.draw(canvas);
        }

        canvas.draw_circle(self.point, 10.0, &skia_safe::Paint::default().set_color(skia_safe::Color::BLUE));

        canvas.restore();
    }
}

impl Layout for Row {
    fn measure(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        let mut layout_params = &mut self.layout_params;

        let mut width = 0.0;
        let mut height = 0.0_f32;

        let mut remaining_width = match width_measure_mode {
            MeasureMode::Exactly(width) => width,
            MeasureMode::AtMost(width) => width,
        };

        self.children.iter_mut().for_each_active(|child| {
            let width_measure_mode = match width_measure_mode {
                MeasureMode::Exactly(_) => MeasureMode::Exactly(remaining_width),
                MeasureMode::AtMost(_) => MeasureMode::AtMost(remaining_width),
            };

            let mut child_occupied_width = 0.0;
            let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, width_measure_mode, height_measure_mode);

            let mut child_layout_params = child.get_layout_params().clone();
            child_layout_params.padding_start = child.get_margin_start().get();
            child_layout_params.padding_top = child.get_margin_top().get();
            child_layout_params.padding_end = child.get_margin_end().get();
            child_layout_params.padding_bottom = child.get_margin_bottom().get();
            child_layout_params.margin_start = child.get_margin_start().get();
            child_layout_params.margin_top = child.get_margin_top().get();
            child_layout_params.margin_end = child.get_margin_end().get();
            child_layout_params.margin_bottom = child.get_margin_bottom().get();
            child.set_layout_params(child_layout_params);

            child.measure(child_width_measure_mode, child_height_measure_mode);
            child_occupied_width = child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
            height = height.max(child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom);


            width += child_occupied_width;

            if remaining_width - child_occupied_width < 0.0 {
                remaining_width = 0.0;
            } else {
                remaining_width -= child_occupied_width;
            }
        });

        match width_measure_mode {
            MeasureMode::Exactly(measured_width) => {
                layout_params.width = measured_width;
            }
            MeasureMode::AtMost(measured_width) => {
                println!("width:{},measured_width:{}", width, measured_width);
                layout_params.width = measured_width.min(width);
            }
        }

        match height_measure_mode {
            MeasureMode::Exactly(measured_height) => {
                layout_params.height = measured_height;
            }
            MeasureMode::AtMost(measured_height) => {
                layout_params.height = measured_height.min(height);
            }
        }

        if let Some(background) = self.background.lock().as_mut() {
            background.measure(MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.measure(MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
        }
    }

    fn layout(&mut self, x: f32, y: f32) {
        self.layout_params.x = x;
        self.layout_params.y = y;
        match self.layout_direction {
            LayoutDirection::LeftToRight => {
                let mut child_x = x + self.layout_params.padding_start;
                self.children.iter_mut().for_each_active(|child| {
                    let child_layout_params = child.get_layout_params().clone();
                    child_x += child_layout_params.margin_start;
                    child.layout(child_x, y + child_layout_params.margin_top + self.layout_params.padding_top);
                    child_x += child_layout_params.width + child_layout_params.margin_end;
                })
            }
            LayoutDirection::RightToLeft => {
                let mut child_x = x + self.layout_params.width - self.layout_params.padding_start;
                self.children.iter_mut().for_each_active(|child| {
                    let child_layout_params = child.get_layout_params().clone();
                    child_x -= (child_layout_params.margin_start + child_layout_params.width);
                    child.layout(child_x, y + child_layout_params.margin_top + self.layout_params.padding_top);
                    child_x -= child_layout_params.margin_end;
                })
            }
        }

        if let Some(background) = self.background.lock().as_mut() {
            background.layout(x, y);
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.layout(x, y);
        }
    }
}


#[macro_export]
macro_rules! row {
    ($($child:expr)*) => {
        Row::new().children({
            let mut children:std::collections::LinkedList<$crate::item::Item>=std::collections::LinkedList::new();
            $(
                children.push_back($child.into());
            )*
            children
        })
    }
}

impl EventInput for Row {
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
        for child in self.children.iter_mut().rev() {
            let child_layout_params = child.get_layout_params();
            if child_layout_params.contains(cursor_x, cursor_y) {
                if child.on_mouse_input(device_id, state, button, cursor_x, cursor_y) {
                    return true;
                }
            }
        }
        if self.on_pointer_input(PointerAction::from_mouse(state, button, cursor_x, cursor_y)) {
            self.app.catch_pointer(PointerType::Cursor { mouse_button: button }, &self.path);
            return true;
        }
        false
    }

    fn on_touch(&mut self, device_id: DeviceId, phase: TouchPhase, location: LogicalPosition<f32>, force: Option<Force>, id: u64) -> bool {
        self.point = Point::new(location.x, location.y);
        self.app.request_redraw();
        self.app.catch_pointer(PointerType::Touch { id }, &self.path);
        return true;
        for child in self.children.iter_mut().rev() {
            let child_layout_params = child.get_layout_params();
            if child_layout_params.contains(location.x, location.y) {
                if child.on_touch(device_id, phase, location, force, id) {
                    return true;
                }
            }
        }
        if self.on_pointer_input(PointerAction::from_touch(phase, location, force, id)) {
            self.app.catch_pointer(PointerType::Touch { id }, &self.path);
            return true;
        }
        false
    }

    fn on_cursor_moved(&mut self, cursor_x: f32, cursor_y: f32) {
        if self.layout_params.contains(cursor_x, cursor_y) {
            if !self.is_cursor_inside {
                self.is_cursor_inside = true;
                self.on_cursor_entered();
            }
        } else {
            if self.is_cursor_inside {
                self.is_cursor_inside = false;
                self.on_cursor_exited();
            }
        }
        self.children.iter_mut().for_each_active(|child| {
            child.on_cursor_moved(cursor_x, cursor_y);
        });
    }

    fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) -> bool {
        let children_iter = (&mut self.children).iter_mut().rev();
        for child in children_iter {
            if child.on_mouse_wheel(delta) {
                return true;
            }
        }
        false
    }
}

impl From<Row> for Item {
    fn from(row: Row) -> Self {
        Item::Row(row)
    }
}