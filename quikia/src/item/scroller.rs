use std::collections::HashMap;
use std::sync::Mutex;
use skia_safe::{Canvas, Paint, Rect};
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, MouseButton, MouseScrollDelta};
use macros::item;
use crate::app::ItemMap;
use crate::item::{ButtonState, Drawable, EventInput, ForEachActiveMut, Item, ItemTrait, Layout, MeasureMode, PointerAction, PointerType};
use crate::item::item::measure_child;
use crate::item_init;
use crate::property::Gettable;

#[item]
pub struct Scroller {
    vertical_scrollable: bool,
    horizontal_scrollable: bool,
    scroll_x: Option<f32>,
    scroll_y: Option<f32>,
    progress_x: f32,
    progress_y: f32,
}

item_init!(
            Scroller{
                vertical_scrollable:true,
                horizontal_scrollable:true,
                scroll_x:None,
                scroll_y:None,
                progress_x:0.0,
                progress_y:0.0
            }
        );

impl Scroller{
    pub fn vertical_scrollable(mut self, vertical_scrollable: bool) -> Self{
        self.vertical_scrollable=vertical_scrollable;
        self
    }

    pub fn horizontal_scrollable(mut self, horizontal_scrollable: bool) -> Self{
        self.horizontal_scrollable=horizontal_scrollable;
        self
    }
}

impl Drawable for Scroller {
    fn draw(&mut self, canvas: &Canvas) {
        if let Some(background) = self.background.lock().as_mut() {
            background.draw(canvas);
        }

        if self.children.len() != 1 {
            panic!("Scroller must have one child!");
        }

        canvas.save();

        let layout_params = &self.layout_params;
        canvas.clip_rect(Rect::from_xywh(layout_params.x, layout_params.y, layout_params.width, layout_params.height), None, Some(true));

        let child = self.children.first_mut().unwrap();
        child.draw(canvas);

        canvas.restore();

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.draw(canvas);
        }
    }
}

impl Layout for Scroller {
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

        if self.children.len() != 1 {
            panic!("Scroller must have one child!");
        }

        if self.scroll_x.is_none() {
            self.scroll_x = Some(0.0);
        }

        if self.scroll_y.is_none() {
            self.scroll_y = Some(0.0);
        }

        let child = self.children.first_mut().unwrap();
        let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, width_measure_mode, MeasureMode::AtMost(f32::INFINITY));
        child.measure(
            child_width_measure_mode,
            child_height_measure_mode,
        );

        match width_measure_mode {
            MeasureMode::Exactly(width) => {
                layout_params.width = width;
            }
            MeasureMode::AtMost(width) => {
                layout_params.width = (layout_params.padding_start + layout_params.padding_end + child.get_layout_params().width).min(width);
            }
        }
        match height_measure_mode {
            MeasureMode::Exactly(height) => {
                layout_params.height = height;
            }
            MeasureMode::AtMost(height) => {
                layout_params.height = (layout_params.padding_top + layout_params.padding_bottom + child.get_layout_params().height).min(height);
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

        if self.children.len() != 1 {
            panic!("Scroller must have one child!");
        }

        let child = self.children.first_mut().unwrap();
        child.layout(x-self.scroll_x.unwrap(), y-self.scroll_y.unwrap());
    }
}

impl EventInput for Scroller {
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
        if !self.is_cursor_inside {
            return false;
        }

        if self.children.len() != 1 {
            panic!("Scroller must have one child!");
        }

        let child = self.children.first().unwrap();

        match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                *self.scroll_x.as_mut().unwrap() += x * 40.0;

                let new_scroll_y = self.scroll_y.unwrap() + y * 40.0;
                if new_scroll_y > 0.0 {
                    *self.scroll_y.as_mut().unwrap() = 0.0;
                } else if -new_scroll_y > child.get_layout_params().height - self.layout_params.height {
                    *self.scroll_y.as_mut().unwrap() = -(child.get_layout_params().height - self.layout_params.height);
                } else {
                    *self.scroll_y.as_mut().unwrap() = new_scroll_y;
                }


                self.app.request_redraw();
                return true;
            }
            MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                *self.scroll_x.as_mut().unwrap() += x as f32;
                *self.scroll_y.as_mut().unwrap() += y as f32;
                self.app.request_redraw();
                return true;
            }
        }
        false
    }
}

#[macro_export]
macro_rules! scroller {
    ($($child:expr)*) => {
        $crate::item::Scroller::new().children({
            let mut children:std::collections::LinkedList<$crate::item::Item>=std::collections::LinkedList::new();
            $(
                children.push_back($child.into());
            )*
            children
        })
    }
}

impl From<Scroller> for Item {
    fn from(item: Scroller) -> Self {
        Item::Scroller(item)
    }
}