use wasm_bindgen::prelude::*;
use crate::v_node::{StateHandle, Scope, Component, ComponentDef, RefObject, make_ref_object, ContextConsumerHandle};
use crate::v_dom_node::{VDomNode, hd, VDom, t, hdk};
use wasm_bindgen::JsCast;
use std::rc::Rc;
use crate::test_context;

pub struct Def2 {

}

impl ComponentDef<VDom, TestComponent2> for Def2 {
    fn name(&self) -> &'static str {
        "Component2"
    }

    fn make(&self, scope: &mut Scope) -> Rc<TestComponent2> {
        Rc::new(TestComponent2 {
            // click_count: scope.use_state(3)
            content: scope.use_state(String::from("initial")),
            input_ref: make_ref_object(None),
            some_context: scope.use_context(&test_context::DEF)
        })
    }
}

pub static DEF: Def2 = Def2 {};

pub struct TestComponent2 {
    content: StateHandle<String>,
    input_ref: RefObject<web_sys::HtmlElement>,
    some_context: ContextConsumerHandle<test_context::ContextType>
}

pub struct Props {
}

pub struct Ref {
    pub some_action: Box<dyn Fn() -> ()>
}

impl Component<VDom> for TestComponent2 {
    type Props = Props;
    type Ref = Ref;

    fn def(&self) -> &'static dyn ComponentDef<VDom, TestComponent2> {
        &DEF
    }

    fn render(self: Rc<Self>, _props: &Props, ref_object: &RefObject<Ref>) -> VDomNode {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from("component 2 render")));
        let c = self.clone();
        let cc = self.clone();
        *ref_object.borrow_mut() = Some(Ref {
            some_action: Box::new(move || {
                // web_sys::console::log(&js_sys::Array::from(&JsValue::from(&*cc.content.get())));
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
                    c.content.request_update(event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().value());
                }))
            ], map!{
                String::from("value") => self.content.get().clone()
            }, vec![], Some(self.input_ref.clone())),
            hdk("div",
                vec![
                ],
                std::collections::HashMap::new(),
                if self.some_context.get().count % 2 == 0 {
                    vec![
                        (String::from("1"), t("a")),
                        (String::from("2"), t("b")),
                        (String::from("3"), t("c")),
                        (String::from("4"), t("d")),
                        (String::from("5"), t(&self.some_context.get().count.to_string())),
                    ]
                } else {
                    vec![
                        (String::from("5"), t(&self.some_context.get().count.to_string())),
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

