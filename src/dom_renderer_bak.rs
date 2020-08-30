use wasm_bindgen::prelude::*;
use crate::v_dom_node::{VDomNode, VDomElement};
use crate::v_node::{Component};
use wasm_bindgen::JsCast;

struct AugVDomElement {
    tag_name: String,
    dom_element: web_sys::Element,
    listeners: Vec<(String, wasm_bindgen::closure::Closure<dyn std::ops::Fn()>)>
}

impl AugVDomElement {
    fn new(v_element: VDomElement) -> AugVDomElement {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let dom_element = document.create_element(&v_element.tag_name).unwrap();
        Self::new_with_dom_element(v_element, dom_element)
    }

    fn new_with_dom_element(v_element: VDomElement, dom_element: web_sys::Element) -> AugVDomElement {
        let mut listeners = vec![];
        for (event, listener) in v_element.listeners.into_iter() {
            let closure = Closure::wrap(listener);
            let function = closure.as_ref().unchecked_ref();
            dom_element.add_event_listener_with_callback(&event, function).unwrap();
            listeners.push((event, closure))
        }
        AugVDomElement {
            dom_element,
            tag_name: v_element.tag_name,
            listeners
        }
    }

    fn create_element_children(&self, children: Vec<(String, VDomNode)>) -> Vec<(String, AugVNode)> {
        children.into_iter().map(|(key, child)| {
            (key, match child {
                VDomNode::VNativeElement(vchild, sub_children) => {
                    let child_element = AugVDomElement::new(vchild);
                    self.dom_element.append_child(&child_element.dom_element).unwrap();
                    let avc = child_element.create_element_children(sub_children);
                    AugVNode::AugVDomElement(child_element, avc)
                }
                VDomNode::VText(vchild) => {
                    let child_text = AugVText::new(vchild);
                    self.dom_element.append_child(&child_text.dom_text_node).unwrap();
                    AugVNode::AugVText(child_text)
                }
            })
        }).collect()
    }

    fn create_child(
        &self,
        new_v_node: VDomNode,
    ) -> AugVNode {
        create_element(&self.dom_element, new_v_node)
    }

//     fn remove_child(
//         &self,
//         aug_v_node: AugVNode,
//     ) -> () {
//         self.dom_element.remove_child(aug_v_node.get_dom_node()).unwrap();
//     }

    fn update_child(
        &self,
        old_aug_v_node: AugVNode,
        new_v_node: VDomNode,
    ) -> AugVNode {
        update_element(&self.dom_element, old_aug_v_node, new_v_node)
    }

    fn update_children(&self, old_children: Vec<(String, AugVNode)>, new_children: Vec<(String, VDomNode)>) -> Vec<(String, AugVNode)> {
        let mut old_iter = old_children.into_iter();
        let mut new_iter = new_children.into_iter();
        let mut children = vec!();
        loop {
            let old_item = old_iter.next();
            let new_item = new_iter.next();
            if let Some(old_child) = old_item {
                if let Some(new_child) = new_item {
                    children.push(self.update_child(old_child, new_child));
                } else {
                    self.remove_child(old_child);
                }
            } else if let Some(new_child) = new_item {
                children.push(self.create_child(new_child));
            } else {
                break;
            }
        }
        children
    }
}

pub struct AugVText {
    text: String,
    dom_text_node: web_sys::Text
}

impl AugVText {
    fn new(text: String) -> AugVText {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        
        let text_node = document.create_text_node(&text);
        AugVText {
            dom_text_node: text_node,
            text: text
        }
    }
}

struct AugVComponentElement<Props> {
    dom_element: web_sys::Element,
    component: Box<dyn Component<Props, VDomElement>>,
    current_props: Props,
}

enum AugVNode {
    AugVDomElement(AugVDomElement, Vec<(String, AugVNode)>),
    AugVComponentElement(Box<AugVComponentElement<dyn std::any::Any>>),
    AugVText(AugVText)
}

impl AugVNode {
    fn get_dom_node(&self) -> &web_sys::Node {
        match self {
            AugVNode::AugVDomElement(vel, _) => &vel.dom_element,
            AugVNode::AugVText(vtext) => &vtext.dom_text_node
        }
    }

    fn new(node: VDomNode) -> AugVNode {
        match node {
            VDomNode::VNativeElement(v_element, children) => AugVNode::new_element(v_element, children),
            VDomNode::VText(vtext) => AugVNode::AugVText(AugVText::new(vtext)),
        }
    }

