use std::collections::HashMap;
use std::ops::{Add, Range};
use std::slice::Iter;
use icu::segmenter::{GraphemeClusterSegmenter, LineSegmenter};
use skia_safe::{Canvas, Color, FontMgr, FontStyle, Paint};
use skia_safe::textlayout::{Decoration, FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, RectHeightStyle, RectWidthStyle, TextAlign, TextDecoration, TextStyle};
use crate::text::{Style, StyledText};

/*pub struct TextLayout {
    paragraphs: Vec<ParagraphWrapper>,
    layout_width: f32,
    layout_height: f32,
}

impl TextLayout {
    pub fn new(text: &StyledText, max_width: f32) -> Self {
        let gc_segmenter = GraphemeClusterSegmenter::new();
        let text_string = text.to_string();
        let mut iter = gc_segmenter.segment_str(text_string.as_str());
        let mut paragraphs = Vec::new();
        let mut line_start = 0_usize;
        let mut last_index = iter.next();

        while let Some(index) = iter.next() {
            let str = &text[last_index.unwrap()..index];
            if str == "\r\n" || str == "\n" {
                paragraphs.push(ParagraphWrapper::new(&text, line_start..index,line_start..last_index.unwrap(), max_width));
                line_start = index;
            }
            last_index = Some(index);
        }
        let range = line_start..text.len();
        paragraphs.push(ParagraphWrapper::new(&text, range.clone(),range, max_width));
        let layout_width = paragraphs.iter().map(|paragraph| paragraph.layout_width()).fold(0.0, f32::max);
        let layout_height = paragraphs.iter().map(|paragraph| paragraph.layout_height()).fold(0.0, Add::add);

        TextLayout {
            paragraphs,
            layout_width,
            layout_height,
        }
    }

    pub fn layout_width(&self) -> f32 {
        self.layout_width
    }

    pub fn layout_height(&self) -> f32 {
        self.layout_height
    }

    pub fn draw(&self, canvas: &Canvas, x: f32, y: f32) {
        let mut y = y;
        self.paragraphs.iter().for_each(|paragraph| {
            paragraph.paragraph.paint(canvas, (x, y));
            y += paragraph.layout_height();
        });
    }
}
*/
pub struct ParagraphWrapper {
    //text:String,
    paragraph: Paragraph,
    range: Range<usize>,
    byte_to_utf16_indices: HashMap<usize, usize>,
    glyph_to_byte_indices: HashMap<usize, usize>,
    byte_to_glyph_indices: HashMap<usize, usize>,
    glyph_length:usize,
    utf16_length:usize,
    byte_length:usize,
}

impl ParagraphWrapper {
    pub fn new(text: &StyledText, range: Range<usize>, max_width: f32) -> ParagraphWrapper {
        let mut text_style = TextStyle::default();
        text_style.set_font_size(30.0);
        text_style.set_color(Color::BLACK);
        let style_segments = create_segments(text, &range, text_style);

        //let text = text[range.clone()].to_string();

        let mut paragraph_style = ParagraphStyle::default();
        paragraph_style.set_text_align(TextAlign::Start);

        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::default(), None);

        let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
        style_segments.iter().for_each(|style_segment| {
            paragraph_builder.add_style_segment(style_segment);
        });

        let mut paragraph = paragraph_builder.build();
        paragraph.layout(max_width);


        let mut utf16_indices = HashMap::new();
        utf16_indices.insert(0, 0);
        let mut glyph_to_byte_indices = HashMap::new();
        glyph_to_byte_indices.insert(0, 0);
        let mut byte_to_glyph_indices = HashMap::new();
        byte_to_glyph_indices.insert(0, 0);

        let mut glyph_index = 1;

        let segmenter = GraphemeClusterSegmenter::new();
        let mut iter = segmenter.segment_str(text.as_str());
        let mut last_utf16_index = 0;
        let mut last_index = iter.next();

