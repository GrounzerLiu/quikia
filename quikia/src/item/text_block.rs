use std::ops::Range;
use std::time::Duration;
use skia_safe::{Canvas, Color, Paint, Point, Rect};
use winit::event::{DeviceId, ElementState, MouseButton};
use winit::keyboard::{Key, NamedKey};
use macros::item;
use crate::item::{ButtonState, Drawable, EventInput, ImeAction, Inputable, Item, ItemTrait, KeyboardInput, Layout, MeasureMode, PointerAction, PointerType, Timer};
use crate::item_init;
use crate::property::{ColorProperty, FloatProperty, Gettable, TextProperty};
use crate::text::{EdgeBehavior, ParagraphWrapper, Style};


#[item]
pub struct TextBlock {
    text: TextProperty,
    text_color: Option<ColorProperty>,
    text_size: Option<FloatProperty>,
    paragraph: Option<ParagraphWrapper>,
    show_cursor: Option<bool>,
    cursor_timer: Option<Timer>,
    /// composing_range is the range of the composing text.
    /// (composing_range,old_selection_range)
    composing: Option<(Range<usize>, Range<usize>)>,
    selection_range: Range<usize>
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
        show_cursor: None,
        cursor_timer: None,
        composing: None,
        selection_range: 0..0
    }
);

impl TextBlock {
    pub fn text(mut self, text: impl Into<TextProperty>) -> Self {
        self.text = text.into();
        let app = self.app.clone();
        self.text.lock().add_observer(
            crate::property::Observer::new_without_id(move || {
                app.request_redraw();
            }));
        self
    }

    pub fn text_color(mut self, text_color: impl Into<ColorProperty>) -> Self {
        let text_color = text_color.into();
        let app = self.app.clone();
        text_color.lock().add_observer(
            crate::property::Observer::new_without_id(move || {
                app.request_redraw();
            }));
        self.text_color = Some(text_color);
        self
    }

    pub fn text_size(mut self, text_size: impl Into<FloatProperty>) -> Self {
        let text_size = text_size.into();
        if text_size.get() <= 0.0 {
            panic!("text_size must be greater than 0.0");
        }
        let app = self.app.clone();
        text_size.lock().add_observer(
            crate::property::Observer::new_without_id(move || {
                app.request_redraw();
            }));
        self.text_size = Some(text_size);
        self
    }
}

impl Drawable for TextBlock {
    fn draw(&mut self, canvas: &Canvas) {
        if self.show_cursor.is_none(){
            self.show_cursor = Some(true);
            self.cursor_timer=Some(self.start_timer("cursor", Duration::from_millis(1000)));
        }

        if let Some(background) = self.background.lock().as_mut() {
            background.draw(canvas);
        }

        let layout_params = &self.layout_params;
        if let Some(paragraph) = &self.paragraph {
            paragraph.draw(canvas, layout_params.x, layout_params.y);

            if let Some((composing_range,_))=&self.composing{
                let color = if let Some(text_color) = &self.text_color {
                    text_color.get()
                } else {
                    Color::BLACK
                };
                for text_box in paragraph.get_rects_for_range(composing_range.clone()).iter()
                {
                    let rect=text_box.rect;
                    let rect = Rect::from_xywh(rect.left + layout_params.x, rect.bottom + layout_params.y, rect.width(), 1.0);
                    canvas.draw_rect(&rect, Paint::default().set_anti_alias(true).set_color(color));
                };
            }

            if self.show_cursor.unwrap(){
                let (x, y, h) = paragraph.get_cursor_position(self.selection_range.start);
                let x = x + layout_params.x;
                let y = y + layout_params.y;
                let rect = Rect::from_xywh(x, y, 2.0, h);
                canvas.draw_rect(&rect, Paint::default().set_anti_alias(true).set_color(0xffff0000));
            }

        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.draw(canvas);
        }
    }
}

