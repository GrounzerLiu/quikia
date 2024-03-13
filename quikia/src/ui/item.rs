use std::collections::HashMap;
use skia_safe::Canvas;
use skia_safe::gpu::SyncCpu::No;
use winit::event::{DeviceId, KeyEvent, MouseButton};
use crate::app::{SharedApp};
use crate::impl_item_property;
use crate::ui::{AdditionalProperty, ButtonState, Gravity, ImeAction, ItemEvent, ItemPath, LayoutDirection, LayoutParams, MeasureMode, PointerAction};
use crate::property::{BoolProperty, FloatProperty, Gettable, GravityProperty, ItemProperty, Observable, Observer, SharedProperty, Size, SizeProperty};

pub struct Item {
    app: SharedApp,
    tag: String,
    children: Vec<Item>,
    active: BoolProperty,
    width: SizeProperty,
    height: SizeProperty,
    layout_direction: SharedProperty<LayoutDirection>,
    horizontal_gravity: GravityProperty,
    vertical_gravity: GravityProperty,
    focusable: BoolProperty,
    focused: BoolProperty,
    focusable_when_clicked: BoolProperty,
    min_width: FloatProperty,
    min_height: FloatProperty,
    max_width: FloatProperty,
    max_height: FloatProperty,
    padding_start: FloatProperty,
    padding_top: FloatProperty,
    padding_end: FloatProperty,
    padding_bottom: FloatProperty,
    margin_start: FloatProperty,
    margin_top: FloatProperty,
    margin_end: FloatProperty,
    margin_bottom: FloatProperty,
    layout_params: LayoutParams,
    background: ItemProperty,
    foreground: ItemProperty,
    enable_clipping: BoolProperty,
    additional_properties: HashMap<String, AdditionalProperty>,
    on_click: Option<Box<dyn Fn()>>,
    on_blur: Option<Box<dyn Fn()>>,
    on_focus: Option<Box<dyn Fn()>>,
    on_cursor_entered: Box<dyn Fn()>,
    on_cursor_exited: Box<dyn Fn()>,
    on_draw: Box<dyn Fn(&mut Item, &Canvas)>,
    on_measure: Box<dyn Fn(&mut Item, MeasureMode, MeasureMode)>,
    on_layout: Box<dyn Fn(&mut Item, f32, f32)>,
    on_mouse_input: Box<dyn Fn(&mut Item, DeviceId, ButtonState, MouseButton, f32, f32) -> bool>,
    
    on_cursor_moved: Box<dyn Fn(&mut Item, f32, f32) -> bool>,
    on_cursor_entered_event: Box<dyn Fn(&mut Item)>,
    on_cursor_exited_event: Box<dyn Fn(&mut Item)>,
    is_cursor_inside: bool,
    
    on_pointer_input: Box<dyn Fn(&mut Item, PointerAction) -> bool>,
    on_ime_input: Box<dyn Fn(&mut Item, ImeAction) -> bool>,
    on_keyboard_input: Box<dyn Fn(&mut Item, DeviceId, KeyEvent, bool) -> bool>,
}


impl_item_property!(Item, active, get_active, BoolProperty);
impl_item_property!(Item, width, get_width, SizeProperty);
impl_item_property!(Item, height, get_height, SizeProperty);
impl_item_property!(Item, layout_direction, get_layout_direction, SharedProperty<LayoutDirection>);
impl_item_property!(Item, horizontal_gravity, get_horizontal_gravity, GravityProperty);
impl_item_property!(Item, vertical_gravity, get_vertical_gravity, GravityProperty);
impl_item_property!(Item, focusable, get_focusable, BoolProperty);
impl_item_property!(Item, focusable_when_clicked, get_focusable_when_clicked, BoolProperty);
impl_item_property!(Item, min_width, get_min_width, FloatProperty);
impl_item_property!(Item, min_height, get_min_height, FloatProperty);
impl_item_property!(Item, max_width, get_max_width, FloatProperty);
impl_item_property!(Item, max_height, get_max_height, FloatProperty);
impl_item_property!(Item, padding_start, get_padding_start, FloatProperty);
impl_item_property!(Item, padding_top, get_padding_top, FloatProperty);
impl_item_property!(Item, padding_end, get_padding_end, FloatProperty);
impl_item_property!(Item, padding_bottom, get_padding_bottom, FloatProperty);
impl_item_property!(Item, margin_start, get_margin_start, FloatProperty);
impl_item_property!(Item, margin_top, get_margin_top, FloatProperty);
impl_item_property!(Item, margin_end, get_margin_end, FloatProperty);
impl_item_property!(Item, margin_bottom, get_margin_bottom, FloatProperty);
impl_item_property!(Item, background, get_background, ItemProperty);
impl_item_property!(Item, foreground, get_foreground, ItemProperty);
impl_item_property!(Item, enable_clipping, get_enable_clipping, BoolProperty);


