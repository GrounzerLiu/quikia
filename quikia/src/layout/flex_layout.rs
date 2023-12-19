use std::sync::{Arc, Mutex, RwLock};
use winit::dpi::LogicalSize;
use crate::item::{Item, ItemEvent, LayoutDirection, LayoutParams, LogicalX, measure_child, MeasureMode};
use crate::property::{Gettable, SharedProperty, Size};

#[macro_export]
macro_rules! flex_layout {
    ($($child:expr)*) => {
        $crate::layout::FlexLayout::new(vec![$($child),*])
    }
}

/// The start position of the main axis and the cross axis in the layout.
/// For example, if the AxisStart is StartTop, the main axis is from start to end, and the cross axis is from top to bottom.
#[derive(Clone, Copy, PartialEq)]
pub enum AxisStart {
    StartTop,
    EndTop,
    StartBottom,
    EndBottom,
    TopStart,
    TopEnd,
    BottomStart,
    BottomEnd,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FlexAlign {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ItemAlign {
    Start,
    End,
    Center,
    Stretch,
}


struct FlexLayoutProperties {
    axis_start: SharedProperty<AxisStart>,
    flex_wrap: SharedProperty<FlexWrap>,
    justify_content: SharedProperty<FlexAlign>,
    align_items: SharedProperty<ItemAlign>,
    align_content: SharedProperty<FlexAlign>,
    children_occupied_space: LogicalSize<f32>,
}

impl Default for FlexLayoutProperties {
    fn default() -> Self {
        FlexLayoutProperties {
            axis_start: SharedProperty::from_value(AxisStart::StartTop),
            flex_wrap: SharedProperty::from_value(FlexWrap::Wrap),
            justify_content: SharedProperty::from_value(FlexAlign::Start),
            align_items: SharedProperty::from_value(ItemAlign::Start),
            align_content: SharedProperty::from_value(FlexAlign::Start),
            children_occupied_space: LogicalSize::new(0.0, 0.0),
        }
    }
}

pub struct FlexLayout {
    item: Item,
    properties: Arc<Mutex<FlexLayoutProperties>>,
}

impl FlexLayout {
    pub fn new(children: Vec<Item>) -> Self {
        let properties = Arc::new(Mutex::new(FlexLayoutProperties::default()));
        let mut item = Item::new(
            ItemEvent::default()
                .set_on_measure({
                    let properties = properties.clone();
                    move |item, width_measure_mode, height_measure_mode| {
                        let mut properties = properties.lock().unwrap();
                        properties.children_occupied_space = LogicalSize::new(0.0, 0.0);

                        let mut layout_params = item.get_layout_params().clone();
                        layout_params.init_from_item(item);

                        let max_width = layout_params.max_width;
                        let max_height = layout_params.max_height;
                        let min_width = layout_params.min_width;
                        let min_height = layout_params.min_height;

                        let mut measure_width = 0.0_f32;
                        let mut measure_height = 0.0_f32;

                        let flex_wrap = properties.flex_wrap.get();
                        let axis_start = properties.axis_start.get();
                        let justify_content = properties.justify_content.get();
                        let align_items = properties.align_items.get();
                        let align_content = properties.align_content.get();

                        match width_measure_mode {
                            MeasureMode::Exactly(width) => {
                                match height_measure_mode {
                                    MeasureMode::Exactly(height) => {
                                        measure_width = width.clamp(min_width, max_width);
                                        measure_height = height.clamp(min_height, max_height);

                                        item.get_children_mut().iter_mut().for_each(|child| {
                                            match flex_wrap {
                                                FlexWrap::NoWrap => {
                                                    match axis_start {
                                                        AxisStart::StartTop |
                                                        AxisStart::EndTop |
                                                        AxisStart::StartBottom |
                                                        AxisStart::EndBottom => {
                                                            let (child_width_measure_mode, child_height_measure_mode) = match align_items {
                                                                ItemAlign::Stretch => {
                                                                    measure_child_stretch(true, child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                                _ => {
                                                                    measure_child(child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                            };
                                                            child.measure(child_width_measure_mode, child_height_measure_mode);
                                                            properties.children_occupied_space.width += child.get_layout_params().width + child.get_layout_params().margin_start + child.get_layout_params().margin_end;
                                                            properties.children_occupied_space.height = properties.children_occupied_space.height.max(child.get_layout_params().height + child.get_layout_params().margin_top + child.get_layout_params().margin_bottom);
                                                        }
                                                        AxisStart::TopStart |
                                                        AxisStart::TopEnd |
                                                        AxisStart::BottomStart |
                                                        AxisStart::BottomEnd => {
                                                            let (child_width_measure_mode, child_height_measure_mode) = match align_items {
                                                                ItemAlign::Stretch => {
                                                                    measure_child_stretch(false, child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                                _ => {
                                                                    measure_child(child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                            };
                                                            child.measure(child_width_measure_mode, child_height_measure_mode);
                                                            properties.children_occupied_space.width = properties.children_occupied_space.width.max(child.get_layout_params().width + child.get_layout_params().margin_start + child.get_layout_params().margin_end);
                                                            properties.children_occupied_space.height += child.get_layout_params().height + child.get_layout_params().margin_top + child.get_layout_params().margin_bottom;
                                                        }
                                                    }
                                                }
                                                FlexWrap::Wrap => {
                                                    let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, &layout_params, width_measure_mode, height_measure_mode);
                                                    child.measure(child_width_measure_mode, child_height_measure_mode);
                                                }
                                            }
                                        });
                                    }
                                    MeasureMode::AtMost(height) => {
                                        measure_width = width.clamp(min_width, max_width);
                                        item.get_children_mut().iter_mut().for_each(|child| {
                                            match flex_wrap {
                                                FlexWrap::NoWrap => {
                                                    match axis_start {
                                                        AxisStart::StartTop |
                                                        AxisStart::EndTop |
                                                        AxisStart::StartBottom |
                                                        AxisStart::EndBottom => {
                                                            let (child_width_measure_mode, child_height_measure_mode) = match align_items {
                                                                ItemAlign::Stretch => {
                                                                    measure_child_stretch(true, child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                                _ => {
                                                                    measure_child(child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                            };
                                                            child.measure(child_width_measure_mode, child_height_measure_mode);
                                                            properties.children_occupied_space.width += child.get_layout_params().width + child.get_layout_params().margin_start + child.get_layout_params().margin_end;
                                                            properties.children_occupied_space.height = properties.children_occupied_space.height.max(child.get_layout_params().height + child.get_layout_params().margin_top + child.get_layout_params().margin_bottom);
                                                        }
                                                        AxisStart::TopStart |
                                                        AxisStart::TopEnd |
                                                        AxisStart::BottomStart |
                                                        AxisStart::BottomEnd => {
                                                            let (child_width_measure_mode, child_height_measure_mode) = match align_items {
                                                                ItemAlign::Stretch => {
                                                                    measure_child_stretch(false, child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                                _ => {
                                                                    measure_child(child, &layout_params, width_measure_mode, height_measure_mode)
                                                                }
                                                            };
                                                            child.measure(child_width_measure_mode, child_height_measure_mode);
                                                            properties.children_occupied_space.width = properties.children_occupied_space.width.max(child.get_layout_params().width + child.get_layout_params().margin_start + child.get_layout_params().margin_end);
                                                            properties.children_occupied_space.height += child.get_layout_params().height + child.get_layout_params().margin_top + child.get_layout_params().margin_bottom;
                                                        }
                                                    }
                                                    measure_height = (properties.children_occupied_space.height + layout_params.padding_top + layout_params.padding_bottom).clamp(min_height, max_height);
                                                }
                                                FlexWrap::Wrap => {}
                                            }
                                        });
                                    }
                                }
                            }
                            MeasureMode::AtMost(width) => {
                                match height_measure_mode {
                                    MeasureMode::Exactly(height) => {
                                        measure_height = height.clamp(min_height, max_height);
                                        item.get_children_mut().iter_mut().for_each(|child| {
                                            match flex_wrap {
                                                FlexWrap::NoWrap => {
                                                    let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, &layout_params, width_measure_mode, height_measure_mode);
                                                    child.measure(child_width_measure_mode, child_height_measure_mode);
                                                    match axis_start {
                                                        AxisStart::StartTop |
                                                        AxisStart::EndTop |
                                                        AxisStart::StartBottom |
                                                        AxisStart::EndBottom => {
                                                            properties.children_occupied_space.width += child.get_layout_params().width + child.get_layout_params().margin_start + child.get_layout_params().margin_end;
                                                            properties.children_occupied_space.height = properties.children_occupied_space.height.max(child.get_layout_params().height + child.get_layout_params().margin_top + child.get_layout_params().margin_bottom);
                                                        }
                                                        AxisStart::TopStart => {}
                                                        AxisStart::TopEnd => {}
                                                        AxisStart::BottomStart => {}
                                                        AxisStart::BottomEnd => {}
                                                    }
                                                    measure_width = (properties.children_occupied_space.width + layout_params.padding_start + layout_params.padding_end).clamp(min_width, max_width);
                                                }
                                                FlexWrap::Wrap => {}
                                            }
                                        });
                                    }
                                    MeasureMode::AtMost(height) => {
                                        item.get_children_mut().iter_mut().for_each(|child| {
                                            match flex_wrap {
                                                FlexWrap::NoWrap => {
                                                    let (child_width_measure_mode, child_height_measure_mode) = measure_child(child, &layout_params, width_measure_mode, height_measure_mode);
                                                    child.measure(child_width_measure_mode, child_height_measure_mode);
                                                    match axis_start {
                                                        AxisStart::StartTop |
                                                        AxisStart::EndTop |
                                                        AxisStart::StartBottom |
                                                        AxisStart::EndBottom => {
                                                            properties.children_occupied_space.width += child.get_layout_params().width + child.get_layout_params().margin_start + child.get_layout_params().margin_end;
                                                            properties.children_occupied_space.height = properties.children_occupied_space.height.max(child.get_layout_params().height + child.get_layout_params().margin_top + child.get_layout_params().margin_bottom);
                                                        }
                                                        AxisStart::TopStart => {}
                                                        AxisStart::TopEnd => {}
                                                        AxisStart::BottomStart => {}
                                                        AxisStart::BottomEnd => {}
                                                    }
                                                    measure_width = (properties.children_occupied_space.width + layout_params.padding_start + layout_params.padding_end).clamp(min_width, max_width);
                                                    measure_height = (properties.children_occupied_space.height + layout_params.padding_top + layout_params.padding_bottom).clamp(min_height, max_height);
                                                }
                                                FlexWrap::Wrap => {}
                                            }
                                        });
                                    }
                                }
                            }
                        }

                        layout_params.width = measure_width;
                        layout_params.height = measure_height;

                        if let Some(background) = item.get_background().lock().as_mut() {
                            background.measure(MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
                        }

                        if let Some(foreground) = item.get_foreground().lock().as_mut() {
                            foreground.measure(MeasureMode::Exactly(layout_params.width), MeasureMode::Exactly(layout_params.height));
                        }

                        item.set_layout_params(&layout_params);
                    }
                })
                .set_on_layout({
                    let properties = properties.clone();
                    move |item, x, y| {
                        let properties = properties.lock().unwrap();
                        let flex_wrap = properties.flex_wrap.get();
                        let axis_start = properties.axis_start.get();
                        let justify_content = properties.justify_content.get();
                        let align_items = properties.align_items.get();
                        let align_content = properties.align_content.get();

                        let mut layout_params = item.get_layout_params().clone();
                        let mut width = layout_params.width;
                        let mut height = layout_params.height;
                        layout_params.x = x;
                        layout_params.y = y;

                        if let Some(background) = item.get_background().lock().as_mut() {
                            background.layout(x, y);
                        }

                        if let Some(foreground) = item.get_foreground().lock().as_mut() {
                            foreground.layout(x, y);
                        }

                        item.set_layout_params(&layout_params);

                        let x = LogicalX::new(item.get_layout_direction().get(), x, x, layout_params.width);

                        let children_len = item.get_children().len();

                        match flex_wrap {
                            FlexWrap::NoWrap => {
                                fn calculate_y(y: f32, axis_start: AxisStart, align_items: ItemAlign, layout_params: &LayoutParams, child_layout_params: &LayoutParams) -> f32 {
                                    let height = layout_params.height;
                                    match axis_start {
                                        AxisStart::StartTop | AxisStart::EndTop => {
                                            match align_items {
                                                ItemAlign::Start | ItemAlign::Stretch => {
                                                    y + layout_params.padding_top + child_layout_params.margin_top
                                                }
                                                ItemAlign::End => {
                                                    y + height - layout_params.padding_bottom - child_layout_params.margin_bottom - child_layout_params.height
                                                }
                                                ItemAlign::Center => {
                                                    y + (height - layout_params.padding_top - layout_params.padding_bottom - child_layout_params.height) / 2.0
                                                }
                                            }
                                        }
                                        AxisStart::StartBottom | AxisStart::EndBottom => {
                                            match align_items {
                                                ItemAlign::Start | ItemAlign::Stretch => {
                                                    y + height - layout_params.padding_bottom - child_layout_params.margin_bottom - child_layout_params.height
                                                }
                                                ItemAlign::End => {
                                                    y + layout_params.padding_top + child_layout_params.margin_top
                                                }
                                                ItemAlign::Center => {
                                                    y + (height - layout_params.padding_top - layout_params.padding_bottom - child_layout_params.height) / 2.0
                                                }
                                            }
                                        }
                                        _ => { 0.0 }
                                    }
                                }

                                fn calculate_x(x: LogicalX, axis_start: AxisStart, align_items: ItemAlign, layout_params: &LayoutParams, child_layout_params: &LayoutParams) -> LogicalX {
                                    let width = layout_params.width;
                                    match axis_start {
                                        AxisStart::TopStart | AxisStart::BottomStart => {
                                            match align_items {
                                                ItemAlign::Start | ItemAlign::Stretch => {
                                                    x + layout_params.padding_start + child_layout_params.margin_start
                                                }
                                                ItemAlign::End => {
                                                    x + width - layout_params.padding_end - child_layout_params.margin_end - child_layout_params.width
                                                }
                                                ItemAlign::Center => {
                                                    x + (width - layout_params.padding_start - layout_params.padding_end - child_layout_params.width) / 2.0
                                                }
                                            }
                                        }
                                        AxisStart::TopEnd | AxisStart::BottomEnd => {
                                            match align_items {
                                                ItemAlign::Start | ItemAlign::Stretch => {
                                                    x + width - layout_params.padding_end - child_layout_params.margin_end - child_layout_params.width
                                                }
                                                ItemAlign::End => {
                                                    x + layout_params.padding_start + child_layout_params.margin_start
                                                }
                                                ItemAlign::Center => {
                                                    x + (width - layout_params.padding_start - layout_params.padding_end - child_layout_params.width) / 2.0
                                                }
                                            }
                                        }
                                        _ => { LogicalX::new(LayoutDirection::LeftToRight, 0.0, 0.0, 0.0) }
                                    }
                                }

                                match axis_start {
                                    AxisStart::StartTop | AxisStart::StartBottom => {
                                        match justify_content {
                                            FlexAlign::Start => {
                                                let mut child_x = x + layout_params.padding_start;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x + child_layout_params.margin_start;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                });
                                            }
                                            FlexAlign::End => {
                                                let mut child_x = x + width - properties.children_occupied_space.width - layout_params.padding_end;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x + child_layout_params.margin_start;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                });
                                            }
                                            FlexAlign::Center => {
                                                let mut child_x = x + (width - properties.children_occupied_space.width) / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x + child_layout_params.margin_start;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                });
                                            }
                                            FlexAlign::SpaceBetween => {
                                                let mut space = (width - properties.children_occupied_space.width - layout_params.padding_start - layout_params.padding_end) / (children_len - 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_x = x + layout_params.padding_start;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x + child_layout_params.margin_start;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end + space;
                                                });
                                            }
                                            FlexAlign::SpaceAround => {
                                                let mut space = (width - properties.children_occupied_space.width - layout_params.padding_start - layout_params.padding_end) / children_len as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_x = x + layout_params.padding_start + space / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x + child_layout_params.margin_start;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end + space;
                                                });
                                            }
                                            FlexAlign::SpaceEvenly => {
                                                let mut space = (width - properties.children_occupied_space.width - layout_params.padding_start - layout_params.padding_end) / (children_len + 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_x = x + layout_params.padding_start + space;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x + child_layout_params.margin_start;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end + space;
                                                });
                                            }
                                        }
                                    }
                                    AxisStart::EndTop | AxisStart::EndBottom => {
                                        match justify_content {
                                            FlexAlign::Start => {
                                                let mut child_x = x + layout_params.padding_start + properties.children_occupied_space.width;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x - child_layout_params.width - child_layout_params.margin_end;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x -= child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                });
                                            }
                                            FlexAlign::End => {
                                                let mut child_x = x + width - layout_params.padding_end;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x - child_layout_params.width - child_layout_params.margin_end;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x -= child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                });
                                            }
                                            FlexAlign::Center => {
                                                let mut child_x = x + width - (width - properties.children_occupied_space.width) / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x - child_layout_params.width - child_layout_params.margin_end;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x -= child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                });
                                            }
                                            FlexAlign::SpaceBetween => {
                                                let mut space = (width - properties.children_occupied_space.width - layout_params.padding_start - layout_params.padding_end) / (children_len - 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_x = x + width - layout_params.padding_end;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x - child_layout_params.width - child_layout_params.margin_end;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x -= child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end + space;
                                                });
                                            }
                                            FlexAlign::SpaceAround => {
                                                let mut space = (width - properties.children_occupied_space.width - layout_params.padding_start - layout_params.padding_end) / children_len as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_x = x + width - layout_params.padding_end - space / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x - child_layout_params.width - child_layout_params.margin_end;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x -= child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end + space;
                                                });
                                            }
                                            FlexAlign::SpaceEvenly => {
                                                let mut space = (width - properties.children_occupied_space.width - layout_params.padding_start - layout_params.padding_end) / (children_len + 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_x = x + width - layout_params.padding_end - space;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let x = child_x - child_layout_params.width - child_layout_params.margin_end;
                                                    let y = calculate_y(y, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_x -= child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end + space;
                                                });
                                            }
                                        }
                                    }
                                    AxisStart::TopStart | AxisStart::TopEnd => {
                                        match justify_content {
                                            FlexAlign::Start => {
                                                let mut child_y = y + layout_params.padding_top;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y + child_layout_params.margin_top;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y += child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom;
                                                });
                                            }
                                            FlexAlign::End => {
                                                let mut child_y = y + height - properties.children_occupied_space.height - layout_params.padding_bottom;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y + child_layout_params.margin_top;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y += child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom;
                                                });
                                            }
                                            FlexAlign::Center => {
                                                let mut child_y = y + (height - properties.children_occupied_space.height) / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y + child_layout_params.margin_top;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y += child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom;
                                                });
                                            }
                                            FlexAlign::SpaceBetween => {
                                                let mut space = (height - properties.children_occupied_space.height - layout_params.padding_top - layout_params.padding_bottom) / (children_len - 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_y = y + layout_params.padding_top;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y + child_layout_params.margin_top;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y += child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom + space;
                                                });
                                            }
                                            FlexAlign::SpaceAround => {
                                                let mut space = (height - properties.children_occupied_space.height - layout_params.padding_top - layout_params.padding_bottom) / children_len as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_y = y + layout_params.padding_top + space / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y + child_layout_params.margin_top;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y += child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom + space;
                                                });
                                            }
                                            FlexAlign::SpaceEvenly => {
                                                let mut space = (height - properties.children_occupied_space.height - layout_params.padding_top - layout_params.padding_bottom) / (children_len + 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_y = y + layout_params.padding_top + space;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y + child_layout_params.margin_top;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y += child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom + space;
                                                });
                                            }
                                        }
                                    }
                                    AxisStart::BottomStart | AxisStart::BottomEnd => {
                                        match justify_content {
                                            FlexAlign::Start => {
                                                let mut child_y = y + layout_params.padding_top + properties.children_occupied_space.height;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y - child_layout_params.height - child_layout_params.margin_bottom;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y -= child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom;
                                                });
                                            }
                                            FlexAlign::End => {
                                                let mut child_y = y + height - layout_params.padding_bottom;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y - child_layout_params.height - child_layout_params.margin_bottom;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y -= child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom;
                                                });
                                            }
                                            FlexAlign::Center => {
                                                let mut child_y = y + height - (height - properties.children_occupied_space.height) / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y - child_layout_params.height - child_layout_params.margin_bottom;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y -= child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom;
                                                });
                                            }
                                            FlexAlign::SpaceBetween => {
                                                let mut space = (height - properties.children_occupied_space.height - layout_params.padding_top - layout_params.padding_bottom) / (children_len - 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_y = y + height - layout_params.padding_bottom;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y - child_layout_params.height - child_layout_params.margin_bottom;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y -= child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom + space;
                                                });
                                            }
                                            FlexAlign::SpaceAround => {
                                                let mut space = (height - properties.children_occupied_space.height - layout_params.padding_top - layout_params.padding_bottom) / children_len as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_y = y + height - layout_params.padding_bottom - space / 2.0;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y - child_layout_params.height - child_layout_params.margin_bottom;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y -= child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom + space;
                                                });
                                            }
                                            FlexAlign::SpaceEvenly => {
                                                let mut space = (height - properties.children_occupied_space.height - layout_params.padding_top - layout_params.padding_bottom) / (children_len + 1) as f32;
                                                if space < 0.0 {
                                                    space = 0.0;
                                                }
                                                let mut child_y = y + height - layout_params.padding_bottom - space;
                                                item.get_children_mut().iter_mut().for_each(|child| {
                                                    let mut child_layout_params = child.get_layout_params().clone();
                                                    let y = child_y - child_layout_params.height - child_layout_params.margin_bottom;
                                                    let x = calculate_x(x, axis_start, align_items, &layout_params, &child_layout_params);
                                                    child.layout(x.physical_value(child_layout_params.width), y);
                                                    child_y -= child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom + space;
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            FlexWrap::Wrap => {
                                match axis_start {
                                    AxisStart::StartTop | AxisStart::StartBottom => {
                                        match justify_content {
                                            FlexAlign::Start => {
                                                let mut child_x = x + layout_params.padding_start;
                                                let mut child_y = y + layout_params.padding_top;
                                                let mut row_height = 0.0;
                                                let mut row_width = 0.0;
                                                let mut row_start_index = 0;
                                                item.get_children_mut().iter_mut().enumerate().for_each(|(index,child)|{
                                                    let child_layout_params = child.get_layout_params().clone();
                                                    if row_width + child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end > width - layout_params.padding_start - layout_params.padding_end {
                                                        if row_start_index != index {
                                                            let x = x + layout_params.padding_start;
                                                            let y = child_y + row_height + child_layout_params.margin_top;
                                                            child.layout(x.physical_value(child_layout_params.width), y);
                                                            child_x = x + layout_params.padding_start + child_layout_params.margin_start + child_layout_params.width + child_layout_params.margin_end;
                                                            child_y += row_height;
                                                            row_height = child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom;
                                                            row_width = child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                            row_start_index = index;
                                                        }
                                                        else {
                                                            let x = child_x + child_layout_params.margin_start;
                                                            let y = child_y + child_layout_params.margin_top;
                                                            child.layout(x.physical_value(child_layout_params.width), y);
                                                            child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                            row_height = row_height.max(child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom);
                                                            row_width += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                        }
                                                    } else {
                                                        let x = child_x + child_layout_params.margin_start;
                                                        let y = child_y + child_layout_params.margin_top;
                                                        child.layout(x.physical_value(child_layout_params.width), y);
                                                        child_x += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                        row_height = row_height.max(child_layout_params.height + child_layout_params.margin_top + child_layout_params.margin_bottom);
                                                        row_width += child_layout_params.width + child_layout_params.margin_start + child_layout_params.margin_end;
                                                    }
                                                });
                                            }
                                            _=>{}
                                        }
                                    }
                                    _=>{}
                                }
                            }
                        }
                    }
                })
        );
        item.set_children(children);

        FlexLayout {
            item,
            properties,
        }
    }

    pub fn unwrap(self) -> Item {
        self.item
    }
}

