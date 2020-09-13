#[macro_use]
mod v_dom_node;
mod dom_renderer;

pub use crate::dom::dom_renderer::mount_dom_component;
pub use crate::dom::v_dom_node::{VDomNode, ordered_children, hd, t, VDom, VDomElement};