impl Item {
    pub fn new(app: SharedApp, item_events: ItemEvent) -> Self {
        let layout_direction = app.layout_direction();
        Item {
            app,
            tag: String::new(),
            children: Vec::with_capacity(1),
            active: true.into(),
            width: Size::Default.into(),
            height: Size::Default.into(),
            layout_direction: layout_direction.into(),
            horizontal_gravity: Gravity::Start.into(),
            vertical_gravity: Gravity::Start.into(),
            focusable: true.into(),
            focused: false.into(),
            focusable_when_clicked: true.into(),
            is_cursor_inside: false,
            min_width: 0.into(),
            min_height: 0.into(),
            max_width: FloatProperty::from_value(f32::MAX),
            max_height: FloatProperty::from_value(f32::MAX),
            padding_start: 0.into(),
            padding_top: 0.into(),
            padding_end: 0.into(),
            padding_bottom: 0.into(),
            margin_start: 0.into(),
            margin_top: 0.into(),
            margin_end: 0.into(),
            margin_bottom: 0.into(),
            layout_params: LayoutParams::default(),
            background: None.into(),
            foreground: None.into(),
            enable_clipping: false.into(),
            additional_properties: HashMap::new(),
            on_click: None,
            on_blur: None,
            on_focus: None,
            on_cursor_entered: Box::new(|| {}),
            on_cursor_exited: Box::new(|| {}),
            on_draw: item_events.on_draw,
            on_measure: item_events.on_measure,
            on_layout: item_events.on_layout,
            on_mouse_input: item_events.on_mouse_input,
            on_cursor_moved: item_events.on_cursor_moved,
            on_cursor_entered_event: item_events.on_cursor_entered,
            on_cursor_exited_event: item_events.on_cursor_exited,
            on_pointer_input: item_events.on_pointer_input,
            on_ime_input: item_events.on_ime_input,
            on_keyboard_input: item_events.on_keyboard_input,
        }
    }

    pub fn get_app(&self) -> SharedApp {
        self.app.clone()
    }

