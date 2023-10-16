use std::collections::linked_list::Iter;
use std::collections::LinkedList;
use crate::app::SharedApp;
use crate::item::Item;

pub trait Page{
    fn build(&mut self, app:SharedApp)->Item;
    fn on_create(&mut self, app:SharedApp){}
    fn on_destroy(&mut self, app:SharedApp){}
}

pub struct PageStack{
    pub pages:LinkedList<(Box<dyn Page>,Item)>,
}

impl PageStack{
    pub fn new() -> Self{
        Self{
            pages:LinkedList::new(),
        }
    }

    pub fn launch(&mut self, mut page:Box<dyn Page>, app:SharedApp){
        let item = page.build(app.clone());
        page.on_create(app.clone());
        self.pages.push_back((page,item));
    }

    pub fn exit(&mut self, app:SharedApp){
        if let Some((mut page,_)) = self.pages.pop_back(){
            page.on_destroy(app.clone());
        }
    }

    pub fn current_page(&mut self) -> Option<(&mut Box<dyn Page>,&mut Item)>{
        self.pages.back_mut().map(|(page,item)| (page,item))
    }

    pub fn iter(&self) -> Iter<(Box<dyn Page>, Item)>{
        self.pages.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::linked_list::IterMut<(Box<dyn Page>, Item)>{
        self.pages.iter_mut()
    }

}

#[macro_export]
macro_rules! closure {
    ($Fn:expr,$s:ident $(,$arg:ident)*) => {
        {
            $(let $arg = $s.$arg.clone();)*
            $Fn
        }
    };
}