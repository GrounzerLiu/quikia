use std::collections::linked_list::{Iter, IterMut};
use std::collections::{HashMap, LinkedList};
use std::sync::Mutex;
use crate::app::SharedApp;
use crate::item::{Item, ItemPath};
//use crate::stack;

pub type ItemMap = HashMap<usize, Item>;

pub trait Page {
    fn build(&mut self, app: SharedApp) -> Item;
    fn on_create(&mut self, app: SharedApp) {}
    fn on_destroy(&mut self, app: SharedApp) {}
}

pub struct PageItem {
    pub page: Box<dyn Page>,
    pub root_item: Option<Item>,
}

impl PageItem {
    pub fn find_item(&self, path: &ItemPath) -> Option<&Item> {
        if self.root_item.is_none() {
            return None;
        }
        self.root_item.as_ref().unwrap().find_item(path)
    }

    pub fn find_item_mut(&mut self, path: &ItemPath) -> Option<&mut Item> {
        if self.root_item.is_none() {
            return None;
        }
        self.root_item.as_mut().unwrap().find_item_mut(path)
    }

    pub fn root_item(&self) -> &Item {
        self.root_item.as_ref().unwrap()
    }

    pub fn root_item_mut(&mut self) -> &mut Item {
        self.root_item.as_mut().unwrap()
    }
}

pub struct PageStack {
    pub pages: LinkedList<PageItem>,
}

impl PageStack {
    pub fn new() -> Self {
        Self {
            pages: LinkedList::new(),
        }
    }

    pub fn push(&mut self, page: Box<dyn Page>) {
        self.pages.push_back(PageItem {
            page,
            root_item: None,
        });
    }

    // pub fn launch(&mut self, mut page: Box<dyn Page>, app: SharedApp) {
    //     let item = page.build(app.clone());
    //
    //     page.on_create(app.clone());
    //     self.pages.push_back(PageItem {
    //         page,
    //         root_item: item,
    //     });
    // }

    pub fn exit(&mut self, app: SharedApp) {
        if let Some(PageItem { mut page, .. }) = self.pages.pop_back() {
            page.on_destroy(app.clone());
        }
    }

    pub fn current_page(&mut self) -> Option<&mut PageItem> {
        self.pages.back_mut()
    }

    pub fn iter_mut(&mut self) -> IterMut<PageItem> {
        self.pages.iter_mut()
    }

    pub fn iter(&self) -> Iter<PageItem> {
        self.pages.iter()
    }
}

#[macro_export]
macro_rules! clonify {
    (|$s:ident $(,$arg:ident)*|$Fn:block) => {
        {
            $(let $arg = $s.$arg.clone();)*
            move||$Fn
        }
    };
}