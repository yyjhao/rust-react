use crate::v_node::{h, ComponentModel};
use crate::scope::{ComponentScope, CallbackHandle, NilRef};
use crate::dom::{VDomNode, ordered_children, hd, t, VDom, VDomElement};
use crate::components::task;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use im_rc::Vector;

#[derive(Clone, PartialEq)]
pub enum ViewType {
    All,
    Incomplete,
    Completed
}

#[derive(PartialEq)]
pub struct Props {
    pub tasks: Vector<Rc<task::Task>>,
    pub on_add_task: CallbackHandle<String>,
    pub on_task_updated: CallbackHandle<(usize, bool)>,
    pub on_view_updated: CallbackHandle<ViewType>,
    pub current_view_type: ViewType
}

impl ComponentModel<VDom, ()> for Props {
    fn render(&self, scope: &mut ComponentScope, _ref_object: &NilRef) -> VDomNode {
        let (new_task_name, new_task_name_handle) = scope.use_state(String::from(""));
        let on_add_task = self.on_add_task.clone();
        let on_view_updated = self.on_view_updated.clone();

        hd(VDomElement {
            tag_name: "div", 
            listeners: vec![],
            attributes: map!{
                "class" => String::from("root")
            },
            style: map! {
                "max-width" => String::from("800px"),
                "margin" => String::from("0 auto"),
                "padding" => String::from("8px"),
                "border" => String::from("2px solid #999"),
                "min-height" => String::from("100vh"),
                "border-bottom-color" => String::from("transparent"),
                "font-family" => String::from("sans-serif")
            },
            children: ordered_children(vec![
                hd(VDomElement {
                    tag_name: "input",
                    listeners: vec![
                        ("input", scope.use_callback(move |scope, event: web_sys::Event| {
                            new_task_name_handle.update(scope, event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().value())
                        })),
                        ("keydown", scope.use_callback(enclose! { (new_task_name) move |scope, event: web_sys::Event| {
                            let key_code = event.dyn_into::<web_sys::KeyboardEvent>().unwrap().key_code();
                            if key_code == 13 && new_task_name.len() > 0 {
                                new_task_name_handle.update(scope, String::from(""));
                                on_add_task.trigger(new_task_name.clone());
                            }
                        }}))
                    ],
                    children: Box::new(VDomNode::Fragment(vec![])),
                    attributes: map!{
                        "value" => new_task_name.clone(),
                        "placeholder" => String::from("Create a new task")
                    },
                    ref_object: None,
                    style: map! {
                        "height" => String::from("40px"),
                        "line-height" => String::from("40px"),
                        "font-size" => String::from("24px"),
                        "width" => String::from("100%"),
                        "display" => String::from("block"),
                        "border" => String::from("2px solid #999"),
                        "border-radius" => String::from("5px")
                    },
                }),
                hd(VDomElement {
                    tag_name: "div",
                    listeners: vec![
                    ],
                    attributes: std::collections::HashMap::new(),
                    children: ordered_children(vec![
                        view_select(false, String::from("All"), scope.use_callback(enclose! { (on_view_updated) move |_, _| {
                            on_view_updated.trigger(ViewType::All)
                        }}), self.current_view_type == ViewType::All),
                        view_select(false, String::from("Completed"), scope.use_callback(enclose! { (on_view_updated) move |_, _| {
                            on_view_updated.trigger(ViewType::Completed)
                        }}), self.current_view_type == ViewType::Completed),
                        view_select(true, String::from("Incomplete"), scope.use_callback(enclose! { (on_view_updated) move |_, _| {
                            on_view_updated.trigger(ViewType::Incomplete)
                        }}), self.current_view_type == ViewType::Incomplete),
                    ]),
                    ref_object: None,
                    style: map! {
                        "border" => String::from("1px solid black"),
                        "height" => String::from("32px"),
                        "border-radius" => String::from("16px"),
                        "display" => String::from("flex"),
                        "margin-top" => String::from("10px"),
                        "overflow" => String::from("hidden")
                    }
                }),
                VDomNode::Fragment(self.tasks.iter().map(|task| {
                    (task.id.to_string(), h(task::Model {
                        task: task.clone(), 
                        on_update_task: self.on_task_updated.clone()
                    }, None))
                }).collect()),
            ]),
            ref_object: None
        })
    }
}

fn view_select(is_last: bool, name: String, on_click: CallbackHandle<web_sys::Event>, active: bool) -> VDomNode {
    hd(VDomElement {
        tag_name: "div",
        listeners: vec![
            ("click", on_click)
        ],
        attributes: std::collections::HashMap::new(),
        children: ordered_children(vec![
            t(&name)
        ]),
        ref_object: None,
        style: map! {
            "border-right" => String::from(if is_last {"none"} else {"1px solid black"}),
            "height" => String::from("32px"),
            "line-height" => String::from("32px"),
            "flex" => String::from("1 1 auto"),
            "text-align" => String::from("center"),
            "cursor" => String::from("pointer"),
            "background" => String::from(if active { "lightblue" } else { "transparent" })
        }
    })
}
