use wasm_bindgen::prelude::*;
use crate::v_node::{Scope, RefObject, h, ct};
use crate::v_dom_node::{VDomNode, ordered_children, hd, t, VDom, VDomElement};
use wasm_bindgen::JsCast;
use im_rc::vector::Vector;
use crate::components::root;
use std::cell::RefCell;

pub fn component_def(scope: &mut Scope, _props: &(), _ref_object: &RefObject<()>) -> VDomNode {
    let (tasks, set_tasks) = scope.use_state(Vector::new());
    let tasks_2 = tasks.clone();
    let id = scope.use_ref::<usize>();
    h(root::component_def, root::Props {
        tasks,
        on_add_task: scope.use_callback(Box::new(move |scope, name| {
            let mut new_tasks = tasks_2.clone();
            let mut id_handle = id.try_borrow_mut().unwrap();
            let current_id = id_handle.unwrap_or(0);
            *id_handle = Some(current_id + 1);
            new_tasks.push_back(root::Task {
                id: current_id,
                name,
                completed: false,
            });
            set_tasks(scope, new_tasks)
        }))
    }, std::rc::Rc::new(RefCell::new(None)))
}

