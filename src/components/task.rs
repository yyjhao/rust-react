use wasm_bindgen::prelude::*;
use crate::v_node::ComponentModel;
use crate::scope::{ComponentScope, CallbackHandle, NilRef};
use crate::dom::{VDomNode, ordered_children, hd, t, VDom, VDomElement};
use crate::components::style_context;
use std::rc::Rc;

#[derive(PartialEq)]
pub struct Task {
    pub id: usize,
    pub completed: bool,
    pub name: String
}

pub struct Model {
    pub task: Rc<Task>,
    pub on_update_task: CallbackHandle<(usize, bool)>
}

impl PartialEq for Model {
    fn eq(&self, other: &Model) -> bool {
        Rc::ptr_eq(&self.task, &other.task) && self.on_update_task == other.on_update_task
    }
}

impl ComponentModel<VDom, ()> for Model {
    fn render(&self, scope: &mut ComponentScope, _ref_object: &NilRef) -> VDomNode {
        let task = &self.task;
        let on_update_task = &self.on_update_task;
        let id = task.id;
        let completed = task.completed;
        let style = scope.use_context::<style_context::StyleType>();
        hd(VDomElement {
            tag_name: "div",
            listeners: vec![
                ("click", scope.use_callback(enclose! { (on_update_task) move |_, _| {
                    on_update_task.trigger((id, !completed));
                }}))
            ],
            attributes: std::collections::HashMap::new(),
            children: ordered_children(vec![
                hd(VDomElement {
                    tag_name: "div",
                    listeners: vec![
                        
                    ],
                    attributes: map!{
                        "class" => String::from(if completed {
                            "completed"
                        } else {
                            "incomplete"
                        })
                    },
                    children: ordered_children(vec![]),
                    ref_object: None,
                    style: map! {
                        "border" => String::from("1px solid black"),
                        "height" => String::from("32px"),
                        "width" => String::from("32px"),
                        "background-color" => String::from(if completed {
                            "green"
                        } else {
                            "transparent"
                        }),
                        "margin-right" => String::from("10px")
                    }
                }),
                t(&task.name),
            ]),
            ref_object: None,
            style: map! {
                "display" => String::from("flex"),
                "align-items" => String::from("center"),
                "padding" => String::from("10px"),
                "font-size" => String::from("24px"),
                "background" => String::from(match *style {
                    style_context::StyleType::Dark => "#555",
                    style_context::StyleType::Light => "#ddd",
                }),
                "color" => String::from(match *style {
                    style_context::StyleType::Dark => "#ddd",
                    style_context::StyleType::Light => "#555",
                })
            }
        })
    }

    fn name(&self) -> &'static str {
        "task"
    }
}

