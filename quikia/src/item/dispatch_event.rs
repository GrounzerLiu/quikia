use std::collections::{HashMap, LinkedList};
use skia_safe::Canvas;
use crate::item::{Item, LayoutParams, MeasureMode};
use crate::property::Gettable;
/*
pub fn dispatch_draw(canvas: &Canvas, items: &mut HashMap<usize, Item>, root_id: usize) {
    let mut item_stack = LinkedList::new();
    item_stack.push_back((root_id,false,0_usize));
    'outer:while let Some((item_id,drawn,index)) = item_stack.pop_back(){
        let mut index = index;
        if let Some(item)=items.get(&item_id){
            if !drawn{
                item.before_draw(canvas);
                item.draw(canvas);
            }

            if let Some(item_group) = item.as_item_group(){
                let children_ids = item_group.get_children_ids();
                while index < children_ids.len(){
                    let child_id = children_ids[index];
                    let child_item = items.get(&child_id).unwrap();
                    if child_item.as_item_group().is_none(){
                        child_item.before_draw(canvas);
                        child_item.draw(canvas);
                        child_item.after_draw(canvas);
                    }
                    else {
                        item_stack.push_back((item_id,true,index+1));
                        item_stack.push_back((child_id,false,0));
                        continue 'outer;
                    }
                    index += 1;
                }
                if drawn{
                    item.after_draw(canvas);
                }
            }
        }
    }
}
*/

// pub fn dispatch_measure(items:&mut HashMap<usize,Item>,root_id:usize,width_mode:MeasureMode,height_mode:MeasureMode){
//    let mut item_stack = LinkedList::new();
//     item_stack.push_back(items.get_mut(&root_id).unwrap());
//     while let Some(item_id) = item_stack.pop_back(){
//         let mut item = items.get_mut(&item_id).unwrap();
//         item.measure(width_mode,height_mode);
//         if let Some(item_group) = item.as_item_group_mut(){
//             let children_ids = item_group.get_children_ids();
//             for child_id in children_ids{
//                 item_stack.push_back(items.get_mut(&child_id).unwrap());
//             }
//         }
//     }
// }