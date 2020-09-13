use wasm_bindgen::prelude::*;
use std::rc::Rc;
use crate::scope::{ComponentScope, RefObject};
use crate::v_node::{h, ct, ComponentModel};
use crate::v_dom_node::{VDomNode, ordered_children, hd, t, VDom, VDomElement};
use im_rc::vector::Vector;
use crate::components::root;
use std::cell::RefCell;
use crate::components::task;
use crate::components::style_context;

fn filter_vector<T: Clone, P: Fn(&T) -> bool>(vector: Vector<T>, predicate: P) -> Vector<T> {
    let mut cur = Vector::<T>::new();
    let mut cur_anchor = 0;
    let mut cur_move = 0;
    let mut vector_clone = vector.clone();
    for item in vector.iter() {
        if predicate(item) {
            cur_move += 1;
        } else {
            if cur_move != cur_anchor {
                cur.append(vector_clone.slice(cur_anchor..cur_move));
                cur_move = cur_anchor;
            }
            cur_move += 1;
            cur_anchor += 1;
        }
    }
    if cur_move != cur_anchor {
        cur.append(vector_clone.slice(cur_anchor..cur_move));
    }

    cur
}

#[derive(PartialEq)]
pub struct Model {

}

impl ComponentModel<VDom, ()> for Model {
    fn render(&self, scope: &mut ComponentScope, _ref_object: &RefObject<()>) -> VDomNode {
        let (tasks, set_tasks) = scope.use_state(Vector::<Rc<task::Task>>::new());
        let (view_type, set_view_type) = scope.use_state(root::ViewType::All);
        let tasks_2 = tasks.clone();
        let set_tasks_2 = set_tasks.clone();
        let id = scope.use_ref::<usize>();
        let tasks_for_display = match view_type {
            root::ViewType::All => tasks,
            root::ViewType::Incomplete => filter_vector(tasks, |t| { !t.completed }),
            root::ViewType::Completed => filter_vector(tasks, |t| { t.completed })
        };
        web_sys::console::log_2(&JsValue::from("view type"), &JsValue::from(match view_type {
            root::ViewType::All => "all",
            root::ViewType::Incomplete => "incomplete",
            root::ViewType::Completed => "completed"
        }));
        let (style, set_style) = scope.use_state(style_context::StyleType::Light);
        let s2 = style.clone();
        ct(style,
            *ordered_children(vec![
                hd(VDomElement {
                    tag_name: "div",
                    listeners: vec![
                        ("click", scope.use_callback(Box::new(move |scope, _| {
                            set_style(scope, Box::new(|s| {
                                match s {
                                    style_context::StyleType::Light => style_context::StyleType::Dark,
                                    style_context::StyleType::Dark => style_context::StyleType::Light
                                }
                            }))
                        })))
                    ],
                    attributes: std::collections::HashMap::new(),
                    children: ordered_children(vec![ t(match s2 {
                        style_context::StyleType::Light => "light",
                        style_context::StyleType::Dark => "dark"
                    }) ]),
                    ref_object: None,
                    style: std::collections::HashMap::new(),
                }),
                h(root::Props {
                    tasks: tasks_for_display,
                    current_view_type: view_type,
                    on_add_task: scope.use_callback(Box::new(move |scope, name| {
                        let mut new_tasks = tasks_2.clone();
                        let mut id_handle = id.try_borrow_mut().unwrap();
                        let current_id = id_handle.unwrap_or(0);
                        *id_handle = Some(current_id + 1);
                        new_tasks.push_back(Rc::new(task::Task {
                            id: current_id,
                            name,
                            completed: false,
                        }));
                        set_tasks(scope, Box::new(|_| {new_tasks}))
                    })),
                    on_task_updated: scope.use_callback(Box::new(move |scope, (id, completed)| {
                        set_tasks_2(scope, Box::new(move |tasks| {
                            let old_task = tasks.get(id).unwrap();
                            let new_task = task::Task {
                                name: old_task.name.clone(),
                                id: old_task.id,
                                completed,
                            };
                            tasks.update(id, Rc::new(new_task))
                        }))
                    })),
                    on_view_updated: scope.use_callback(Box::new(move |scope, view_type| {
                        set_view_type(scope, Box::new(|_| {view_type}));
                    }))
                }, std::rc::Rc::new(RefCell::new(None)))
            ])
        )
    }
}

