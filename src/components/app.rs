use wasm_bindgen::prelude::*;
use std::rc::Rc;
use crate::scope::{ComponentScope, RefObject, NilRef};
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
    fn render(&self, scope: &mut ComponentScope, _: &NilRef) -> VDomNode {
        let (tasks, tasks_handle) = scope.use_state(Vector::<Rc<task::Task>>::new());
        let (view_type, view_type_handle) = scope.use_state(root::ViewType::All);
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
        let (style, style_handle) = scope.use_state(style_context::StyleType::Light);
        ct(style,
            *ordered_children(vec![
                hd(VDomElement {
                    tag_name: "div",
                    listeners: vec![
                        ("click", scope.use_callback(move |scope, _| {
                            style_handle.update_map(scope, |s| {
                                match s {
                                    style_context::StyleType::Light => style_context::StyleType::Dark,
                                    style_context::StyleType::Dark => style_context::StyleType::Light
                                }
                            })
                        }))
                    ],
                    attributes: std::collections::HashMap::new(),
                    children: ordered_children(vec![ t(match style {
                        style_context::StyleType::Light => "light",
                        style_context::StyleType::Dark => "dark"
                    }) ]),
                    ref_object: None,
                    style: std::collections::HashMap::new(),
                }),
                h(root::Props {
                    tasks: tasks_for_display,
                    current_view_type: view_type,
                    on_add_task: scope.use_callback_memo(|input, scope, name| {
                        let (id, tasks_handle) = input;
                        tasks_handle.update_map(scope, |tasks| {
                            let mut new_tasks = tasks.clone();
                            let mut id_handle = id.borrow_mut();
                            let current_id = id_handle.unwrap_or(0);
                            *id_handle = Some(current_id + 1);
                            new_tasks.push_back(Rc::new(task::Task {
                                id: current_id,
                                name,
                                completed: false,
                            }));
                            new_tasks
                        })
                    }, (id, tasks_handle)),
                    on_task_updated: scope.use_callback_memo(|tasks_handle, scope, (id, completed)| {
                        tasks_handle.update_map(scope, |tasks| {
                            let old_task = tasks.get(id).unwrap();
                            let new_task = task::Task {
                                name: old_task.name.clone(),
                                id: old_task.id,
                                completed,
                            };
                            tasks.update(id, Rc::new(new_task))
                        })
                    }, tasks_handle),
                    on_view_updated: scope.use_callback(move |scope, view_type| {
                        view_type_handle.update(scope, view_type);
                    })
                }, None)
            ])
        )
    }
}

