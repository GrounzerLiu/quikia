use std::collections::HashMap;
use std::ops::Range;
use std::sync::Mutex;
use icu::segmenter::GraphemeClusterSegmenter;
use skia_safe::{Canvas, Paint, Rect};
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton};
use winit::keyboard::{Key, NamedKey};
use macros::item;
use crate::app::ItemMap;
use crate::item::{ButtonState, Drawable, EventInput, ImeAction, Inputable, Item, ItemTrait, KeyboardInput, Layout, MeasureMode, PointerAction, PointerType};
use crate::item_init;
use crate::property::{ColorProperty, FloatProperty, Gettable, TextProperty};
use crate::text::{EdgeBehavior, foreach_grapheme_cluster, ParagraphWrapper, Style};


#[item]
pub struct TextBlock {
    text: TextProperty,
    text_color: Option<ColorProperty>,
    text_size: Option<FloatProperty>,
    paragraph: Option<ParagraphWrapper>,
    composing_range: Option<Range<usize>>,
    selection_range: Range<usize>,
}

#[macro_export]
macro_rules! text_block {
    () => {
        TextBlock::new()
        .focusable(true)
        .focusable_when_clicked(true)
    };
}

item_init!(
    TextBlock{
        text: "".into(),
        text_color: None,
        text_size: None,
        paragraph: None,
        composing_range: None,
        selection_range: 0..0
    }
);

impl TextBlock {
    pub fn text(mut self, text: impl Into<TextProperty>) -> Self {
        self.text = text.into();
        let app = self.app.clone();
        self.text.lock().add_observer(
            crate::property::Observer::new_without_id(move ||{
            app.request_redraw();
        }));
        self
    }

    pub fn text_color(mut self, text_color: impl Into<ColorProperty>) -> Self {
        let text_color = text_color.into();
        let app = self.app.clone();
        text_color.lock().add_observer(
            crate::property::Observer::new_without_id(move ||{
            app.request_redraw();
        }));
        self.text_color = Some(text_color);
        self
    }

    pub fn text_size(mut self, text_size: impl Into<FloatProperty>) -> Self {
        let text_size = text_size.into();
        if text_size.get()<=0.0{
            panic!("text_size must be greater than 0.0");
        }
        let app = self.app.clone();
        text_size.lock().add_observer(
            crate::property::Observer::new_without_id(move ||{
            app.request_redraw();
        }));
        self.text_size = Some(text_size);
        self
    }
}

impl Drawable for TextBlock {
    fn draw(&mut self, canvas: &Canvas) {
        if let Some(background) = self.background.lock().as_mut() {
            background.draw( canvas);
        }

        let layout_params = &self.layout_params;
        if let Some(paragraph) = &self.paragraph {
            paragraph.draw(canvas, layout_params.x, layout_params.y);
            let (x,y,h)=paragraph.get_cursor_position(self.selection_range.start);
            let x=x+layout_params.x;
            let y=y+layout_params.y;
            let rect=Rect::from_xywh(x,y,2.0,h);
            canvas.draw_rect(&rect, Paint::default().set_anti_alias(true).set_color(0xffff0000));
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.draw(canvas);
        }
    }
}

