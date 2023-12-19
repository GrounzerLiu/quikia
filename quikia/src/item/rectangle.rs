use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use skia_safe::{Canvas, Color, Paint, Rect};
use crate::item::item::Item;
use crate::item::{ItemEvent, LayoutDirection, MeasureMode};
use crate::property::{BoolProperty, ColorProperty, FloatProperty, Gettable, Observable, Observer};

struct RectangleProperties {
    color: ColorProperty,
    use_smooth_corners: BoolProperty,
    radius_start_top: FloatProperty,
    radius_end_top: FloatProperty,
    radius_start_bottom: FloatProperty,
    radius_end_bottom: FloatProperty,
}

pub struct Rectangle {
    item: Item,
    properties: Arc<Mutex<RectangleProperties>>,
}

impl Rectangle {
    pub fn new() -> Self {
        let properties = Arc::new(Mutex::new(RectangleProperties {
            color: ColorProperty::from_value(Color::TRANSPARENT),
            use_smooth_corners: BoolProperty::from_value(false),
            radius_start_top: FloatProperty::from_value(0.0),
            radius_end_top: FloatProperty::from_value(0.0),
            radius_start_bottom: FloatProperty::from_value(0.0),
            radius_end_bottom: FloatProperty::from_value(0.0),
        }));

        let mut item = Item::new(
            ItemEvent::default()

                .set_on_draw({
                    let properties = properties.clone();
                    move |item, canvas| {
                        if let Some(background) = item.get_background().lock().as_mut() {
                            background.draw(canvas);
                        }

                        let layout_params = item.get_layout_params();

                        let layout_direction = item.get_layout_direction().get();

                        let x = match layout_direction {
                            LayoutDirection::LeftToRight => {
                                layout_params.x + layout_params.padding_start
                            }
                            LayoutDirection::RightToLeft => {
                                layout_params.x + layout_params.padding_end
                            }
                        };

                        let y = layout_params.y + layout_params.padding_top;

                        let width = layout_params.width - layout_params.padding_start - layout_params.padding_end;
                        let height = layout_params.height - layout_params.padding_top - layout_params.padding_bottom;

                        let rect = Rect::from_xywh(x, y, width, height);

                        let properties = properties.lock().unwrap();

                        let color = *layout_params.get_color_param("color").unwrap_or(&(properties.color.get()));
                        let use_smooth_corners = properties.use_smooth_corners.get();

                        let radius_left_top = match layout_direction {
                            LayoutDirection::LeftToRight => {
                                properties.radius_start_top.get()
                            }
                            LayoutDirection::RightToLeft => {
                                properties.radius_end_top.get()
                            }
                        };

                        let radius_right_top = match layout_direction {
                            LayoutDirection::LeftToRight => {
                                properties.radius_end_top.get()
                            }
                            LayoutDirection::RightToLeft => {
                                properties.radius_start_top.get()
                            }
                        };

                        let radius_right_bottom = match layout_direction {
                            LayoutDirection::LeftToRight => {
                                properties.radius_end_bottom.get()
                            }
                            LayoutDirection::RightToLeft => {
                                properties.radius_start_bottom.get()
                            }
                        };

                        let radius_left_bottom = match layout_direction {
                            LayoutDirection::LeftToRight => {
                                properties.radius_start_bottom.get()
                            }
                            LayoutDirection::RightToLeft => {
                                properties.radius_end_bottom.get()
                            }
                        };

                        draw_round_rect(canvas, use_smooth_corners, rect, radius_left_top, radius_right_top, radius_right_bottom, radius_left_bottom, &Paint::default().set_anti_alias(true).set_color(color));

                        if let Some(foreground) = item.get_foreground().lock().as_mut() {
                            foreground.draw(canvas);
                        }
                    }
                })

                .set_on_measure({
                    let properties = properties.clone();
                    move|item, width_measure_mode, height_measure_mode| {
                        let mut layout_params = item.get_layout_params().clone();
                        layout_params.init_from_item(item);

                        layout_params.set_color_param("color", properties.lock().unwrap().color.get());

                        match width_measure_mode {
                            MeasureMode::Exactly(width) => {
                                layout_params.width = width + layout_params.padding_start + layout_params.padding_end;
                            }
                            MeasureMode::AtMost(_) => {
                                layout_params.width = layout_params.padding_start + layout_params.padding_end;
                            }
                        }
                        layout_params.width = layout_params.width.max(item.get_min_width().get()).min(item.get_max_width().get());
                        match height_measure_mode {
                            MeasureMode::Exactly(height) => {
                                layout_params.height = height + layout_params.padding_top + layout_params.padding_bottom;
                            }
                            MeasureMode::AtMost(_) => {
                                layout_params.height = layout_params.padding_top + layout_params.padding_bottom;
                            }
                        }

                        layout_params.height = layout_params.height.max(item.get_min_height().get()).min(item.get_max_height().get());

                        if let Some(background) = item.get_background().lock().as_mut() {
                            background.measure(
                                MeasureMode::Exactly(layout_params.width),
                                MeasureMode::Exactly(layout_params.height),
                            );
                        }

                        if let Some(foreground) = item.get_foreground().lock().as_mut() {
                            foreground.measure(
                                MeasureMode::Exactly(layout_params.width),
                                MeasureMode::Exactly(layout_params.height),
                            );
                        }

                        item.set_layout_params(&layout_params);
                    }
                })

                .set_on_layout(
                    |item, x, y| {
                        let mut layout_params = item.get_layout_params().clone();
                        layout_params.x = x;
                        layout_params.y = y;
                        item.set_layout_params(&layout_params);
                        if let Some(background) = item.get_background().lock().as_mut() {
                            background.layout(layout_params.width, layout_params.height);
                        }
                        if let Some(foreground) = item.get_foreground().lock().as_mut() {
                            foreground.layout(layout_params.width, layout_params.height);
                        }
                    }
                )
        );

        Rectangle {
            item,
            properties,
        }
    }

