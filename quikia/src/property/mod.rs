mod float_property;
mod size_property;
mod color_property;
mod bool_property;
mod item_property;
mod alignment_property;

pub use float_property::*;
pub use size_property::*;
pub use color_property::*;
pub use bool_property::*;
pub use item_property::*;
pub use alignment_property::*;

use std::sync::{Arc, Mutex};

pub struct ValueChangedListener{
    listener: Box<dyn FnMut()>,
    owner_id: usize,
}

impl ValueChangedListener{
    pub fn new(listener: Box<dyn FnMut()>, owner_id:usize) -> Self{
        Self{
            listener,
            owner_id
        }
    }

    pub fn new_without_id(listener: impl FnMut()+'static) -> Self{
        let listener = Box::new(listener);
        Self{
            listener,
            owner_id: 0
        }
    }

    pub fn owner_id(&self) -> usize{
        self.owner_id
    }

    pub fn call(&mut self){
        (self.listener)();
    }
}

pub trait Observable{
    fn clone(&self) -> Box<dyn Observable>;
    fn add_value_changed_listener(&self, listener: ValueChangedListener);
    fn remove_value_changed_listener(&self, owner_id: usize);
}

pub struct SharedProperty<T:'static+Clone>{
    value:Arc<Mutex<Property<T>>>,
}

impl<T:'static+Clone> SharedProperty<T>{
    pub fn from_generator(value_generator: Box<dyn Fn()->T>) -> Self{
        Self{
            value: Arc::new(Mutex::new(Property::from_generator(value_generator))),
        }
    }

    pub fn from_value(value: T) -> Self{
        Self{
            value: Arc::new(Mutex::new(Property::from_value(value))),
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Property<T>>{
        self.value.lock().unwrap()
    }

    pub fn value(&self) -> Arc<Mutex<Property<T>>>{
        Arc::clone(&self.value)
    }

    pub fn observe<O:'static+Observable>(&self, observable: &O){
        self.value.lock().unwrap().observe(observable);
    }

    pub fn add_value_changed_listener(&self, listener:ValueChangedListener){
        self.value.lock().unwrap().add_value_changed_listener(listener);
    }

    pub fn get(&self) -> T{
        self.value.lock().unwrap().get()
    }

    pub fn set<U:Into<T>>(&self, value:U){
        self.value.lock().unwrap().set(value);
    }
}

impl<T:'static+Clone> Observable for SharedProperty<T> {
    fn clone(&self) -> Box<dyn Observable> {
        Box::new(Clone::clone(self))
    }

    fn add_value_changed_listener(&self, listener: ValueChangedListener){
        self.value.lock().unwrap().add_value_changed_listener(listener);
    }

    fn remove_value_changed_listener(&self, owner_id: usize){
        self.value.lock().unwrap().remove_value_changed_listener(owner_id);
    }
}

impl<T:'static+Clone> Clone for SharedProperty<T>{
    fn clone(&self) -> Self {
        Self{
            value: Arc::clone(&self.value),
        }
    }
}

pub struct Property<T:'static+Clone>{
    value:Box<dyn Fn()->T>,
    value_changed_listeners: Arc<Mutex<Vec<ValueChangedListener>>>,
    observed_properties: Vec<Box<dyn Observable>>,
}

impl<T:'static+Clone> Property<T>{
    pub fn from_generator(value_generator: Box<dyn Fn()->T>) -> Self{
        Self{
            value: value_generator,
            value_changed_listeners: Arc::new(Mutex::new(Vec::new())),
            observed_properties: Vec::new(),
        }
    }

    pub fn from_value(value: T) -> Self{
        Self{
            value: Box::new(move || value.clone()),
            value_changed_listeners: Arc::new(Mutex::new(Vec::new())),
            observed_properties: Vec::new(),
        }
    }

    pub fn observe<O:'static+Observable>(&mut self, observable: &O){
        self.observed_properties.push(observable.clone());
        let owner_id = self as *const _ as usize;
        let value_changed_listeners = Arc::clone(&self.value_changed_listeners);
        observable.add_value_changed_listener(ValueChangedListener::new(Box::new(move || {
            value_changed_listeners.lock().unwrap().iter_mut().for_each(|listener|{
                listener.call()
            });
        }), owner_id));
    }

    pub fn get(&self) -> T{
        (self.value)()
    }

    pub fn set<U:Into<T>>(&mut self, value:U){
        let value = value.into();
        self.observed_properties.clear();
        self.value = Box::new(move || value.clone());
        self.notify_value_changed();
    }

    pub fn add_value_changed_listener(&mut self, listener:ValueChangedListener){
        self.value_changed_listeners.lock().unwrap().push(listener);
    }

    pub fn remove_value_changed_listener(&mut self, owner_id: usize){
        self.value_changed_listeners.lock().unwrap().retain(|listener|{
            listener.owner_id() != owner_id
        });
    }

    pub fn notify_value_changed(&mut self){
        self.value_changed_listeners.lock().unwrap().iter_mut().for_each(|listener|{
            listener.call()
        });
    }
}