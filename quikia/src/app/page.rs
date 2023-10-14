use crate::app::SharedApp;
use crate::item::Item;

pub trait Page{
    fn build(&mut self, app:SharedApp)->Item;
    fn on_create(&mut self, app:SharedApp){}
    fn on_destroy(&mut self, app:SharedApp){}
}

pub struct PageStack{
    pub pages:Vec<(Box<dyn Page>,Item)>,
}

impl PageStack{
    pub fn new() -> Self{
        Self{
            pages:Vec::new(),
        }
    }

    pub fn launch(&mut self, mut page:Box<dyn Page>, app:SharedApp){
        let item = page.build(app.clone());
        page.on_create(app.clone());
        self.pages.push((page,item));
    }

    pub fn exit(&mut self, app:SharedApp){
        if let Some((mut page,_)) = self.pages.pop(){
            page.on_destroy(app.clone());
        }
    }

    pub fn current_page(&mut self) -> Option<(&mut Box<dyn Page>,&mut Item)>{
        self.pages.last_mut().map(|(page,item)| (page,item))
    }

}