    fn new_element(v_element: VDomElement, children: Vec<(String, VDomNode)>) -> AugVNode {
        let element = AugVDomElement::new(v_element);
        let c = element.create_element_children(children);
        AugVNode::AugVDomElement(element, c)
    }

}

fn create_element(
    container_dom_element: &web_sys::Element,
    new_v_node: VDomNode,
) -> AugVNode {
    let augmented_node = AugVNode::new(new_v_node);
    container_dom_element.append_child(augmented_node.get_dom_node()).unwrap();
    augmented_node
}

fn remove_element(
    container_dom_element: &web_sys::Element,
    aug_v_node: AugVNode,
) -> () {
    container_dom_element.remove_child(aug_v_node.get_dom_node()).unwrap();
}

fn replace_node_with_element(container_dom_element: &web_sys::Element, new_element: VDomElement, new_children: Vec<(String, VDomNode)>, old_dom_node: &web_sys::Node) -> AugVNode {
    let result = AugVNode::new_element(new_element, new_children);
    container_dom_element.replace_child(result.get_dom_node(), &old_dom_node).unwrap();
    result
}

fn replace_node_with_text(container_dom_element: &web_sys::Element, new_text: String, old_dom_node: &web_sys::Node) -> AugVNode {
    let augmented_node = AugVText::new(new_text);
    container_dom_element.replace_child(&augmented_node.dom_text_node, old_dom_node).unwrap(); 
    AugVNode::AugVText(augmented_node)
}

fn update_element(
    container_dom_element: &web_sys::Element,
    old_aug_v_node: AugVNode,
    new_v_node: VDomNode,
) -> AugVNode {
    match new_v_node {
        VDomNode::VNativeElement(new_element, new_children) => {
            if let AugVNode::AugVDomElement(old_element, old_children) = old_aug_v_node {
                if old_element.tag_name == new_element.tag_name {
                    let children = old_element.update_children(old_children, new_children);
                    for (event, closure) in old_element.listeners.iter() {
                        old_element.dom_element.remove_event_listener_with_callback(&event, closure.as_ref().unchecked_ref()).unwrap();
                    }
                    AugVNode::AugVDomElement(AugVDomElement::new_with_dom_element(new_element, old_element.dom_element), children)
                } else {
                    replace_node_with_element(container_dom_element, new_element, new_children, &old_element.dom_element)
                }
            } else {
                replace_node_with_element(container_dom_element, new_element, new_children, old_aug_v_node.get_dom_node())
            }
        }
        VDomNode::VText(new_text) => {
            if let AugVNode::AugVText(old_text) = &old_aug_v_node {
                if new_text.eq(&old_text.text) {
                    old_aug_v_node
                } else {
                    replace_node_with_text(container_dom_element, new_text, &old_text.dom_text_node)
                }
            } else {
                replace_node_with_text(container_dom_element, new_text, old_aug_v_node.get_dom_node())
            }
        },
    }
}

fn maybe_update_element(
    container_dom_element: &web_sys::Element,
    op_old_aug_v_node: Option<AugVNode>,
    op_new_v_node: Option<VDomNode>,
) -> Option<AugVNode> {
    match op_old_aug_v_node {
        None => match op_new_v_node {
            None => None,
            Some(new_v_node) => {
                Some(create_element(container_dom_element, new_v_node))
            }
        }
        Some(old_aug_v_node) => match op_new_v_node {
            None => {
                remove_element(container_dom_element, old_aug_v_node);
                None
            },
            Some(new_v_node) => Some(update_element(container_dom_element, old_aug_v_node, new_v_node))
        },
    }
}


pub struct Mount {
    root_dom_node: web_sys::Element,
    root_augmented_virtual_node: Option<AugVNode>
}

impl Mount {
    pub fn new(el: web_sys::Element) -> Mount {
        Mount {
            root_dom_node: el,
            root_augmented_virtual_node: None
        }
    }

    pub fn render(&mut self, new_node: Option<VDomNode>) {
        let root_augmented_virtual_node = self.root_augmented_virtual_node.take();
        self.root_augmented_virtual_node = maybe_update_element(&self.root_dom_node, root_augmented_virtual_node, new_node);
    }

    pub fn unmount(&mut self) {
        self.root_augmented_virtual_node = None;
    }
}
