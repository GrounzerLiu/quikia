use std::collections::{HashMap, LinkedList};
use std::time::{Duration, Instant};
use material_color_utilities::{blend_cam16ucs, blend_hct_hue};
use skia_safe::Color;
use crate::app::ANIMATIONS;
use crate::item::{Item, LayoutParams};

pub(crate) struct LayoutTransition {
    pub action: Box<dyn FnMut() + 'static>,
}

unsafe impl Send for LayoutTransition {}

pub struct  AnimationDuration {
    pub duration: Duration,
}

impl AnimationDuration {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
        }
    }
}

impl From<Duration> for AnimationDuration {
    fn from(duration: Duration) -> Self {
        Self {
            duration,
        }
    }
}

impl From<u32> for AnimationDuration {
    fn from(duration: u32) -> Self {
        Self {
            duration: Duration::from_millis(duration as u64),
        }
    }
}

impl From<u64> for AnimationDuration {
    fn from(duration: u64) -> Self {
        Self {
            duration: Duration::from_millis(duration),
        }
    }
}

pub struct Animation {
    start_time: Instant,
    duration: Duration,
    pub(crate) layout_transition: LayoutTransition,
    pub(crate) from: Option<HashMap<usize, LayoutParams>>,
    pub(crate) to: Option<HashMap<usize, LayoutParams>>,
    is_running: bool,
}

impl Animation {
    pub fn new(layout_change: impl FnMut() + 'static) -> Self {
        let layout_transition = LayoutTransition {
            action: Box::new(layout_change),
        };
        Self {
            start_time: Instant::now(),
            duration: Duration::from_millis(2000),
            layout_transition,
            from: None,
            to: None,
            is_running: false,
        }
    }

    pub fn start(mut self) {
        self.start_time = Instant::now();
        self.is_running = true;
        ANIMATIONS.lock().unwrap().push(self)
    }

    fn color_to_argb(color: &Color) -> u32 {
        let mut color = color.clone();
        let a = color.a();
        let r = color.r();
        let g = color.g();
        let b = color.b();
        return (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
    }

    pub fn update(&mut self, item: &mut Item, now: Instant) {
        let elapsed = now - self.start_time;
        let mut progress = (elapsed.as_secs_f64() / self.duration.as_secs_f64()) as f32;
        let mut is_running = true;
        if progress >= 1.0 {
            progress = 1.0;
            is_running = false;
        }
        else if progress < 0.0 {
            return;
        }
        let from_map = self.from.as_mut().unwrap();
        let to_map = self.to.as_mut().unwrap();
        let mut stack = LinkedList::new();
        stack.push_back(item);
        while let Some(item) = stack.pop_back() {
            if let Some(from)=from_map.get(&item.get_id()){
                let to = to_map.get(&item.get_id()).unwrap();
                if from != to {
                    let mut layout_params = item.get_layout_params().clone();
                    layout_params.x = from.x + (to.x - from.x) * progress;
                    layout_params.y = from.y + (to.y - from.y) * progress;
                    layout_params.width = from.width + (to.width - from.width) * progress;
                    layout_params.height = from.height + (to.height - from.height) * progress;
                    layout_params.margin_start = from.margin_start + (to.margin_start - from.margin_start) * progress;
                    layout_params.margin_top = from.margin_top + (to.margin_top - from.margin_top) * progress;
                    layout_params.margin_end = from.margin_end + (to.margin_end - from.margin_end) * progress;
                    layout_params.margin_bottom = from.margin_bottom + (to.margin_bottom - from.margin_bottom) * progress;
                    layout_params.padding_start = from.padding_start + (to.padding_start - from.padding_start) * progress;
                    layout_params.padding_top = from.padding_top + (to.padding_top - from.padding_top) * progress;
                    layout_params.padding_end = from.padding_end + (to.padding_end - from.padding_end) * progress;
                    layout_params.padding_bottom = from.padding_bottom + (to.padding_bottom - from.padding_bottom) * progress;
                    from.float_params.iter().for_each(|(key, value)| {
                        let to_value = to.float_params.get(key).unwrap();
                        layout_params.float_params.insert(key.clone(), value + (to_value - value) * progress);
                    });
                    from.color_params.iter().for_each(|(key, value)| {
                        let to_value = to.color_params.get(key).unwrap();
                        let from_argb = Self::color_to_argb(value);
                        let to_argb = Self::color_to_argb(to_value);
                        let argb = blend_cam16ucs(from_argb, to_argb, progress as f64);
                        layout_params.color_params.insert(key.clone(), Color::from(argb));
                    });
                    item.set_layout_params(&layout_params);
                }
                else {
                    from_map.remove(&item.get_id());
                    to_map.remove(&item.get_id());
                }
            }

            stack.extend(item.get_children_mut().iter_mut());
        }
        if !is_running {
            self.is_running = false;
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn duration(mut self, duration: impl Into<AnimationDuration>) -> Self {
        self.duration = duration.into().duration;
        self
    }

    pub(crate) fn item_to_layout_params(item: &Item) -> HashMap<usize, LayoutParams> {
        let mut map = HashMap::new();
        let mut stack = LinkedList::new();
        stack.push_back(item);
        while let Some(item) = stack.pop_back() {
            map.insert(item.get_id(), item.get_layout_params().clone());
            stack.extend(item.get_children().iter());
        }
        map
    }
}
