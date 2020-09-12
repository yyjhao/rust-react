use crate::v_node::ComponentModel;
use crate::scope::{Scope, RefObject, CallbackHandle};
use crate::v_dom_node::{VDomNode, ordered_children, hd, t, VDom, VDomElement};
use crate::components::style_context;

#[derive(Clone, PartialEq)]
pub struct Task {
    pub id: usize,
    pub completed: bool,
    pub name: String
}

impl ComponentModel<VDom, ()> for (Task, CallbackHandle<(usize, bool)>) {
    fn render(&self, scope: &mut Scope, _ref_object: &RefObject<()>) -> VDomNode {
        let (task, on_task_updated) = self;
        let on_task_updated_2 = on_task_updated.clone();
        let id = task.id;
        let completed = task.completed;
        let style = scope.use_context::<style_context::StyleType>();
        hd(VDomElement {
            tag_name: "div",
            listeners: vec![
                ("click", scope.use_callback::<web_sys::Event>(Box::new(move |_, _| {
                    on_task_updated_2.trigger((id, !completed));
                })))
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
}