impl Layout for TextBlock {
    fn measure(&mut self, x: f32, y: f32, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        let mut layout_params = &mut self.layout_params;
        layout_params.x = x;
        layout_params.y = y;

        let mut text = self.text.lock();
        let text_ref = text.as_mut();
        if let Some(text_color) = &self.text_color {
            text_ref.set_style(Style::TextColor(text_color.get()), 0..text_ref.len(), EdgeBehavior::IncludeAndInclude);
        }
        if let Some(text_size) = &self.text_size {
            text_ref.set_style(Style::FontSize(text_size.get()), 0..text_ref.len(), EdgeBehavior::IncludeAndInclude);
        }

        let mut paragraph = None;
        match width_measure_mode {
            MeasureMode::Exactly(width) => {
                layout_params.width = width;
                paragraph = Some(ParagraphWrapper::new(text_ref, 0..text_ref.len(), width));
            }
            MeasureMode::AtMost(width) => {
                paragraph = Some(ParagraphWrapper::new(text_ref, 0..text_ref.len(), width));
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
        self.paragraph = paragraph;

        if let Some(background) = self.background.lock().as_mut() {
            background.measure(x, y, MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
        }

        if let Some(foreground) = self.foreground.lock().as_mut() {
            foreground.measure(x, y, MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
        }
    }
}


impl Inputable for TextBlock {
    fn input(&mut self, action: ImeAction) {
        if self.paragraph.is_none() {
            return;
        }
        let paragraph = self.paragraph.as_ref().unwrap();
        let mut text = self.text.lock();
        match action {
            ImeAction::Enabled => {
            }
            ImeAction::Enter => {
                if self.selection_range.start != self.selection_range.end {
                    text.as_mut().remove(self.selection_range.clone());
                    self.selection_range.end = self.selection_range.start;
                }
                text.as_mut().insert(self.selection_range.start, "\n");
                self.selection_range.start += 1;
                self.selection_range.end += 1;
            }
            ImeAction::Delete => {
                let text_mut = text.as_mut();

                let selection_range = self.selection_range.clone();
                if selection_range.start != selection_range.end{
                    text_mut.remove(selection_range.clone());
                    self.selection_range.end = self.selection_range.start;
                }

                if selection_range.start == 0{
                    return;
                }

                let glyph_index = paragraph.byte_index_to_glyph_index(selection_range.start);
                let prev_glyph_index = paragraph.glyph_index_to_byte_index(glyph_index - 1);
                text_mut.remove(prev_glyph_index..selection_range.start);
                self.selection_range.start = prev_glyph_index;
                self.selection_range.end = prev_glyph_index;

            }
            ImeAction::Preedit(pr_text, range) => {
                if let Some((composing_range,old_selection_range)) = &self.composing {
                    text.as_mut().remove(composing_range.clone());
                    self.selection_range.start = old_selection_range.start;
                    self.selection_range.end = old_selection_range.end;
                    self.composing = None;
                }

                if let Some((start,end)) = range {
                    text.as_mut().insert(self.selection_range.start, &pr_text);
                    self.composing = Some((self.selection_range.start..(self.selection_range.start + pr_text.len()),self.selection_range.clone()));
                    self.selection_range.start += start;
                    self.selection_range.end += end;
                }
                if let Some(cursor_timer) = &self.cursor_timer{
                    cursor_timer.cancel();
                }
                self.show_cursor = Some(true);
                self.cursor_timer=Some(self.start_timer("cursor", Duration::from_millis(1000)));
            }
            ImeAction::Commit(commit_text) => {
                let commit_text_len = commit_text.len();
                if self.selection_range.start != self.selection_range.end {
                    text.as_mut().remove(self.selection_range.clone());
                    self.selection_range.end = self.selection_range.start;
                }
                text.as_mut().insert(self.selection_range.start, &commit_text);
                self.selection_range.start += commit_text_len;
                self.selection_range.end += commit_text_len;
                if let Some(cursor_timer) = &self.cursor_timer{
                    cursor_timer.cancel();
                }
                self.show_cursor = Some(true);
                self.cursor_timer=Some(self.start_timer("cursor", Duration::from_millis(1000)));
            }
            ImeAction::Disabled => {
            }
        }
    }
}

impl EventInput for TextBlock {
    fn on_pointer_input(&mut self, action: PointerAction) -> bool {
        if self.focusable_when_clicked.get() {
            self.app.request_focus(&self.path);
            self.app.activate_ime();
        }

        if let Some(paragraph)=&self.paragraph{
            if let PointerAction::Down { x, y, .. }=action{
                let x= x - self.layout_params.x-self.layout_params.padding_left;
                let y= y - self.layout_params.y-self.layout_params.padding_top;
                let index=paragraph.get_closest_glyph_cluster_at(Point::new(x,y));
                self.selection_range.start=index;
                self.selection_range.end=index;
                if let Some(cursor_timer) = &self.cursor_timer{
                    cursor_timer.cancel();
                }
                self.show_cursor = Some(true);
                self.cursor_timer=Some(self.start_timer("cursor", Duration::from_millis(1000)));
                self.app.request_redraw();
            }
        }


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
    fn on_keyboard_input(&mut self, keyboard_input: KeyboardInput) -> bool {
        match keyboard_input.event.state {
            ElementState::Pressed => {
                match keyboard_input.event.logical_key {
                    Key::Named(named_key) => {
                        match named_key {
                            NamedKey::ArrowRight => {
                                if let Some(paragraph) = &self.paragraph {
                                    if self.selection_range.start != self.selection_range.end {
                                        self.selection_range.start = self.selection_range.end;
                                    }
                                    if self.selection_range.start < self.text.lock().as_ref().len() {
                                        let glyph_index = paragraph.byte_index_to_glyph_index(self.selection_range.start);
                                        let next_glyph_index = paragraph.glyph_index_to_byte_index(glyph_index + 1);
                                        self.selection_range.start = next_glyph_index;
                                        self.selection_range.end = next_glyph_index;
                                    }
                                    if let Some(cursor_timer) = &self.cursor_timer{
                                        cursor_timer.cancel();
                                    }
                                    self.show_cursor = Some(true);
                                    self.cursor_timer=Some(self.start_timer("cursor", Duration::from_millis(1000)));
                                    self.app.request_redraw();
                                }
                                return true;
                            }
                            NamedKey::ArrowLeft => {
                                if let Some(paragraph) = &self.paragraph {
                                    if self.selection_range.start != self.selection_range.end {
                                        self.selection_range.start = self.selection_range.end;
                                    }
                                    if self.selection_range.start > 0 {
                                        let glyph_index = paragraph.byte_index_to_glyph_index(self.selection_range.start);
                                        let next_glyph_index = paragraph.glyph_index_to_byte_index(glyph_index - 1);
                                        self.selection_range.start = next_glyph_index;
                                        self.selection_range.end = next_glyph_index;
                                    }
                                    if let Some(cursor_timer) = &self.cursor_timer{
                                        cursor_timer.cancel();
                                    }
                                    self.show_cursor = Some(true);
                                    self.cursor_timer=Some(self.start_timer("cursor", Duration::from_millis(1000)));
                                    self.app.request_redraw();
                                }
                                return true;
                            }
                            NamedKey::Enter=> {
                                self.input(ImeAction::Enter);
                                return true;
                            }
                            NamedKey::Backspace=>{
                                self.input(ImeAction::Delete);
                                return true;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            ElementState::Released => {}
        }
        if let Some(text)=keyboard_input.event.text{
            self.input(ImeAction::Commit(text.to_string()));
            return true;
        }
        true
    }

    fn on_blur(&mut self) {
        self.app.deactivate_ime();
    }

    fn on_timer_expired(&mut self, _msg: String) {
        if let Some(show_cursor) = self.show_cursor {
            self.show_cursor = Some(!show_cursor);
            self.app.request_redraw();
            self.cursor_timer=Some(self.start_timer("cursor", Duration::from_millis(1000)));
        }
    }
}

impl From<TextBlock> for Item {
    fn from(value: TextBlock) -> Self {
        Item::TextBlock(value)
    }
}