use wasm_bindgen::prelude::*;
use crate::v_node::{StateHandle, Scope, ComponentDef, RefObject, ContextConsumerHandle};
use crate::v_dom_node::{VDomNode, hd, VDom, t, hdk};
use wasm_bindgen::JsCast;
use std::rc::Rc;
use crate::test_context;

pub struct Def2 {

}

impl ComponentDef<VDom> for Def2 {
    type Props = Props;
    type Ref = Ref;

    fn name(&self) -> &'static str {
        "Component2"
    }

    fn render(&self, scope: &mut Scope, _props: &Props, ref_object: &RefObject<Ref>) -> VDomNode {
        let content = scope.use_state(String::from("initial"));
        let c = content.clone();
        let some_context = scope.use_context(&test_context::DEF);

        let count = some_context.get().count;
        scope.use_effect(move || {
            web_sys::console::log_2(&JsValue::from("context updated"), &JsValue::from(count.to_string()));
            Some(|| {})
        }, count);

        let input_ref = scope.use_ref();
        *ref_object.borrow_mut() = Some(Ref {
            some_action: Box::new(move || {
                // web_sys::console::log(&js_sys::Array::from(&JsValue::from(&*ccontent.get())));
                // web_sys::console::log(&js_sys::Array::from(&JsValue::from(cc.input_ref.borrow().as_ref().unwrap().dyn_ref::<web_sys::HtmlInputElement>().unwrap().value())));
            })
        });
        hd("div", vec![],
            map!{
                String::from("class") => String::from("component_2")
            },
            vec![
            hd("input", vec![
                (String::from("input"), Box::new(move |event| {
                    content.request_update(event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().value());
                }))
            ], map!{
                String::from("value") => c.get().clone()
            }, vec![], Some(input_ref.clone())),
            hdk("div",
                vec![
                ],
                std::collections::HashMap::new(),
                if some_context.get().count % 2 == 0 {
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
                },
                None
            )
        ], None)
    }
}

pub static DEF: Def2 = Def2 {};

pub struct Props {
}

pub struct Ref {
    pub some_action: Box<dyn Fn() -> ()>
}


