/*use skia_safe::Rect;
use skia_safe::Canvas;
use macros::item;
use crate::item::{Alignment, Drawable, EventInput, Item, ItemGroup, ItemTrait, Layout, MeasureMode};
use crate::item_init;
use crate::property::{AlignmentProperty, Gettable, Size};

#[item]
pub struct Stack {
    children: Vec<Item>,
    content_align: AlignmentProperty,
}


item_init!(
    Stack{
        children: Vec::new(),
        content_align: Alignment::TopStart.into()
    }
);

impl Stack{
    pub fn content_align(mut self, content_align: impl Into<AlignmentProperty>) -> Self{
        self.content_align = content_align.into();
        let app = self.app.clone();
        self.content_align.lock().add_observer(
            crate::property::Observer::new_without_id(move ||{
            app.request_redraw();
        }));
        self
    }
}


impl ItemGroup for Stack {
    fn get_children(&self) -> &Vec<Item> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Item> {
        &mut self.children
    }

    fn add_child(&mut self, child: Item) {
        self.children.push(child);
    }

    fn remove_child_at(&mut self, index: usize) {
        self.children.remove(index);
    }

    fn clear_children(&mut self) {
        self.children.clear();
    }
}

impl Stack {
    pub fn children(mut self, children: Vec<Item>) -> Self {
        self.children = children;
        self
    }
}

impl Drawable for Stack {
    fn draw(&self, canvas: &Canvas) {
        canvas.save();

        let layout_params = &self.layout_params;
        canvas.clip_rect(Rect::from_xywh(layout_params.x, layout_params.y, layout_params.width, layout_params.height), None, Some(false));

        if let Some(background) = self.background.lock().as_mut() {
            //let background = background.lock().unwrap();
            background.draw(canvas);
        }

        self.children.iter().for_each(|child| {
            child.draw(canvas);
        });

        if let Some(foreground) = self.foreground.lock().as_mut() {
            //let foreground = foreground.lock().unwrap();
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
    let child_height = child.get_height().get();

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

impl Stack {
    fn measure_children(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        self.children.iter_mut().for_each(|child| {
            let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, width_measure_mode, height_measure_mode);
            child.measure(child_width_measure_mode, child_height_measure_mode);
        });
    }
}

impl Layout for Stack {
    fn measure(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        self.measure_children(width_measure_mode, height_measure_mode);

        let mut layout_params = &mut self.layout_params;
        match width_measure_mode {
            MeasureMode::Exactly(width) => {
                layout_params.width = width
            }
            MeasureMode::AtMost(width) => {
                layout_params.width = self.children.iter().fold(0.0, |acc, child| {
                    acc.max(child.get_layout_params().width)
                });
                if layout_params.width > width {
                    layout_params.width = width;
                }
            }
        }
        match height_measure_mode {
            MeasureMode::Exactly(height) => layout_params.height = height,
            MeasureMode::AtMost(height) => {
                layout_params.height = self.children.iter().fold(0.0, |acc, child| {
                    acc.max(child.get_layout_params().height)
                });
                if layout_params.height > height {
                    layout_params.height = height;
                }
            }
        }
    }

    fn layout(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let mut layout_params = &mut self.layout_params;

        layout_params.x = x;
        layout_params.y = y;
        layout_params.width = width;
        layout_params.height = height;

        for child in self.children.iter_mut() {
            let child_width = child.get_layout_params().width;
            let child_height = child.get_layout_params().height;
            match self.content_align.get(){
                Alignment::TopStart => {
                    child.layout(x, y, child_width, child_height);
                }
                Alignment::Top => {
                    child.layout(x + (width - child_width) / 2.0, y, child_width, child_height);
                }
                Alignment::TopEnd => {
                    child.layout(x + width - child_width, y, child_width, child_height);
                }
                Alignment::Start => {
                    child.layout(x, y + (height - child_height) / 2.0, child_width, child_height);
                }
                Alignment::Center => {
                    child.layout(x + (width - child_width) / 2.0, y + (height - child_height) / 2.0, child_width, child_height);
                }
                Alignment::End => {
                    child.layout(x + width - child_width, y + (height - child_height) / 2.0, child_width, child_height);
                }
                Alignment::BottomStart => {
                    child.layout(x, y + height - child_height, child_width, child_height);
                }
                Alignment::Bottom => {
                    child.layout(x + (width - child_width) / 2.0, y + height - child_height, child_width, child_height);
                }
                Alignment::BottomEnd => {
                    child.layout(x + width - child_width, y + height - child_height, child_width, child_height);
                }
            }
        }
    }
}

impl EventInput for Stack{
    fn on_pointer_input(&mut self, _action: crate::item::PointerAction) -> bool {
        false
    }
}

impl From<Stack> for Item {
    fn from(flow: Stack) -> Self {
        Item::Stack(flow)
    }
}


#[macro_export]
macro_rules! stack {
    ($($child:expr)*) => {
        $crate::item::Stack::new().children(vec![$($child.into()),*])
    }
}

#[macro_export]
macro_rules! stack_generate {
    ($count:expr,$child_create:expr) => {
        {
            let mut children = Vec::new();
            for i in 0..$count{
                let child = $child_create(i);
                children.push(child.into());
            }
            $crate::display::Stack::new().children(children)
        }
    };
}*/