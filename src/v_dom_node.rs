use crate::v_node::{VNode, RefObject, CallbackHandle};
use std::collections::HashMap;

pub type VDomNode = VNode<VDom>;
pub type Listener = (&'static str, CallbackHandle<web_sys::Event>);

pub struct VDomElement {
    pub tag_name: &'static str,
    pub listeners: Vec<Listener>,
    pub attributes: HashMap<&'static str, String>,
    pub style: HashMap<&'static str, String>,
    pub children: Box<VDomNode>,
    pub ref_object: Option<RefObject<web_sys::HtmlElement>>
}

pub enum VDom {
    Element(VDomElement),
    Text(String)
}

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

pub fn ordered_children(children: Vec<VDomNode>) -> Box<VDomNode> {
    Box::new(VDomNode::Fragment(children.into_iter().enumerate().map(|(index, c)| {
        (index.to_string(), c)
    }).collect()))
}


pub fn hd(element: VDomElement) -> VDomNode {
    VDomNode::Native(VDom::Element(element))
}

pub fn t(s: &str)-> VDomNode {
    VDomNode::Native(VDom::Text(String::from(s)))
}


