use wasm_bindgen::prelude::*;
use crate::v_node::{Scope, ComponentDef, RefObject, ContextConsumerHandle};
use crate::v_dom_node::{VDomNode, hd, VDom, t, ordered_children, VDomElement};
use wasm_bindgen::JsCast;
use std::rc::Rc;
use crate::test_context;

pub fn component_def(scope: &mut Scope, _props: &(), ref_object: &RefObject<Ref>) -> VDomNode {
    let (content_val, set_content_val) = scope.use_state(String::from("initial"));
    let some_context = scope.use_context(&test_context::def);

    let count = some_context.get().count;
    scope.use_effect(move || {
        web_sys::console::log_2(&JsValue::from("context updated"), &JsValue::from(count.to_string()));
        Some(|| {})
    }, count);

    let count_times_2 = scope.use_memo(|count| {
        count * 2
    }, count);

    let input_ref = scope.use_ref();
    *ref_object.try_borrow_mut().unwrap() = Some(Ref {
        some_action: Box::new(move || {
            // web_sys::console::log(&js_sys::Array::from(&JsValue::from(&content_val)));
        })
    });
    hd(VDomElement {
        tag_name: "div", 
        listeners: vec![],
        attributes: map!{
            "class" => String::from("component_2")
        },
        style: map! {
            "background-color" => String::from("green")
        },
        children: ordered_children(vec![
            hd(VDomElement {
                tag_name: "input",
                listeners: vec![
                    ("input", scope.use_callback(Box::new(move |scope, event| {
                        set_content_val(scope, event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().value())
                    })))
                ],
                attributes: map!{
                    "value" => content_val
                },
                children: ordered_children(vec![]),
                ref_object: Some(input_ref.clone()),
                style: map! {
                    "background-color" => String::from("red")
                },
            }),
            hd(VDomElement {
                tag_name: "div",
                listeners: vec![
                ],
                attributes: std::collections::HashMap::new(),
                children: Box::new(VDomNode::Fragment(if some_context.get().count % 2 == 0 {
                    vec![
                        (String::from("1"), t("a")),
                        (String::from("2"), t("b")),
                        (String::from("3"), t("c")),
                        (String::from("4"), t("d")),
                        (String::from("5"), t(&some_context.get().count.to_string())),
                    ]
                } else {
                    vec![
                        (String::from("5"), t(&some_context.get().count.to_string())),
                        (String::from("2"), t("b")),
                        (String::from("3"), t("c")),
                        (String::from("1"), t("a")),
                    ]                    
                })),
                style:  std::collections::HashMap::new(),
                ref_object: None
            })
        ]),
        ref_object: None
    })
}

pub struct Ref {
    pub some_action: Box<dyn Fn() -> ()>
}