        let mut glyph_length = 0;
        let mut utf16_length = 0;

        while let Some(index) = iter.next() {
            let str = &text[last_index.unwrap()..index];
            let count = str.encode_utf16().count();
            let utf16_index = last_utf16_index + count;
            utf16_indices.insert(index, utf16_index);
            utf16_length = utf16_index;
            last_utf16_index += count;
            last_index = Some(index);

            glyph_to_byte_indices.insert(glyph_index, index);
            byte_to_glyph_indices.insert(index, glyph_index);
            glyph_length = glyph_index;
            glyph_index += 1;
        }

        ParagraphWrapper {
            //text,
            paragraph,
            range,
            byte_to_utf16_indices: utf16_indices,
            glyph_to_byte_indices,
            byte_to_glyph_indices,
            glyph_length,
            utf16_length,
            byte_length: text.len(),
        }
    }


    pub fn draw(&self, canvas: &Canvas, x: f32, y: f32) {
        self.paragraph.paint(canvas, (x, y));
    }

    pub fn layout_width(&self) -> f32 {
        self.paragraph.max_intrinsic_width()
    }

    pub fn layout_height(&self) -> f32 {
        self.paragraph.height()
    }

    pub fn get_cursor_position(&self,index:usize)->(f32,f32,f32){
        let is_end = index == self.byte_length;
        let index=if is_end{
            index-1
        }
        else {
            index
        };
        let utf16_index = *self.byte_to_utf16_indices.get(&index).unwrap();
        let glyph_index = *self.byte_to_glyph_indices.get(&index).unwrap();
        let next_byte_index = *self.glyph_to_byte_indices.get(&(glyph_index + 1)).unwrap();
        let next_utf16_index = *self.byte_to_utf16_indices.get(&next_byte_index).unwrap();
        let boxes = self.paragraph.get_rects_for_range(utf16_index..next_utf16_index,RectHeightStyle::Max,RectWidthStyle::Tight);
        if boxes.len()> 1{
            panic!("something wrong, utf16_index:{},next_utf16_index:{},boxes:{:?}",utf16_index,next_utf16_index,boxes);
        }
        if boxes.len() == 0{
            panic!("something wrong, utf16_index:{},next_utf16_index:{},boxes:{:?}",utf16_index,next_utf16_index,boxes);
        }
        let box0=boxes[0];
        if is_end{
            (box0.rect.right, box0.rect.top,box0.rect.height())
        }
        else{
            (box0.rect.left, box0.rect.top,box0.rect.height())
        }
    }

    pub fn glyph_index_to_byte_index(&self, glyph_index: usize) -> usize {
        if let Some(byte_index) = self.glyph_to_byte_indices.get(&glyph_index) {
            *byte_index
        } else {
            panic!("glyph_index_to_byte_index: glyph_index not found");
        }
    }

    pub fn byte_index_to_glyph_index(&self, byte_index: usize) -> usize {
        if let Some(glyph_index) = self.byte_to_glyph_indices.get(&byte_index) {
            *glyph_index
        } else {
            panic!("the index of {} is not a grapheme cluster boundary", byte_index);
        }
    }

    pub fn glyph_length(&self)->usize{
        self.glyph_length
    }

    pub fn utf16_length(&self)->usize{
        self.utf16_length
    }

    pub fn byte_length(&self)->usize{
        self.byte_length
    }
}

