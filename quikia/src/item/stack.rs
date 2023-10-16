//TODO: implement stack
/*use skia_safe::Rect;
use skia_safe::svg::Canvas;
use macros::item;
use crate::item::{Alignment, Drawable, Item, ItemGroup, Layout, MeasureMode};
use crate::item_init;
use crate::property::Size;

#[item]
pub struct Stack {
    children: Vec<Item>,
}


item_init! (
    Stack{
        children: Vec::new()
    }
);


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
        canvas.clip_rect(Rect::new(self.measure_x, self.measure_y, self.measure_x + self.measure_width, self.measure_y + self.measure_height), None, None);
        if let Some(background) = self.background.get() {
            let mut background = background.lock().unwrap();
            background.layout(self.measure_x, self.measure_y, self.measure_width, self.measure_height);
            background.draw(canvas);
        }
        self.children.iter().for_each(|child| {
            // if !(child.measure_x() + child.measure_width() < self.measure_x || child.measure_y() + child.measure_height() < self.measure_y
            //     || child.measure_x() > self.measure_x + self.measure_width || child.measure_y() > self.measure_y + self.measure_height) {
            //     child.draw(canvas);
            // }
            child.draw(canvas);
        });
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

        match width_measure_mode {
            MeasureMode::Exactly(width) => {
                self.measure_width = width
            }
            MeasureMode::AtMost(width) => {
                self.measure_width = self.children.iter().fold(0.0, |acc, child| {
                    acc.max(child.measure_width())
                });
                if self.measure_width > width {
                    self.measure_width = width;
                }
            }
        }
        match height_measure_mode {
            MeasureMode::Exactly(height) => self.measure_height = height,
            MeasureMode::AtMost(height) => {
                self.measure_height = self.children.iter().fold(0.0, |acc, child| {
                    acc.max(child.measure_height())
                });
                if self.measure_height > height {
                    self.measure_height = height;
                }
            }
        }
    }

    fn layout(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.measure_x = x;
        self.measure_y = y;
        self.measure_width = width;
        self.measure_height = height;

        for child in self.children.iter_mut() {
            let child_width = child.measure_width();
            let child_height = child.measure_height();
            match self.content_align.get() {
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

impl From<Stack> for Item {
    fn from(flow: Stack) -> Self {
        Item::Stack(flow)
    }
}


#[macro_export]
macro_rules! stack {
    ($($child:expr)*) => {
        $crate::display::Stack::new().children(vec![$($child.into()),*])
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