impl Layout for TextBlock {
    fn measure(&mut self, x:f32,y:f32, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        let mut layout_params = &mut self.layout_params;
        layout_params.x = x;
        layout_params.y = y;

        let mut text = self.text.lock();
        let text_ref=text.as_mut();
        if let Some(text_color)=&self.text_color{
            text_ref.set_style(Style::TextColor(text_color.get()),0..text_ref.len(),EdgeBehavior::IncludeAndInclude);
        }
        if let Some(text_size)=&self.text_size{
            text_ref.set_style(Style::FontSize(text_size.get()),0..text_ref.len(),EdgeBehavior::IncludeAndInclude);
        }

        let mut paragraph=None;
        match width_measure_mode {
            MeasureMode::Exactly(width) => {
                layout_params.width = width;
                paragraph = Some(ParagraphWrapper::new(text_ref,0..text_ref.len(), width));
            }
            MeasureMode::AtMost(width) => {
                paragraph = Some(ParagraphWrapper::new(text_ref,0..text_ref.len(), width));
                layout_params.width = paragraph.as_ref().unwrap().layout_width();
            }
        }
        match height_measure_mode {
            MeasureMode::Exactly(height) => {
                layout_params.height = height;
            }
            MeasureMode::AtMost(height) => {
                if let Some(paragraph) = &paragraph {
                    layout_params.height = paragraph.layout_height().min(height);
                } else {
                    layout_params.height = 0.0;
                }
            }
        }
        self.paragraph=paragraph;

        if let Some(background) = self.background.lock().as_mut() {
            background.measure(x,y, MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.measure(x,y, MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
        }
    }
}


impl Inputable for TextBlock{
    fn input(&mut self, action: ImeAction) {
        let mut text =self.text.lock();
        match action {
            ImeAction::Enabled => {}
            ImeAction::Enter => {
                if self.selection_range.start!=self.selection_range.end{
                    text.as_mut().remove(self.selection_range.clone());
                    self.selection_range.end=self.selection_range.start;
                }
                text.as_mut().insert(self.selection_range.start, "\n");
                self.selection_range.start+=1;
                self.selection_range.end+=1;
            }
            ImeAction::Delete => {
                let text_mut=text.as_mut();
                let text_str=text_mut.as_str();

                let selection_range=self.selection_range.clone();
                let mut delete_range=foreach_grapheme_cluster(text_str,move|range|{
                    if range.end==selection_range.end{
                        return Some(range);
                    }
                    None
                });

                if let Some(delete_range)=delete_range{
                    text_mut.remove(delete_range.clone());
                    self.selection_range.start=delete_range.start;
                    self.selection_range.end=delete_range.start;
                }
            }
            ImeAction::Preedit(pr_text, range) => {
                if let Some(composing_range)=&self.composing_range{
                    text.as_mut().remove(composing_range.clone());
                    if self.selection_range.start>=composing_range.start{
                        self.selection_range.start-=composing_range.len();
                    }
                    if self.selection_range.end>=composing_range.start{
                        self.selection_range.end-=composing_range.len();
                    }
                    self.composing_range=None;
                }
                if self.selection_range.start!=self.selection_range.end{
                    text.as_mut().remove(self.selection_range.clone());
                    self.selection_range.end=self.selection_range.start;
                }
                if let Some(range)=range{
                    text.as_mut().insert(self.selection_range.start, &pr_text);
                    self.composing_range=Some(self.selection_range.start..(self.selection_range.start+pr_text.len()));
                    self.selection_range.start+=range.0;
                    self.selection_range.end+=range.1;
                }
            }
            ImeAction::Commit(commit_text) => {
                let commit_text_len=commit_text.len();
                if self.selection_range.start!=self.selection_range.end{
                    text.as_mut().remove(self.selection_range.clone());
                    self.selection_range.end=self.selection_range.start;
                }
                text.as_mut().insert(self.selection_range.start, &commit_text);
                self.selection_range.start+=commit_text_len;
                self.selection_range.end+=commit_text_len;
            }
            ImeAction::Disabled => {}
        }
    }
}

impl EventInput for TextBlock{
    fn on_pointer_input(&mut self, action: PointerAction) -> bool {
        if self.focusable_when_clicked.get(){
            self.app.request_focus(&self.path);
            self.app.activate_ime();
        }
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
        let children_iter = (&mut self.children).iter_mut().rev();
        for child in children_iter {
            let child_layout_params = child.get_layout_params();
            if child_layout_params.contains(cursor_x, cursor_y){
                if child.on_mouse_input(device_id, state, button, cursor_x, cursor_y) {
                    return true;
                }
            }

        }
        if self.on_pointer_input(PointerAction::from_mouse(state,button,cursor_x,cursor_y)){
            if state==ButtonState::Pressed{
                self.app.catch_pointer(PointerType::Cursor { mouse_button: button }, &self.path);
            }
            return true;
        }
        false
    }
    fn on_keyboard_input(&mut self, keyboard_input: KeyboardInput) -> bool {
        match keyboard_input.event.state{
            ElementState::Pressed => {
                match keyboard_input.event.logical_key {
                    Key::Named(named_key)=>{
                        match named_key {
                            NamedKey::ArrowRight=>{
                                if let Some(paragraph)=&self.paragraph{
                                    if self.selection_range.start!=self.selection_range.end{
                                        self.selection_range.start=self.selection_range.end;
                                    }
                                    if self.selection_range.start<self.text.lock().as_ref().len(){
                                        let glyph_index=paragraph.byte_index_to_glyph_index(self.selection_range.start);
                                        let next_glyph_index=paragraph.glyph_index_to_byte_index(glyph_index+1);
                                        self.selection_range.start=next_glyph_index;
                                        self.selection_range.end=next_glyph_index;
                                    }
                                    self.app.request_redraw();
                                }

                            }
                            NamedKey::ArrowLeft=>{
                                if let Some(paragraph)=&self.paragraph{
                                    if self.selection_range.start!=self.selection_range.end{
                                        self.selection_range.start=self.selection_range.end;
                                    }
                                    if self.selection_range.start>0{
                                        let glyph_index=paragraph.byte_index_to_glyph_index(self.selection_range.start);
                                        let next_glyph_index=paragraph.glyph_index_to_byte_index(glyph_index-1);
                                        self.selection_range.start=next_glyph_index;
                                        self.selection_range.end=next_glyph_index;
                                    }
                                    self.app.request_redraw();
                                }
                            }
                            _=>{}
                        }
                    }
                    _=>{}
                }
            }
            ElementState::Released => {}
        }
        true
    }

    fn on_blur(&mut self) {
        self.app.deactivate_ime();
    }
}

impl From<TextBlock> for Item{
    fn from(value: TextBlock) -> Self {
        Item::TextBlock(value)
    }
}