use wasm_bindgen::prelude::*;
use crate::v_node::{Scope, ComponentDef, RefObject, h, ct};
use crate::v_dom_node::{VDomNode, ordered_children, hd, t, VDom, VDomElement};
use crate::components::component_2;
use std::rc::Rc;
use std::cell::RefCell;
use crate::test_context;

pub struct Props {
    pub num_rows: usize
}

pub struct Ref {

}

pub fn component_def(scope: &mut Scope, _props: &Props, _ref_object: &RefObject<Ref>) -> VDomNode {
    let (updated_val, set_updated_val) = scope.use_state(false);
    let com_ref = scope.use_ref::<component_2::Ref>();
    let c = com_ref.clone();

    let render_count_ref = scope.use_ref::<usize>();

    scope.use_effect_always(move || {
        let mut render_count = render_count_ref.try_borrow_mut().unwrap();
        match render_count.as_mut() {
            Some(count) => *count += 1,
            None => *render_count = Some(1)
        }
        web_sys::console::log_2(&JsValue::from("render count"), &JsValue::from(render_count.unwrap().to_string()));

        Some(|| {
            web_sys::console::log_1(&JsValue::from("cleanup"));
        })
    });

    ct(test_context::ContextType {
        // name: String::from(if *updated_val { "a" } else { "b" }),
        name: String::from("context"),
        count: if updated_val { 1 } else { 0 }
    },
        hd(VDomElement {
            tag_name: "div",
            listeners: vec![
                ("click", scope.use_callback(Box::new(move |scope: &mut Scope, _| {
                    {let r = com_ref.try_borrow().unwrap();
                    match r.as_ref() {
                        Some(rr) => (rr.some_action)(),
                        None => ()
                    };}
                    set_updated_val(scope, !updated_val);
                })))
            ],
            attributes: map! {
                "class" => String::from("component_1")
            },
            style: map! {
                "color" => String::from("blue")
            },
            children: ordered_children(vec![
                h(component_2::component_def, (), c),
                t("not context: "),
                t(if updated_val { "a" } else { "b" }),
                t(&updated_val.to_string())
            ]),
            ref_object: None
        })
    )
}

