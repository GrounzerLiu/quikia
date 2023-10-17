use skia_safe::{Canvas, Rect};
use macros::item;
use crate::item::{Drawable, Item, ItemGroup, Layout, MeasureMode};
use crate::item_init;
use crate::property::Size;

#[item]
pub struct Row {
    children: Vec<Item>,
}


item_init! {
    Row{
        children: Vec::new()
    }
}


impl ItemGroup for Row {
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

impl Row {
    pub fn children(mut self, children: Vec<Item>) -> Self {
        self.children = children;
        self
    }
}

impl Drawable for Row {
    fn draw(&self, canvas: &Canvas) {
        canvas.save();

        let layout_params = &self.layout_params;
        canvas.clip_rect(Rect::from_xywh(layout_params.x, layout_params.y, layout_params.width, layout_params.height), None, Some(false));

        if let Some(background) = self.background.get(){
            let background=background.lock().unwrap();
            background.draw(canvas);
        }

        self.children.iter().for_each(|child| {
            child.draw(canvas);
        });

        if let Some(foreground) = self.foreground.get(){
            let foreground=foreground.lock().unwrap();
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

impl Row {
    fn measure_children(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        self.children.iter_mut().for_each(|child| {
            if child.get_enabled().get(){
                let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, width_measure_mode, height_measure_mode);
                child.measure(child_width_measure_mode, child_height_measure_mode);
            }
            else {
                child.measure(MeasureMode::Exactly(0.0), MeasureMode::Exactly(0.0));
            }
        });
    }
}

impl Layout for Row {
    fn measure(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        self.measure_children(width_measure_mode, height_measure_mode);
        let mut layout_params = &mut self.layout_params;
        match width_measure_mode {
            MeasureMode::Exactly(width) => layout_params.width = width,
            MeasureMode::AtMost(width) => {
                match self.width.get() {
                    Size::Default => {
                        layout_params.width = self.children.iter().fold(0.0, |acc, child| {
                            acc + child.get_layout_params().width
                        });
                    }
                    Size::Fill => {
                        layout_params.width = width;
                    }
                    Size::Fixed(width) => {
                        layout_params.width = width;
                    }
                    Size::Relative(scale) => {
                        layout_params.width = width * scale;
                    }
                }
            }
        }
        match height_measure_mode {
            MeasureMode::Exactly(height) => layout_params.height = height,
            MeasureMode::AtMost(height) => {
                match self.height.get() {
                    Size::Default => {
                        layout_params.height = self.children.iter().fold(0.0, |acc, child| acc.max(child.get_layout_params().height));
                    }
                    Size::Fill => {
                        layout_params.height = height;
                    }
                    Size::Fixed(height) => {
                        layout_params.height = height;
                    }
                    Size::Relative(scale) => {
                        layout_params.height = height * scale;
                    }
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

        let mut current_x = x;
        self.children.iter_mut().for_each(|child| {
            let child_width = child.get_layout_params().width;
            let child_height = child.get_layout_params().height;
            child.layout(current_x, y, child_width, child_height);
            current_x += child_width;
        });

        if let Some(background) = self.background.get(){
            let mut background=background.lock().unwrap();
            background.measure(MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
            background.layout(layout_params.x, layout_params.y, layout_params.width, layout_params.height);
        }


        if let Some(foreground) = self.foreground.get(){
            let mut foreground=foreground.lock().unwrap();
            foreground.measure(MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
            foreground.layout(layout_params.x, layout_params.y, layout_params.width, layout_params.height);
        }
    }
}


#[macro_export]
macro_rules! row {
    ($($child:expr)*) => {
        Row::new().children({
            let children:Vec<Item> = vec![$($child.into()),*];
            let mut ids=std::collections::HashSet::new();
            children.iter().for_each(|child|{
                if !ids.insert(child.get_id()){
                    panic!("The same id cannot be reused in a item group");
                }
            });
            children
        })
    }
}

impl From<Row> for Item {
    fn from(row: Row) -> Self {
        Item::Row(row)
    }
}