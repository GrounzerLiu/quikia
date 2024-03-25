use skia_safe::Canvas;
use winit::event::{DeviceId, KeyEvent, MouseButton};
use crate::ui::{ButtonState, ImeAction, Item, MeasureMode, PointerAction, PointerType};
use crate::property::Gettable;


pub struct ItemEvent {
    /// item, canvas
    pub draw_event: Box<dyn Fn(&mut Item, &Canvas)>,
    /// item, canvas
    pub on_draw: Box<dyn Fn(&mut Item, &Canvas)>,
    /// item, width_measure_mode, height_measure_mode
    pub measure_event: Box<dyn Fn(&mut Item, MeasureMode, MeasureMode)>,
    /// item, x, y
    pub layout_event: Box<dyn Fn(&mut Item, f32, f32)>,
    /// item, device_id, state, button, x, y
    pub on_mouse_input: Box<dyn Fn(&mut Item, DeviceId, ButtonState, MouseButton, f32, f32) -> bool>,
    /// item, x, y
    pub on_cursor_moved: Box<dyn Fn(&mut Item, f32, f32) -> bool>,
    pub on_cursor_entered: Box<dyn Fn(&mut Item)>,
    pub on_cursor_exited: Box<dyn Fn(&mut Item)>,
    /// item, pointer_action
    pub on_pointer_input: Box<dyn Fn(&mut Item, PointerAction) -> bool>,
    /// item, ime_action
    pub on_ime_input: Box<dyn Fn(&mut Item, ImeAction) -> bool>,
    /// item, device_id, key_event, is_synthetic
    pub on_keyboard_input: Box<dyn Fn(&mut Item, DeviceId, KeyEvent, bool) -> bool>,
}

impl ItemEvent {
    /// item, canvas
    pub fn set_draw_event(mut self, draw_event: impl Fn(&mut Item, &Canvas) + 'static) -> Self {
        self.draw_event = Box::new(draw_event);
        self
    }
    /// item, canvas
    pub fn set_on_draw(mut self, on_draw: impl Fn(&mut Item, &Canvas) + 'static) -> Self {
        self.on_draw = Box::new(on_draw);
        self
    }
    /// item, width_measure_mode, height_measure_mode
    pub fn set_measure_event(mut self, measure_event: impl Fn(&mut Item, MeasureMode, MeasureMode) + 'static) -> Self {
        self.measure_event = Box::new(measure_event);
        self
    }

    /// item, x, y
    pub fn set_layout_event(mut self, layout_event: impl Fn(&mut Item, f32, f32) + 'static) -> Self {
        self.layout_event = Box::new(layout_event);
        self
    }

    /// item, device_id, state, button, x, y
    pub fn set_on_mouse_input(mut self, on_mouse_input: impl Fn(&mut Item, DeviceId, ButtonState, MouseButton, f32, f32) -> bool + 'static) -> Self {
        self.on_mouse_input = Box::new(on_mouse_input);
        self
    }

    /// item, x, y
    pub fn set_on_cursor_moved(mut self, on_cursor_moved: impl Fn(&mut Item, f32, f32) -> bool + 'static) -> Self {
        self.on_cursor_moved = Box::new(on_cursor_moved);
        self
    }

    pub fn set_on_cursor_entered(mut self, on_cursor_entered: impl Fn(&mut Item) + 'static) -> Self {
        self.on_cursor_entered = Box::new(on_cursor_entered);
        self
    }

    pub fn set_on_cursor_exited(mut self, on_cursor_exited: impl Fn(&mut Item) + 'static) -> Self {
        self.on_cursor_exited = Box::new(on_cursor_exited);
        self
    }

    /// item, pointer_action
    pub fn set_on_pointer_input(mut self, on_pointer_input: impl Fn(&mut Item, PointerAction) -> bool + 'static) -> Self {
        self.on_pointer_input = Box::new(on_pointer_input);
        self
    }

    /// item, ime_action
    pub fn set_on_ime_input(mut self, on_ime_input: impl Fn(&mut Item, ImeAction) -> bool + 'static) -> Self {
        self.on_ime_input = Box::new(on_ime_input);
        self
    }

    /// item, device_id, key_event, is_synthetic
    pub fn set_on_keyboard_input(mut self, on_keyboard_input: impl Fn(&mut Item, DeviceId, KeyEvent, bool) -> bool + 'static) -> Self {
        self.on_keyboard_input = Box::new(on_keyboard_input);
        self
    }
}

impl Default for ItemEvent {
    fn default() -> Self {
        Self {
            draw_event: Box::new(|item, canvas| {
                let layout_params = item.get_layout_params().clone();

                if let Some(background) = item.get_background().lock().as_mut() {
                    let layout_params = item.get_layout_params_mut();
                    layout_params.parent_x = layout_params.x();
                    layout_params.parent_y = layout_params.y();
                    background.draw(canvas);
                }

                item.on_draw(canvas);
                item.get_children().lock().iter_mut().for_each(|child| {
                    let child_layout_params = child.get_layout_params_mut();
                    child_layout_params.parent_x = layout_params.x();
                    child_layout_params.parent_y = layout_params.x();
                    child.draw(canvas);
                });

                if let Some(foreground) = item.get_foreground().lock().as_mut() {
                    let layout_params = item.get_layout_params_mut();
                    layout_params.parent_x = layout_params.x();
                    layout_params.parent_y = layout_params.y();
                    foreground.draw(canvas);
                }
            }),
            on_draw: Box::new(|_, _| {}),
            measure_event: Box::new(|_, _, _| {}),
            layout_event: Box::new(|_, _, _| {}),
            on_mouse_input: Box::new(|_, _, _, _, _, _| {
                false
            }),
            on_cursor_moved: Box::new(|_, _, _| {
                false
            }),
            on_cursor_entered: Box::new(|_| {}),
            on_cursor_exited: Box::new(|_| {}),
            on_pointer_input: Box::new(|_, _| {
                false
            }),
            on_ime_input: Box::new(|_, _| {
                false
            }),
            on_keyboard_input: Box::new(|_, _, _, _| {
                false
            }),
        }
    }
}