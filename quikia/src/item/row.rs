use skia_safe::{Canvas, Rect};
use winit::event::{DeviceId, MouseButton};
use macros::item;
use crate::item::{ButtonState, Drawable, EventInput, ForEachActiveMut, Item, ItemTrait, Layout, MeasureMode, PointerAction, PointerType};
use crate::item_init;
use crate::property::{Gettable, Size};

#[item]
pub struct Row {
}


item_init! {
    Row{
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

        canvas.restore();
    }
}

fn measure_child(child: &Item, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) -> (MeasureMode, MeasureMode) {
    let max_width = match width_measure_mode {
        MeasureMode::Exactly(width) => width,
        MeasureMode::AtMost(width) => width,
    };
    let max_height = match height_measure_mode {
        MeasureMode::Exactly(height) => height,
        MeasureMode::AtMost(height) => height,
    };

    let child_width = child.get_width().get();
    let child_height = child.get_height().get().clone();

    let child_width_measure_mode = match child_width {
        Size::Default => MeasureMode::AtMost(max_width),
        Size::Fill => MeasureMode::Exactly(max_width),
        Size::Fixed(width) => MeasureMode::Exactly(width),
        Size::Relative(scale) => MeasureMode::Exactly(max_width * scale),
    };

    let child_height_measure_mode = match child_height {
        Size::Default => MeasureMode::AtMost(max_height),
        Size::Fill => MeasureMode::Exactly(max_height),
        Size::Fixed(height) => MeasureMode::Exactly(height),
        Size::Relative(percent) => MeasureMode::Exactly(max_height * percent),
    };

    (child_width_measure_mode, child_height_measure_mode)
}

impl Layout for Row {
    fn measure(&mut self, x: f32, y: f32, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        let mut layout_params = &mut self.layout_params;
        layout_params.x = x;
        layout_params.y = y;

        let mut width = 0.0;
        let mut height = 0.0_f32;
        let mut child_x = x;

        let mut remaining_width = match width_measure_mode {
            MeasureMode::Exactly(width) => width,
            MeasureMode::AtMost(width) => width,
        };

        self.children.iter_mut().for_each_active(|child| {

            let width_measure_mode=match width_measure_mode {
                MeasureMode::Exactly(_) => MeasureMode::Exactly(remaining_width),
                MeasureMode::AtMost(_) => MeasureMode::AtMost(remaining_width),
            };

            child_x+=child.get_margin_left().get();

            let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, width_measure_mode, height_measure_mode);
            child.measure(child_x, y, child_width_measure_mode, child_height_measure_mode);

            let child_layout_params = child.get_layout_params();
            let child_occupied_width = child_layout_params.width + child_layout_params.margin_left + child_layout_params.margin_right;

            child_x += child_layout_params.width+ child_layout_params.margin_right;

            width += child_occupied_width;
            height = height.max(child_layout_params.height+child_layout_params.margin_top+child_layout_params.margin_bottom);
            if remaining_width-child_occupied_width<0.0{
                remaining_width=0.0;
            }else{
                remaining_width -= child_occupied_width;
            }
        });

        match width_measure_mode {
            MeasureMode::Exactly(measured_width) => {
                layout_params.width = measured_width;
            }
            MeasureMode::AtMost(measured_width) => {
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
            background.measure( x, y, MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.measure( x, y, MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
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
        if let Some(on_click)=self.get_on_click(){
            if let PointerAction::Up {..}=action{
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
            if child_layout_params.contains(cursor_x, cursor_y){
                if child.on_mouse_input(device_id, state, button, cursor_x, cursor_y) {
                    return true;
                }
            }

        }
        if self.on_pointer_input(PointerAction::from_mouse(state,button,cursor_x,cursor_y)){
            self.app.catch_pointer(PointerType::Cursor { mouse_button: button }, &self.path);
            return true;
        }
        false
    }
}

impl From<Row> for Item {
    fn from(row: Row) -> Self {
        Item::Row(row)
    }
}