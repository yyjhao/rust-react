use wasm_bindgen::prelude::*;
#[macro_use]
extern crate downcast_rs;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use components::{app};
use std::rc::Rc;
use crate::v_node::VComponentElement;
use crate::scope::{Updater, RefObject};

mod v_node;
#[macro_use]
mod dom;
mod renderer;
mod components;
mod scope;


#[wasm_bindgen(start)]
pub fn start() -> () {
    console_error_panic_hook::set_once();
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    crate::dom::mount_dom_component(
        Box::new(VComponentElement::new(
            app::Model {
            },
           None 
        )),
        document.body().unwrap().query_selector("#mount").unwrap().unwrap().dyn_into::<web_sys::HtmlElement>().unwrap(), Rc::new(RefCell::new(Updater::new())));
}