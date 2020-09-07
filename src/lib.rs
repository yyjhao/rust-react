use wasm_bindgen::prelude::*;
#[macro_use]
extern crate downcast_rs;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use components::{app};
use std::rc::Rc;
use crate::v_node::Updater;

mod v_node;
#[macro_use]
mod v_dom_node;
mod dom_renderer;
mod renderer;
mod components;
mod test_context;


#[wasm_bindgen(start)]
pub fn start() -> () {
    console_error_panic_hook::set_once();
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    crate::dom_renderer::mount_dom_component(Box::new(
        crate::v_node::VComponentElement {
            component_def: app::component_def,
            props: (),
            ref_object: std::rc::Rc::new(RefCell::new(None))
        }
    ), document.body().unwrap().query_selector("#mount").unwrap().unwrap().dyn_into::<web_sys::HtmlElement>().unwrap(), Rc::new(RefCell::new(Updater::new())));
    // let renderer = component_renderer::ComponentRenderer::mount(document.body().unwrap().dyn_into::<web_sys::Element>().unwrap(), &component_2::testComponentDef, component_2::Props {
    // });

    // let closure = Closure::once(move || {
    //     renderer.try_borrow_mut().unwrap().unmount();
    // });
    // let function = closure.as_ref().unchecked_ref();

    // window.set_timeout_with_callback_and_timeout_and_arguments(function, 3000, &Array::new()).unwrap();
    // closure.forget();
}