fn measure_child_stretch(is_height: bool, child: &Item, parent_layout_params: &LayoutParams, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) -> (MeasureMode, MeasureMode) {
    let layout_params = child.get_layout_params();
    let max_width = match width_measure_mode {
        MeasureMode::Exactly(width) => width,
        MeasureMode::AtMost(width) => width,
    } - layout_params.margin_start - layout_params.margin_end - parent_layout_params.padding_start - parent_layout_params.margin_end;
    let max_height = match height_measure_mode {
        MeasureMode::Exactly(height) => height,
        MeasureMode::AtMost(height) => height,
    } - layout_params.margin_top - layout_params.margin_bottom - parent_layout_params.padding_top - parent_layout_params.margin_bottom;

    let child_width = child.get_width().get();
    let child_height = child.get_height().get();

    let child_width_measure_mode =
        if is_height {
            match child_width {
                Size::Default => MeasureMode::AtMost(max_width),
                Size::Fill => MeasureMode::Exactly(max_width),
                Size::Fixed(width) => MeasureMode::Exactly(width),
                Size::Relative(scale) => MeasureMode::Exactly(max_width * scale),
            }
        } else {
            MeasureMode::Exactly(max_width)
        };

    let child_height_measure_mode =
        if is_height {
            MeasureMode::Exactly(max_height)
        } else {
            match child_height {
                Size::Default => MeasureMode::AtMost(max_height),
                Size::Fill => MeasureMode::Exactly(max_height),
                Size::Fixed(height) => MeasureMode::Exactly(height),
                Size::Relative(scale) => MeasureMode::Exactly(max_height * scale),
            }
        };

    (child_width_measure_mode, child_height_measure_mode)
}