use crate::v_node::{VNode, RefObject};
use std::collections::HashMap;

pub type VDomNode = VNode<VDom>;
pub type Listener = (String, Box<dyn Fn(web_sys::Event) -> ()>);

pub struct VDomElement {
    pub tag_name: String,
    pub listeners: Vec<Listener>,
    pub attributes: HashMap<String, String>,
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

pub fn hd(tag_name: &str, listeners: Vec<Listener>, attributes: HashMap<String, String>, children: Vec<VDomNode>, ref_object: Option<RefObject<web_sys::HtmlElement>>) -> VDomNode {
    hdk(tag_name, listeners, attributes, children.into_iter().enumerate().map(|(index, c)| {
        (index.to_string(), c)
    }).collect(), ref_object)
}

pub fn hdk(tag_name: &str, listeners: Vec<Listener>, attributes: HashMap<String, String>, children: Vec<(String, VDomNode)>, ref_object: Option<RefObject<web_sys::HtmlElement>>) -> VDomNode {
    VDomNode::Native(VDom::Element(VDomElement {
        tag_name: String::from(tag_name),
        listeners,
        attributes,
        children: Box::new(VDomNode::Fragment(children)),
        ref_object
    }))
}

pub fn t(s: &str)-> VDomNode {
    VDomNode::Native(VDom::Text(String::from(s)))
}


