use wasm_bindgen::prelude::*;
use crate::v_node::{StateHandle, Scope, Component, ComponentDef, RefObject, h, ct};
use crate::v_dom_node::{VDomNode, hd, t, VDom};
use crate::component_2;
use std::rc::Rc;
use std::cell::RefCell;
use crate::test_context;

pub struct Def {

}

impl ComponentDef<VDom, TestComponent> for Def {
    fn name(&self) -> &'static str {
        "Component"
    }

    fn make(&self, scope: &mut Scope) -> Rc<TestComponent> {
        TestComponent::new(scope)
    }
}

pub static DEF: Def = Def {};

pub struct TestComponent {
    updated: StateHandle<bool>,
    com_ref: RefObject<component_2::Ref>
}

pub struct Props {
    pub num_rows: usize
}

impl TestComponent {
    pub fn new(scope: &mut Scope) -> Rc<TestComponent> {
        Rc::new(TestComponent {
            updated: scope.use_state(false),
            com_ref: Rc::new(RefCell::new(None))
        })
    }
}

pub struct Ref {

}

impl Component<VDom> for TestComponent {
    type Props = Props;
    type Ref = Ref;
    fn def(&self) -> &'static dyn ComponentDef<VDom, TestComponent> {
        &DEF
    }
    fn render(self: Rc<Self>, _props: &Props, _ref_object: &RefObject<Ref>) -> VDomNode {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from("component 1 render")));
        let updated_val = self.updated.get();
        let c = self.clone();

        ct(&test_context::DEF, test_context::ContextType {
            // name: String::from(if *updated_val { "a" } else { "b" }),
            name: String::from("context"),
            count: if *updated_val { 1 } else { 0 }
        },
            hd("div", vec![
                (String::from("click"), Box::new(move |_| {
                    {let r = c.com_ref.borrow();
                    match r.as_ref() {
                        Some(rr) => (rr.some_action)(),
                        None => ()
                    };}
                    let new_val = {
                        !*c.updated.get()
                    };
                    c.updated.request_update(new_val);
                }))
            ], map! {
                String::from("class") => String::from("component_1")
            }, vec![
                h(&component_2::DEF, component_2::Props {

                }, self.com_ref.clone()),
                t("not context: "),
                // t(if *updated_val { "a" } else { "b" })
                t(&updated_val.to_string())
            ], None)
        )
    }
}

