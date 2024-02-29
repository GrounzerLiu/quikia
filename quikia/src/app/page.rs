use std::collections::linked_list::{Iter, IterMut};
use std::collections::LinkedList;
use crate::app::SharedApp;
use crate::item::Item;


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
    pub fn find_item(&self, id: usize) -> Option<&Item> {
        if let Some(root_item) = &self.root_item {
            if root_item.get_id() == id {
                return Some(root_item);
            }

            let mut stack = LinkedList::new();
            stack.push_back(root_item);

            while let Some(item) = stack.pop_back() {
                if item.get_id() == id {
                    return Some(item);
                }
                stack.extend(item.get_children().iter());
            }

            return None;
        }
        None
    }

    pub fn find_item_mut(&mut self, id: usize) -> Option<&mut Item> {
        if let Some(root_item) = &mut self.root_item {
            if root_item.get_id() == id {
                return Some(root_item);
            }

            let mut stack = LinkedList::new();
            stack.push_back(root_item);

            while let Some(item) = stack.pop_back() {
                if item.get_id() == id {
                    return Some(item);
                }
                stack.extend(item.get_children_mut().iter_mut());
            }

            return None;
        }
        None
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
    //     let old_item = page.build(app.clone());
    //
    //     page.on_create(app.clone());
    //     self.pages.push_back(PageItem {
    //         page,
    //         root_item: old_item,
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