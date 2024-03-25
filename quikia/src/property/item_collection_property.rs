use std::slice::Iter;
use std::rc::Rc;
use std::sync::Mutex;
use crate::property::{Observable, Observer, SharedProperty};
use crate::ui::Item;

#[macro_export]
macro_rules! children {
    () => (
        {
            let item_collection=$crate::property::ItemCollection::new();
            $crate::property::ItemCollectionProperty::from_value(item_collection)
        }
    );
    ($($x:expr),+ $(,)?) => (
        {
            let mut children = $crate::property::ItemCollection::new();
            $(
                children.add($x);
            )+
            $crate::property::ItemCollectionProperty::from_value(children)
        }
    );
}

pub struct ItemCollection{
    items: Vec<Item>,
    observers: Rc<Mutex<Vec<Observer>>>
}

impl Observable for ItemCollection{
    fn add_observer(&self, listener: Observer){
        self.observers.lock().unwrap().push(listener);
    }
    fn remove_observer(&self, owner_id: usize){
        let mut observers = self.observers.lock().unwrap();
        observers.retain(|observer| observer.owner_id != owner_id);
    }
    fn clear_observers(&self){
        self.observers.lock().unwrap().clear();
    }
    fn notify(&self){
        let mut observers = self.observers.lock().unwrap();
        for observer in observers.iter_mut(){
            observer.notify();
        }
    }
}

impl ItemCollection{
    pub fn new() -> Self{
        Self{
            items: Vec::new(),
            observers: Rc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn add(&mut self, item: Item){
        self.items.push(item);
        self.notify();
    }

    pub fn remove(&mut self, index: usize){
        self.items.remove(index);
        self.notify();
    }

    pub fn get(&self, index: usize) -> Option<&Item>{
        self.items.get(index)
    }

    pub fn len(&self) -> usize{
        self.items.len()
    }

    pub fn clear(&mut self){
        self.items.clear();
        self.notify();
    }

    pub fn iter(&self) -> Iter<Item>{
        self.items.iter()
    }
    
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Item>{
        self.items.iter_mut()
    }
}

pub type ItemCollectionProperty = SharedProperty<ItemCollection>;

impl ItemCollectionProperty{
    pub fn new() -> Self{
        Self::from_value(ItemCollection::new())
    }
}