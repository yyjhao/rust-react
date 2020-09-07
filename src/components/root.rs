use wasm_bindgen::prelude::*;
use crate::v_node::{Scope, RefObject, h, ct, CallbackHandle};
use crate::v_dom_node::{VDomNode, ordered_children, hd, t, VDom, VDomElement};
use wasm_bindgen::JsCast;
use im_rc::Vector;

pub struct Props {
    pub tasks: Vector<Task>,
    pub on_add_task: CallbackHandle<String>,
}

#[derive(Clone, PartialEq)]
pub struct Task {
    pub id: usize,
    pub completed: bool,
    pub name: String
}

pub fn component_def(scope: &mut Scope, props: &Props, _ref_object: &RefObject<()>) -> VDomNode {
    let (new_task_name, set_new_task_name) = scope.use_state(String::from(""));
    let set_new_task_name_2 = set_new_task_name.clone();
    let new_task_name_2 = new_task_name.clone();
    let on_add_task = props.on_add_task.clone();
    let (has_rendered, set_has_rendered) = scope.use_state(false);
    let update_has_rendered = scope.use_callback::<()>(Box::new(move |scope, _| {
        set_has_rendered(scope, true);
    }));

    scope.use_effect_always(move || {
        if !has_rendered {
            update_has_rendered.trigger(());
        }
        Some(|| {
        })
    });

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
            "border-bottom-color" => String::from("transparent")
        },
        children: ordered_children(vec![
            hd(VDomElement {
                tag_name: "input",
                listeners: vec![
                    ("input", scope.use_callback(Box::new(move |scope, event| {
                        set_new_task_name(scope, event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().value())
                    }))),
                    ("keydown", scope.use_callback(Box::new(move |scope, event| {
                        let key_code = event.dyn_into::<web_sys::KeyboardEvent>().unwrap().key_code();
                        if key_code == 13 {
                            set_new_task_name_2(scope, String::from(""));
                            on_add_task.trigger(new_task_name.clone());
                        }
                    })))
                ],
                children: Box::new(VDomNode::Fragment(vec![])),
                attributes: map!{
                    "value" => new_task_name_2.clone(),
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
            VDomNode::Fragment(props.tasks.iter().map(|task| {
                (task.id.to_string(), hd(VDomElement {
                    tag_name: "div",
                    listeners: vec![],
                    attributes: std::collections::HashMap::new(),
                    children: Box::new(t(&task.name)),
                    ref_object: None,
                    style: std::collections::HashMap::new()
                }))
            }).collect()),
        ]),
        ref_object: None
    })
}

