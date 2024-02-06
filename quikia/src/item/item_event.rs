use skia_safe::Canvas;
use winit::event::{DeviceId, MouseButton};
use crate::item::{ButtonState, ImeAction, Item, MeasureMode, PointerAction, PointerType};
use crate::property::Gettable;


pub struct ItemEvent {
    /// item, canvas
    pub on_draw: Box<dyn Fn(&mut Item, &Canvas)>,
    /// item, width_measure_mode, height_measure_mode
    pub on_measure: Box<dyn Fn(&mut Item, MeasureMode, MeasureMode)>,
    /// item, x, y
    pub on_layout: Box<dyn Fn(&mut Item, f32, f32)>,
    /// item, device_id, state, button, x, y
    pub on_mouse_input: Box<dyn Fn(&mut Item, DeviceId, ButtonState, MouseButton, f32, f32) -> bool>,
    /// item, pointer_action
    pub on_pointer_input: Box<dyn Fn(&mut Item, PointerAction) -> bool>,
    /// item, ime_action
    pub on_ime_input: Box<dyn Fn(&mut Item, ImeAction)>,
}

impl ItemEvent {
    /// item, canvas
    pub fn set_on_draw(mut self, on_draw: impl Fn(&mut Item, &Canvas) + 'static) -> Self {
        self.on_draw = Box::new(on_draw);
        self
    }

    /// item, width_measure_mode, height_measure_mode
    pub fn set_on_measure(mut self, on_measure: impl Fn(&mut Item, MeasureMode, MeasureMode) + 'static) -> Self {
        self.on_measure = Box::new(on_measure);
        self
    }

    /// item, x, y
    pub fn set_on_layout(mut self, on_layout: impl Fn(&mut Item, f32, f32) + 'static) -> Self {
        self.on_layout = Box::new(on_layout);
        self
    }

    /// item, device_id, state, button, x, y
    pub fn set_on_mouse_input(mut self, on_mouse_input: impl Fn(&mut Item, DeviceId, ButtonState, MouseButton, f32, f32) -> bool + 'static) -> Self {
        self.on_mouse_input = Box::new(on_mouse_input);
        self
    }

    /// item, pointer_action
    pub fn set_on_pointer_input(mut self, on_pointer_input: impl Fn(&mut Item, PointerAction) -> bool + 'static) -> Self {
        self.on_pointer_input = Box::new(on_pointer_input);
        self
    }

    /// item, ime_action
    pub fn set_on_ime_input(mut self, on_ime_input: impl Fn(&mut Item, ImeAction) + 'static) -> Self {
        self.on_ime_input = Box::new(on_ime_input);
        self
    }
}

impl Default for ItemEvent {
    fn default() -> Self {
        Self {
            on_draw: Box::new(
                |item, canvas| {
                    if let Some(background) = item.get_background().lock().as_mut() {
                        background.draw(canvas);
                    }
                    item.get_children_mut().iter_mut().for_each(|child| {
                        child.draw(canvas);
                    });
                    if let Some(foreground) = item.get_foreground().lock().as_mut() {
                        foreground.draw(canvas);
                    }
                }),
            on_measure: Box::new(
                |item, width_measure_mode, height_measure_mode| {
                    let mut layout_params = item.get_layout_params().clone();

                    layout_params.init_from_item(item);

                    match width_measure_mode {
                        MeasureMode::Specified(width) => {
                            layout_params.width = width + layout_params.padding_start + layout_params.padding_end;
                        }
                        MeasureMode::Unspecified(_) => {
                            layout_params.width = layout_params.padding_start + layout_params.padding_end;
                        }
                    }
                    layout_params.width = layout_params.width.max(item.get_min_width().get()).min(item.get_max_width().get());
                    match height_measure_mode {
                        MeasureMode::Specified(height) => {
                            layout_params.height = height + layout_params.padding_top + layout_params.padding_bottom;
                        }
                        MeasureMode::Unspecified(_) => {
                            layout_params.height = layout_params.padding_top + layout_params.padding_bottom;
                        }
                    }

                    layout_params.height = layout_params.height.max(item.get_min_height().get()).min(item.get_max_height().get());

                    if let Some(background) = item.get_background().lock().as_mut() {
                        background.measure(
                            MeasureMode::Specified(layout_params.width),
                            MeasureMode::Specified(layout_params.height),
                        );
                    }

                    if let Some(foreground) = item.get_foreground().lock().as_mut() {
                        foreground.measure(
                            MeasureMode::Specified(layout_params.width),
                            MeasureMode::Specified(layout_params.height),
                        );
                    }

                    item.set_layout_params(&layout_params);
                }
            ),
            on_layout: Box::new(
                |item, x, y| {
                    let mut layout_params = item.get_layout_params().clone();
                    layout_params.x = x;
                    layout_params.y = y;
                    item.set_layout_params(&layout_params);
                    if let Some(background) = item.get_background().lock().as_mut() {
                        background.layout(layout_params.x, layout_params.y);
                    }
                    if let Some(foreground) = item.get_foreground().lock().as_mut() {
                        foreground.layout(layout_params.x, layout_params.y);
                    }
                }
            ),
            on_mouse_input: Box::new(
                |item, device_id, state, button, x, y| {
                    let mut handled = false;
                    if let Some(background) = item.get_background().lock().as_mut() {
                        handled = background.mouse_input(device_id, state, button, x, y);
                    }

                    if !handled {
                        item.get_children_mut().iter_mut().for_each(|child| {
                            if !handled {
                                if state == ButtonState::Pressed {
                                    if child.get_layout_params().contains(x, y) {
                                        handled = child.mouse_input(device_id, state, button, x, y);
                                    }
                                } else {
                                    handled = child.mouse_input(device_id, state, button, x, y);
                                }
                            }
                        });
                    }

                    if !handled {
                        handled = item.pointer_input(PointerAction::from_mouse(state, button, x, y));
                        if handled && state == ButtonState::Pressed {
                            item.get_app().catch_pointer(PointerType::Cursor { mouse_button: button }, item.get_id())
                        }
                    }

                    if !handled {
                        if let Some(foreground) = item.get_foreground().lock().as_mut() {
                            handled = foreground.mouse_input(device_id, state, button, x, y);
                        }
                    }

                    handled
                }
            ),
            on_pointer_input: Box::new(
                |item, pointer_action| {
                    let mut handled = false;
                    match pointer_action {
                        PointerAction::Down { .. } => {
                            if item.get_on_click().is_some() {
                                handled = true;
                            }
                        }
                        PointerAction::Up { x: _x, y: _y, pointer_type: _pointer_type } => {
                            if let Some(on_click) = item.get_on_click() {
                                on_click();
                            }
                        }
                        PointerAction::Move { .. } => {}
                        PointerAction::Cancel => {}
                    }
                    handled
                }
            ),
            on_ime_input: Box::new(|_, _| {}),
        }
    }
}