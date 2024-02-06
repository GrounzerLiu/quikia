use skia_safe::Canvas;
use skia_safe::gpu::SyncCpu::No;
use winit::event::{DeviceId, MouseButton};
use crate::app::{current_app, SharedApp};
use crate::impl_item_property;
use crate::item::{ButtonState, Gravity, ImeAction, ItemEvent, ItemPath, LayoutDirection, LayoutParams, MeasureMode, PointerAction};
use crate::property::{BoolProperty, FloatProperty, GravityProperty, ItemProperty, Observable, Observer, SharedProperty, Size, SizeProperty};

pub struct Item {
    app: SharedApp,
    path: ItemPath,
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
    is_cursor_inside: bool,
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
    on_click: Option<Box<dyn Fn()>>,
    on_draw: Box<dyn Fn(&mut Item, &Canvas)>,
    on_measure: Box<dyn Fn(&mut Item, MeasureMode, MeasureMode)>,
    on_layout: Box<dyn Fn(&mut Item, f32, f32)>,
    on_mouse_input: Box<dyn Fn(&mut Item, DeviceId, ButtonState, MouseButton, f32, f32) -> bool>,
    on_pointer_input: Box<dyn Fn(&mut Item, PointerAction) -> bool>,
    on_ime_input: Box<dyn Fn(&mut Item, ImeAction)>,
}


impl_item_property!(Item, active, get_active, BoolProperty);
impl_item_property!(Item, width, get_width, SizeProperty);
impl_item_property!(Item, height, get_height, SizeProperty);
impl_item_property!(Item, layout_direction, get_layout_direction, SharedProperty<LayoutDirection>);
impl_item_property!(Item, horizontal_gravity, get_horizontal_gravity, GravityProperty);
impl_item_property!(Item, vertical_gravity, get_vertical_gravity, GravityProperty);
impl_item_property!(Item, focusable, get_focusable, BoolProperty);
impl_item_property!(Item, focused, get_focused, BoolProperty);
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
    pub fn new(item_events: ItemEvent) -> Self {
        let app = current_app().unwrap();
        let id = app.lock().unwrap().new_id();
        let layout_direction = app.layout_direction();
        Item {
            app,
            path: ItemPath::new(),
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
            on_click: None,
            on_draw: item_events.on_draw,
            on_measure: item_events.on_measure,
            on_layout: item_events.on_layout,
            on_mouse_input: item_events.on_mouse_input,
            on_pointer_input: item_events.on_pointer_input,
            on_ime_input: item_events.on_ime_input,
        }
    }

    pub fn get_app(&self) -> SharedApp {
        self.app.clone()
    }

    pub fn get_id(&self) -> usize {
        self as *const Item as usize
    }

    pub(crate) fn get_item_path(&self) -> &ItemPath {
        &self.path
    }

    pub(crate) fn set_item_path(&mut self, item_path: ItemPath) {
        self.path = item_path;
    }

    pub fn set_children(&mut self, children: Vec<Item>) {
        self.children = children;
        let self_path = &self.path;
        self.children.iter_mut().enumerate().for_each(|(i, child)| {
            let mut child_path = self_path.clone();
            child_path.push_back(i);
            child.set_item_path(child_path);
        });
    }

    pub fn get_children(&self) -> &Vec<Item> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Item> {
        &mut self.children
    }


    pub fn draw(&mut self, canvas: &Canvas) {
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

    pub fn layout(&mut self, width: f32, height: f32) {
        unsafe {
            let s = self as *const Item;
            let on_layout = &(*s).on_layout;
            on_layout(self, width, height);
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

    pub fn pointer_input(&mut self, action: PointerAction) -> bool
    {
        unsafe {
            let s = self as *const Item;
            let on_pointer_input = &(*s).on_pointer_input;
            on_pointer_input(self, action)
        }
    }

    pub fn ime_input(&mut self, action: ImeAction) {
        unsafe {
            let s = self as *const Item;
            let on_ime_input = &(*s).on_ime_input;
            on_ime_input(self, action)
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
}

impl Into<(GravityProperty,GravityProperty)> for &SharedProperty<Gravity> {
    fn into(self) -> (GravityProperty, GravityProperty) {
        let horizontal_gravity = self.clone();
        let vertical_gravity = self.clone();
        (horizontal_gravity.into(), vertical_gravity.into())
    }
}