    pub fn color(self, color: impl Into<ColorProperty>) -> Self {
        let color = color.into();
        let app = self.item.get_app();
        color.add_observer(
            Observer::new_without_id(
                move || {
                    app.request_redraw();
                }
            )
        );
        self.properties.lock().unwrap().color = color;
        self
    }

    pub fn radius_start_top(self, radius: impl Into<FloatProperty>) -> Self {
        let radius = radius.into();
        let app = self.item.get_app();
        radius.add_observer(
            Observer::new_without_id(
                move || {
                    app.request_redraw();
                }
            )
        );
        self.properties.lock().unwrap().radius_start_top = radius;
        self
    }

    pub fn radius_end_top(self, radius: impl Into<FloatProperty>) -> Self {
        let radius = radius.into();
        let app = self.item.get_app();
        radius.add_observer(
            Observer::new_without_id(
                move || {
                    app.request_redraw()
                }
            )
        );
        self.properties.lock().unwrap().radius_end_top = radius;
        self
    }

    pub fn radius_start_bottom(self, radius: impl Into<FloatProperty>) -> Self {
        let radius = radius.into();
        let app = self.item.get_app();
        radius.add_observer(
            Observer::new_without_id(
                move || {
                    app.request_redraw();
                }
            )
        );
        self.properties.lock().unwrap().radius_start_bottom = radius;
        self
    }

    pub fn radius_end_bottom(self, radius: impl Into<FloatProperty>) -> Self {
        let radius = radius.into();
        let app = self.item.get_app();
        radius.add_observer(
            Observer::new_without_id(
                move || {
                    app.request_redraw();
                }
            )
        );
        self.properties.lock().unwrap().radius_end_bottom = radius;
        self
    }

    pub fn radius(self, radius: impl Into<FloatProperty>) -> Self {
        let radius = radius.into();
        let app = self.item.get_app();
        radius.add_observer(
            Observer::new_without_id(
                move || {
                    app.request_redraw();
                }
            )
        );
        let mut properties = self.properties.lock().unwrap();
        properties.radius_start_top = radius.clone();
        properties.radius_end_top = radius.clone();
        properties.radius_start_bottom = radius.clone();
        properties.radius_end_bottom = radius;
        drop(properties);
        self
    }

    pub fn use_smooth_corners(self, use_smooth_corners: impl Into<BoolProperty>) -> Self {
        let use_smooth_corners = use_smooth_corners.into();
        let app = self.item.get_app();
        use_smooth_corners.add_observer(
            Observer::new_without_id(
                move || {
                    app.request_redraw();
                }
            )
        );
        self.properties.lock().unwrap().use_smooth_corners = use_smooth_corners;
        self
    }

    pub fn unwrap(self) -> Item {
        self.item
    }
}

fn draw_round_rect(canvas: &Canvas, smooth: bool, rect: Rect, radius_left_top: f32, radius_right_top: f32, radius_right_bottom: f32, radius_left_bottom: f32, paint: &Paint) {
    let radius_left_top = radius_left_top.clamp(0.0, rect.width() / 2.0);
    let radius_right_top = radius_right_top.clamp(0.0, rect.width() / 2.0);
    let radius_right_bottom = radius_right_bottom.clamp(0.0, rect.width() / 2.0);
    let radius_left_bottom = radius_left_bottom.clamp(0.0, rect.width() / 2.0);

    let mut path = skia_safe::Path::new();

    if smooth {
        path.move_to((rect.left + radius_left_top, rect.top));
        path.line_to((rect.right - radius_right_top, rect.top));
        path.quad_to((rect.right, rect.top), (rect.right, rect.top + radius_right_top));
        path.line_to((rect.right, rect.bottom - radius_right_bottom));
        path.quad_to((rect.right, rect.bottom), (rect.right - radius_right_bottom, rect.bottom));
        path.line_to((rect.left + radius_left_bottom, rect.bottom));
        path.quad_to((rect.left, rect.bottom), (rect.left, rect.bottom - radius_left_bottom));
        path.line_to((rect.left, rect.top + radius_left_top));
        path.quad_to((rect.left, rect.top), (rect.left + radius_left_top, rect.top));
        path.close();
    } else {
        path.move_to((rect.left + radius_left_top, rect.top));
        path.arc_to(
            Rect::from_xywh(rect.left, rect.top, radius_left_top * 2.0, radius_left_top * 2.0),
            180.0,
            90.0,
            false,
        );
        path.line_to((rect.right - radius_right_top, rect.top));
        path.arc_to(
            Rect::from_xywh(
                rect.right - radius_right_top * 2.0,
                rect.top,
                radius_right_top * 2.0,
                radius_right_top * 2.0,
            ),
            270.0,
            90.0,
            false,
        );
        path.line_to((rect.right, rect.bottom - radius_right_bottom));
        path.arc_to(
            Rect::from_xywh(
                rect.right - radius_right_bottom * 2.0,
                rect.bottom - radius_right_bottom * 2.0,
                radius_right_bottom * 2.0,
                radius_right_bottom * 2.0,
            ),
            0.0,
            90.0,
            false,
        );
        path.line_to((rect.left + radius_left_bottom, rect.bottom));
        path.arc_to(
            Rect::from_xywh(
                rect.left,
                rect.bottom - radius_left_bottom * 2.0,
                radius_left_bottom * 2.0,
                radius_left_bottom * 2.0,
            ),
            90.0,
            90.0,
            false,
        );

        path.line_to((rect.left, rect.top + radius_left_top));
        path.close();
    }


    canvas.draw_path(&path, paint);
}