fn create_segments<'text>(text: &'text StyledText, range: &Range<usize>, text_style: TextStyle) -> Vec<StyleSegment<'text>> {
    let mut text_segments = Vec::new();

    let first_segment = StyleSegment::new(text, range, &text_style);
    text_segments.push(first_segment);
    text.get_styles(range.clone()).iter().for_each(|(style, range, _)| {
        let mut index = 0;
        while index < text_segments.len() {
            if let Some(text_segment) = text_segments.get_mut(index) {
                if text_segment.range.start >= range.end {
                    break;
                }
                if range.start <= text_segment.range.start && range.end >= text_segment.range.end {
                    text_segment.apply_style(*style);
                    index += 1;
                } else if range.start > text_segment.range.start
                    && range.start < text_segment.range.end
                    && range.end > text_segment.range.start
                    && range.end < text_segment.range.end
                {
                    let left_segment = StyleSegment::new(text, &(text_segment.range.start..range.start), &text_segment.text_style);
                    let middle_segment = StyleSegment::new(text, &(range.start..range.end), &text_segment.text_style);
                    let right_segment = StyleSegment::new(text, &(range.end..text_segment.range.end), &text_segment.text_style);
                    text_segments.remove(index);
                    text_segments.push(left_segment);
                    text_segments.push(middle_segment);
                    text_segments.push(right_segment);
                } else if range.start > text_segment.range.start && range.start < text_segment.range.end {
                    let left_segment = StyleSegment::new(text, &(text_segment.range.start..range.start), &text_segment.text_style);
                    let right_segment = StyleSegment::new(text, &(range.start..text_segment.range.end), &text_segment.text_style);
                    text_segments.remove(index);
                    text_segments.push(left_segment);
                    text_segments.push(right_segment);
                } else if range.end > text_segment.range.start && range.end < text_segment.range.end {
                    let left_segment = StyleSegment::new(text, &(text_segment.range.start..range.end), &text_segment.text_style);
                    let right_segment = StyleSegment::new(text, &(range.end..text_segment.range.end), &text_segment.text_style);
                    text_segments.remove(index);
                    text_segments.push(left_segment);
                    text_segments.push(right_segment);
                } else {
                    index += 1;
                }
            }
        }
    });

    text_segments
}

#[derive(Debug)]
struct StyleSegment<'text> {
    text: &'text str,
    range: Range<usize>,
    text_style: TextStyle,
}

impl<'text> StyleSegment<'text> {
    pub fn new(text: &'text StyledText, range: &Range<usize>, def_text_style: &TextStyle) -> StyleSegment<'text> {
        StyleSegment {
            text: &text[range.clone()],
            range: range.clone(),
            text_style: def_text_style.clone(),
        }
    }

    pub fn apply_style(&mut self, style: Style) {
        match style {
            Style::Bold => {
                let font_style = self.text_style.font_style();

                if font_style == FontStyle::italic() {
                    self.text_style.set_font_style(FontStyle::bold_italic());
                } else if font_style != FontStyle::bold() {
                    self.text_style.set_font_style(FontStyle::bold());
                }
            }
            Style::Italic => {
                let font_style = self.text_style.font_style();

                if font_style == FontStyle::bold() {
                    self.text_style.set_font_style(FontStyle::bold_italic());
                } else if font_style != FontStyle::italic() {
                    self.text_style.set_font_style(FontStyle::italic());
                }
            }
            Style::Underline => {
                let mut ty=self.text_style.decoration().clone();
                ty.ty.insert(TextDecoration::UNDERLINE);
                self.text_style.set_decoration(&ty);
            }
            Style::Strikethrough => {
                let mut ty=self.text_style.decoration().clone();
                ty.ty.insert(TextDecoration::LINE_THROUGH);
                self.text_style.set_decoration(&ty);
            }
            Style::FontSize(font_size) => {
                self.text_style.set_font_size(font_size);
            }
            Style::BackgroundColor(color) => {
                self.text_style.set_background_paint(Paint::default().set_color(color));
            }
            Style::TextColor(color) => {
                self.text_style.set_color(color);
            }
        }
    }
}

trait AddStyleSegment {
    fn add_style_segment(&mut self, style_segment: &StyleSegment);
}

impl AddStyleSegment for ParagraphBuilder {
    fn add_style_segment(&mut self, style_segment: &StyleSegment) {
        self.push_style(&style_segment.text_style);
        self.add_text(&style_segment.text);
        self.pop();
    }
}