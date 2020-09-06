use wasm_bindgen::prelude::*;
#[macro_use]
extern crate downcast_rs;
use wasm_bindgen::JsCast;
use std::cell::RefCell;

mod v_node;
#[macro_use]
mod v_dom_node;
mod dom_renderer;
mod renderer;
mod component;
mod component_2;
mod test_context;
// mod component_renderer;


#[wasm_bindgen(start)]
pub fn start() -> () {
    console_error_panic_hook::set_once();
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    crate::dom_renderer::mount_dom_component(Box::new(
        crate::v_node::VComponentElement {
            component_def: component::component_def,
            props: component::Props {
                num_rows: 1
            },
            ref_object: std::rc::Rc::new(RefCell::new(None))
        }
    ), document.body().unwrap().query_selector("#mount").unwrap().unwrap().dyn_into::<web_sys::HtmlElement>().unwrap());
    // let renderer = component_renderer::ComponentRenderer::mount(document.body().unwrap().dyn_into::<web_sys::Element>().unwrap(), &component_2::testComponentDef, component_2::Props {
    // });

    // let closure = Closure::once(move || {
    //     renderer.borrow_mut().unmount();
    // });
    // let function = closure.as_ref().unchecked_ref();

    // window.set_timeout_with_callback_and_timeout_and_arguments(function, 3000, &Array::new()).unwrap();
    // closure.forget();
}