    pub fn get_id(&self) -> usize {
        self as *const Item as usize
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn set_children(&mut self, children: Vec<Item>) {
        self.children = children;
    }

    pub fn get_children(&self) -> &Vec<Item> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Item> {
        &mut self.children
    }

    pub fn set_additional_property(&mut self, key: impl Into<String>, value: impl Into<AdditionalProperty>) {
        self.additional_properties.insert(key.into(), value.into());
    }
    
    pub fn get_additional_property(&self, key: impl Into<String>) -> Option<&AdditionalProperty> {
        self.additional_properties.get(&key.into())
    }

    pub fn focus(&mut self) {
        self.focused.set_value(true);
    }

    pub fn blur(&mut self) {
        self.focused.set_value(false);
    }

    pub fn draw(&mut self, canvas: &Canvas) {
        let layout_params = self.get_layout_params();
        let content_width = self.app.content_width();
        let content_height = self.app.content_height();
        if layout_params.x + layout_params.width < 0.0 || layout_params.x > content_width || layout_params.y + layout_params.height < 0.0 || layout_params.y > content_height {
            return;
        }
        unsafe {
            let s = self as *const Item;
            let on_draw = &(*s).on_draw;
            on_draw(self, canvas);
        }
    }

    pub fn measure(&mut self, width_measure_mode: MeasureMode, height_measure_mode: MeasureMode) {
        unsafe {
            let s = self as *const Item;
            let on_measure = &(*s).on_measure;
            on_measure(self, width_measure_mode, height_measure_mode);
        }
    }

    pub fn layout(&mut self, x: f32, y: f32) {
        unsafe {
            let s = self as *const Item;
            let on_layout = &(*s).on_layout;
            on_layout(self, x, y);
        }
    }

    pub fn mouse_input(&mut self, device_id: DeviceId, state: ButtonState, button: MouseButton, x: f32, y: f32) -> bool
    {
        unsafe {
            let s = self as *const Item;
            let on_mouse_input = &(*s).on_mouse_input;
            on_mouse_input(self, device_id, state, button, x, y)
        }
    }
    
    pub fn cursor_moved(&mut self, x: f32, y: f32) -> bool
    {
        if self.get_layout_params().contains(x, y){
            if !self.is_cursor_inside {
                self.is_cursor_inside = true;
                unsafe {
                    let s = self as *const Item;
                    let on_cursor_entered_event = &(*s).on_cursor_entered_event;
                    on_cursor_entered_event(self);
                    let on_cursor_entered = &(*s).on_cursor_entered;
                    on_cursor_entered();
                }
            }
        } else {
            if self.is_cursor_inside {
                self.is_cursor_inside = false;
                unsafe {
                    let s = self as *const Item;
                    let on_cursor_exited_event = &(*s).on_cursor_exited_event;
                    on_cursor_exited_event(self);
                    let on_cursor_exited = &(*s).on_cursor_exited;
                    on_cursor_exited();
                }
            }
        }
        unsafe {
            let s = self as *const Item;
            let on_cursor_moved = &(*s).on_cursor_moved;
            let handled = on_cursor_moved(self, x, y);
            if !handled {
                for child in self.get_children_mut() {
                    if child.cursor_moved(x, y) {
                        return true;
                    }
                }
            }
            handled
        }
    }

    pub fn pointer_input(&mut self, action: PointerAction) -> bool
    {
        unsafe {
            let s = self as *const Item;
            let on_pointer_input = &(*s).on_pointer_input;
            on_pointer_input(self, action)
        }
    }

    pub fn ime_input(&mut self, action: ImeAction) -> bool {
        unsafe {
            let s = self as *const Item;
            let on_ime_input = &(*s).on_ime_input;
            let handled = on_ime_input(self, action.clone());
            if !handled {
                for child in self.get_children_mut() {
                    if child.ime_input(action.clone()) {
                        return true;
                    }
                }
            }
            handled
        }
    }

    pub fn keyboard_input(&mut self, device_id: DeviceId, event: KeyEvent, is_synthetic: bool) -> bool  {
        unsafe {
            let s = self as *const Item;
            let on_keyboard_input = &(*s).on_keyboard_input;
            let handled = on_keyboard_input(self, device_id, event.clone(), is_synthetic);
            if !handled {
                for child in self.get_children_mut() {
                    if child.keyboard_input(device_id, event.clone(), is_synthetic) {
                        return true;
                    }
                }
            }
            handled
        }
    }

    pub fn get_layout_params(&self) -> &LayoutParams {
        &self.layout_params
    }

    pub fn get_layout_params_mut(&mut self) -> &mut LayoutParams {
        &mut self.layout_params
    }

    pub fn set_layout_params(&mut self, layout_params: &LayoutParams) {
        self.layout_params = layout_params.clone();
    }

    pub fn on_click(mut self, on_click: impl Fn() + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }

    pub fn get_on_click(&self) -> Option<&Box<dyn Fn()>> {
        self.on_click.as_ref()
    }

    pub fn on_blur(mut self, on_blur: impl Fn() + 'static) -> Self {
        self.on_blur = Some(Box::new(on_blur));
        self
    }

    pub fn invoke_on_blur(&mut self) {
        if let Some(on_blur) = &self.on_blur {
            on_blur();
        }
    }

    pub fn on_focus(mut self, on_focus: impl Fn() + 'static) -> Self {
        self.on_focus = Some(Box::new(on_focus));
        self
    }

    pub fn invoke_on_focus(&mut self) {
        if let Some(on_focus) = &self.on_focus {
            on_focus();
        }
    }

    pub fn on_cursor_entered(mut self, on_cursor_entered: impl Fn() + 'static) -> Self {
        self.on_cursor_entered = Box::new(on_cursor_entered);
        self
    }

    pub fn on_cursor_exited(mut self, on_cursor_exited: impl Fn() + 'static) -> Self {
        self.on_cursor_exited = Box::new(on_cursor_exited);
        self
    }

    pub fn gravity(mut self, gravity:impl Into<(GravityProperty,GravityProperty)>) -> Self {
        let (horizontal_gravity,vertical_gravity) = gravity.into();
        self.horizontal_gravity = horizontal_gravity;
        self.vertical_gravity = vertical_gravity;
        {
            let app = self.app.clone();
            self.horizontal_gravity.add_observer(
                Observer::new_without_id(move||{
                    app.lock().unwrap().request_layout();
                })
            );
        }

        {
            let app = self.app.clone();
            self.vertical_gravity.add_observer(
                Observer::new_without_id(move||{
                    app.lock().unwrap().request_layout();
                })
            );
        }

        self
    }

    pub fn focused(mut self, focused: impl Into<BoolProperty>) -> Self {
        self.focused = focused.into();
        let app = self.app.clone();
        let id = self.get_id();
        let focused_clone = self.focused.clone();
        self.focused.add_observer(
            Observer::new_without_id(move||{
                if focused_clone.get() {
                    app.lock().unwrap().request_focus(id)
                } else {
                    app.lock().unwrap().request_focus(0)
                }
                app.lock().unwrap().request_layout();
            })
        );
        self
    }

    pub fn get_focused(&self) -> BoolProperty {
        self.focused.clone()
    }
}

impl Into<(GravityProperty,GravityProperty)> for &SharedProperty<Gravity> {
    fn into(self) -> (GravityProperty, GravityProperty) {
        let horizontal_gravity = self.clone();
        let vertical_gravity = self.clone();
        (horizontal_gravity.into(), vertical_gravity.into())
    }
}

macro_rules! impl_into_additional_property {
    ($t:ty, $variant:ident) => {
        impl From<$t> for AdditionalProperty {
            fn from(value: $t) -> Self {
                AdditionalProperty::$variant(value)
            }
        }
    };
}

impl_into_additional_property!(i8, I8);
impl_into_additional_property!(i16, I16);
impl_into_additional_property!(i32, I32);
impl_into_additional_property!(i64, I64);
impl_into_additional_property!(u8, U8);
impl_into_additional_property!(u16, U16);
impl_into_additional_property!(f32, F32);
impl_into_additional_property!(f64, F64);
impl_into_additional_property!(bool, Bool);
impl_into_additional_property!(String, String);