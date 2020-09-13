use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use crate::dom::v_dom_node::{VDomNode, VDomElement, VDom};
use crate::v_node::VComponentElementT;
use crate::scope::{RefObject, ContextLink, clone_context_link, Updater};
use crate::renderer::{NativeMount, ComponentMount, NativeMountFactory, Mount};
use wasm_bindgen::JsCast;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref, RefMut};

pub struct DomElementMount {
    root_dom_node: web_sys::HtmlElement,
    updater: Rc<RefCell<Updater>>,
    listeners: Vec<(&'static str, wasm_bindgen::closure::Closure<dyn std::ops::Fn(web_sys::Event)>)>,
    style: HashMap<&'static str, String>,
    attributes: HashMap<&'static str, String>,
    children_mount: Option<Mount<VDom>>,
    dom_factory: Rc<DomMountFactory>,
    parent_dom_factory: Rc<DomMountFactory>,
    context_link: ContextLink,
    ref_object: Option<RefObject<web_sys::HtmlElement>>,
}

impl DomElementMount {
    pub fn new(v_element: VDomElement, context_link: ContextLink, dom_factory: Rc<DomMountFactory>, updater: Rc<RefCell<Updater>>) -> DomElementMount {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let dom_element = document.create_element(&v_element.tag_name).unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
        let listeners = v_element.listeners.into_iter().map(|(event, handle)| {
            let listener: Box<dyn Fn(web_sys::Event) -> ()> = Box::new(move |event| {
                handle.trigger(event);
            });
            (event, Closure::wrap(listener))
        }).collect();
        v_element.ref_object.as_ref().map(|inner| {
            inner.replace(Some(dom_element.clone()))
        });
        let mut r = DomElementMount {
            updater,
            root_dom_node: dom_element.clone(),
            style: v_element.style,
            listeners,
            children_mount: None,
            attributes: v_element.attributes,
            dom_factory: Rc::new(DomMountFactory::new(dom_element)),
            parent_dom_factory: dom_factory,
            context_link,
            ref_object: v_element.ref_object,
        };
        r.rerender(*v_element.children);
        r
    }

    fn rerender(&mut self, children: VDomNode) {
        self.children_mount = Some(if let Some(children_mount) = self.children_mount.take() {
            children_mount.update(children, self.dom_factory.clone(), self.updater.clone())
        } else {
            Mount::new(children, clone_context_link(&self.context_link), self.dom_factory.clone(), self.updater.clone())
        });
        for (event, closure) in self.listeners.iter() {
            let function = closure.as_ref().unchecked_ref();
            self.root_dom_node.add_event_listener_with_callback(&event, function).unwrap();
        }
        for (key, value) in self.style.iter() {
            self.root_dom_node.style().set_property(key, value).unwrap();
        }
        for (key, value) in self.attributes.iter() {
            match key {
                &"value" => self.root_dom_node.dyn_ref::<web_sys::HtmlInputElement>().unwrap().set_value(value),
                _ => self.root_dom_node.set_attribute(key, value).unwrap()
            }
        }
    }

    fn update(&mut self, new_node: VDomElement) {
        for (event, closure) in self.listeners.iter() {
            let function = closure.as_ref().unchecked_ref();
            self.root_dom_node.remove_event_listener_with_callback(&event, function).unwrap();
        }
        for (key, _) in self.attributes.iter() {
            self.root_dom_node.remove_attribute(key).unwrap();
        }
        for (key, _) in self.style.iter() {
            self.root_dom_node.style().remove_property(key).unwrap();
        }
        self.attributes = new_node.attributes;
        self.style = new_node.style;
        self.listeners = new_node.listeners.into_iter().map(|(event, handle)| {
            let listener: Box<dyn Fn(web_sys::Event) -> ()> = Box::new(move |event| {
                handle.trigger(event);
            });
            (event, Closure::wrap(listener))
        }).collect();
        if new_node.ref_object.is_some() {
            self.ref_object.as_ref().map(|inner| { inner.replace(Some(self.root_dom_node.clone().dyn_into::<web_sys::HtmlElement>().unwrap())) });
        }
        self.dom_factory.reset_scanner();
        self.rerender(*new_node.children);
    }

    fn unmount(&mut self) {
        self.root_dom_node.parent_element().unwrap().remove_child(&self.root_dom_node).unwrap();
        self.ref_object.as_ref().map(|inner| { inner.replace(None) });
        self.ref_object = None;
        self.listeners = vec![];
        self.parent_dom_factory.remove_dom_child(self.root_dom_node.clone().dyn_into::<web_sys::Node>().ok().unwrap());
        if let Some(mut child) = self.children_mount.take() {
            child.unmount();
        }
    }
}

pub struct DomTextMount {
    root_dom_node: web_sys::Text,
    context_link: ContextLink,
    parent_dom_factory: Rc<DomMountFactory>,
    text: String
}

impl DomTextMount {
    pub fn new(text: String, context_link: ContextLink, parent_dom_factory: Rc<DomMountFactory>) -> DomTextMount {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        
        let text_node = document.create_text_node(&text);
        DomTextMount {
            root_dom_node: text_node,
            text,
            context_link,
            parent_dom_factory
        }
    }

    fn rerender(&self) {
        self.root_dom_node.set_text_content(Some(&self.text));
    }
    fn update(&mut self, new_text: String) {
        self.text = new_text;
        self.rerender();
    }

    fn unmount(&mut self) {
        self.root_dom_node.parent_element().unwrap().remove_child(&self.root_dom_node).unwrap();
        self.parent_dom_factory.remove_dom_child(self.root_dom_node.clone().dyn_into::<web_sys::Node>().ok().unwrap());
    }
}

enum DomMount {
    Element(DomElementMount),
    Text(DomTextMount),
    None
}

impl DomMount{
    fn new(vnode: VDom, context_link: ContextLink, dom_mount_factory: Rc<DomMountFactory>, updater: Rc<RefCell<Updater>>) -> DomMount {
        match vnode {
            VDom::Element(v_element) => DomMount::Element(DomElementMount::new(v_element, context_link, dom_mount_factory, updater)),
            VDom::Text(v_text) => DomMount::Text(DomTextMount::new(v_text, context_link, dom_mount_factory))
        }
    }

    fn get_dom_node(&self) -> &web_sys::Node {
        match self {
            DomMount::Element(element) => &element.root_dom_node,
            DomMount::Text(text) => &text.root_dom_node,
            DomMount::None => panic!("nope")
        }
    }
}


impl NativeMount<VDom> for DomMount {
    fn update(&mut self, new_node: VDom, dom_mount_factory: Rc<dyn NativeMountFactory<VDom>>, updater: Rc<RefCell<Updater>>) {
        *self = match (std::mem::replace(self, DomMount::None), new_node) {
            (DomMount::Element(mut element), VDom::Element(v_element)) => {
                element.update(v_element);
                DomMount::Element(element)
            }
            (DomMount::Text(mut text), VDom::Text(v_text)) => {
                text.update(v_text);
                DomMount::Text(text)
            }
            (mut m, vnode) => {
                m.unmount();
                DomMount::new(vnode, clone_context_link(self.get_context_link()), dom_mount_factory.downcast_rc::<DomMountFactory>().ok().unwrap(), updater)
            }
        }
    }
    fn unmount(&mut self) {
        match self {
            DomMount::Element(element) => element.unmount(),
            DomMount::Text(text) => text.unmount(),
            DomMount::None => ()
        }
    }

    fn get_context_link(&self) -> &ContextLink {
        match self {
            DomMount::Element(element) => &element.context_link,
            DomMount::Text(text) => &text.context_link,
            DomMount::None => &None
        }
    }
}

enum DomChildren {
    Component(Rc<DomMountFactory>),
    Dom(web_sys::Node),
}

pub struct DomMountFactory {
    parent_dom_node: web_sys::HtmlElement,
    dom_children: RefCell<Vec<DomChildren>>,
    current_index: RefCell<usize>,
    parent: Weak<DomMountFactory>
}

impl DomMountFactory {
    fn new(parent_dom_node: web_sys::HtmlElement) -> DomMountFactory {
        DomMountFactory {
            parent_dom_node,
            dom_children: RefCell::new(vec![]),
            current_index: RefCell::new(0),
            parent: Weak::default()
        }
    }

    fn remove_dom_child(&self, dom_node: web_sys::Node) {
        let mut dom_children = self.dom_children.try_borrow_mut().unwrap();
        let current_index = *{self.current_index.try_borrow().unwrap()};
        let pos = dom_children.iter().position(|child| {
            if let DomChildren::Dom(node) = child {
                node == &dom_node
            } else {
                false
            }
        }).unwrap();
        dom_children.remove(pos);
        if pos < current_index {
            *self.current_index.try_borrow_mut().unwrap() -= 1;
        }
    }

    fn remove_component_child(&self, dom_factory: Rc<DomMountFactory>) {
        let mut dom_children = self.dom_children.try_borrow_mut().unwrap();
        let current_index = *{self.current_index.try_borrow().unwrap()};
        let pos = dom_children.iter().position(|child| {
            if let DomChildren::Component(factory) = child {
                Rc::as_ptr(factory) == Rc::as_ptr(&dom_factory)
            } else {
                false
            }
        }).unwrap();
        dom_children.remove(pos);
        if pos < current_index {
            *self.current_index.try_borrow_mut().unwrap() -= 1;
        }
    }

    fn get_dom_at(&self, index: usize) -> Option<web_sys::Node> {
        let dom_children = self.dom_children.try_borrow().unwrap();
        for child in dom_children[index..dom_children.len()].iter() {
            match child {
                DomChildren::Dom(dom) => {
                    return Some(dom.clone());
                }
                DomChildren::Component(component) => {
                    if let Some(result) = component.get_first_dom() {
                        return Some(result.clone());
                    }
                }
            }
        }
        self.parent.upgrade().and_then(|p| {
            p.get_first_dom_after(self)
        })
    }

    fn get_first_dom_after(&self, child: &DomMountFactory) -> Option<web_sys::Node> {
        let dom_children = self.dom_children.try_borrow().unwrap();
        let next_child_pos = dom_children.iter().position(|c| {
            if let DomChildren::Component(component) = c {
                Rc::as_ptr(component) == child
            } else {
                false
            }
        }).unwrap() + 1;
        web_sys::console::log_3(&JsValue::from("get_first_dom_after"), &JsValue::from(next_child_pos.to_string()), &JsValue::from(dom_children.len().to_string()));
        if next_child_pos == dom_children.len() {
            None
        } else {
            self.get_dom_at(next_child_pos)
        }
    }

    fn get_first_dom(&self) -> Option<web_sys::Node> {
        let dom_children = self.dom_children.try_borrow().unwrap();
        for child in dom_children.iter() {
            match child {
                DomChildren::Dom(dom) => {
                    return Some(dom.clone());
                }
                DomChildren::Component(component) => {
                    if let Some(result) = component.get_first_dom() {
                        return Some(result);
                    }
                }
            }
        }
        None
    }

    fn insert_at(&self, mut dom_children: RefMut<Vec<DomChildren>>, index: usize, dom_node: web_sys::Node) {
        let original = dom_children.iter().position(|child| {
            if let DomChildren::Dom(node) = child {
                node == &dom_node
            } else {
                false
            }
        });
        if let Some(original_index) = original {
            if original_index <= index {
                panic!("insert_at original <= index {} {} {}", original_index, index, dom_children.len())
            }
            dom_children.remove(original_index);
        }
        let dom_child = dom_children.get(index);
        let ref_dom = if let Some(child) = dom_child {
            match child {
                DomChildren::Dom(dom) => Some(dom.clone()),
                DomChildren::Component(component) => match component.get_first_dom() {
                    Some(dom) => Some(dom),
                    None => self.get_dom_at(index + 1)
                }
            }
        } else {
            None
        }.or_else(|| {
            self.parent.upgrade().and_then(|p| {
                p.get_first_dom_after(self)
            })
        });
        web_sys::console::log_4(&dom_node, &JsValue::from(ref_dom.as_ref()), &JsValue::from(index.to_string()), &JsValue::from(dom_children.len().to_string()));
        self.parent_dom_node.insert_before(&dom_node, ref_dom.as_ref()).unwrap();
        dom_children.insert(index, DomChildren::Dom(dom_node.clone()));
    }

    fn insert_dom_factory_at(&self, mut dom_children: RefMut<Vec<DomChildren>>, index: usize, dom_factory: Rc<DomMountFactory>) {
        let original = dom_children.iter().position(|child| {
            if let DomChildren::Component(factory) = child {
                Rc::as_ptr(factory) == Rc::as_ptr(&dom_factory)
            } else {
                false
            }
        });
        if let Some(original_index) = original {
            if original_index <= index {
                panic!("insert_dom_factory_at original <= index {} {} {}", original_index, index, dom_children.len())
            }
            dom_children.remove(original_index);
        }
        dom_children.insert(index, DomChildren::Component(dom_factory));
    }
}

impl NativeMountFactory<VDom> for DomMountFactory {
    fn on_unmount(self: Rc<Self>) {
        if let Some(parent) = self.parent.upgrade() {
            parent.remove_component_child(self)
        }
    }

    fn reset_scanner(&self) {
        *self.current_index.try_borrow_mut().unwrap() = 0;
    }

    fn make_native_mount(self: Rc<Self>, vdom: VDom, context_link: ContextLink, updater: Rc<RefCell<Updater>>)-> Rc<RefCell<dyn NativeMount<VDom>>> {
        let mount = DomMount::new(vdom, context_link, self.clone(), updater);
        let index = *{
            self.current_index.try_borrow().unwrap()
        };
        self.insert_at(self.dom_children.try_borrow_mut().unwrap(), index, mount.get_dom_node().clone());
        *self.current_index.try_borrow_mut().unwrap() += 1;
        Rc::new(RefCell::new(mount))
    }

    fn maybe_update_component_mount_sequence(&self, factory: Rc<dyn NativeMountFactory<VDom>>) {
        let dom_children = self.dom_children.try_borrow_mut().unwrap();
        let current_index = * {self.current_index.try_borrow().unwrap()};
        let dom_factory = factory.downcast_rc::<DomMountFactory>().ok().unwrap();
        let maybe_current_child = dom_children.get(current_index);
        match maybe_current_child {
            Some(current_child) => match current_child {
                DomChildren::Dom(_) => {
                    self.insert_dom_factory_at(dom_children, current_index, dom_factory);
                }
                DomChildren::Component(component) => {
                    if Rc::as_ptr(component) != Rc::as_ptr(&dom_factory) {
                        self.insert_dom_factory_at(dom_children, current_index, dom_factory);
                    }
                }
            }
            None => {
                self.insert_dom_factory_at(dom_children, current_index, dom_factory);
            }
        }
        *self.current_index.try_borrow_mut().unwrap() += 1;
    }

    fn maybe_update_native_mount_sequence(&self, mount: Rc<RefCell<dyn NativeMount<VDom>>>) {
        let dom_children = self.dom_children.try_borrow_mut().unwrap();
        let current_index = * {self.current_index.try_borrow().unwrap()};
        let mount_dom = Ref::map(mount.try_borrow().unwrap(), |mount| { mount.downcast_ref::<DomMount>().unwrap().get_dom_node() });
        web_sys::console::log_3(&JsValue::from("maybe_update_native_mount_sequence"), &JsValue::from(current_index.to_string()), &JsValue::from(&*mount_dom));
        let maybe_current_child = dom_children.get(current_index);
        match maybe_current_child {
            Some(current_child) => match current_child {
                DomChildren::Dom(dom_child) => {
                    if dom_child != &*mount_dom {
                        web_sys::console::log_3(&JsValue::from(dom_child), &JsValue::from(&*mount_dom), &JsValue::from(current_index.to_string()));
                        self.insert_at(dom_children, current_index, mount_dom.clone())
                    }
                }
                DomChildren::Component(_) => {
                    self.insert_at(dom_children, current_index, mount_dom.clone())
                }
            }
            None => {
                self.insert_at(dom_children, current_index, mount_dom.clone())
            }
        }
        *self.current_index.try_borrow_mut().unwrap() += 1;
    }

    fn component_native_mount_factory(self: Rc<Self>) -> Rc<dyn NativeMountFactory<VDom>> {
        let result = Rc::new(DomMountFactory {
            parent_dom_node: self.parent_dom_node.clone(),
            dom_children: RefCell::new(vec![]),
            current_index: RefCell::new(0),
            parent: Rc::downgrade(&self)
        });

        let index = {
            *self.current_index.try_borrow().unwrap()
        };
        web_sys::console::log_2(&JsValue::from("component_native_mount_factory"), &JsValue::from(index.to_string()));
        self.insert_dom_factory_at(self.dom_children.try_borrow_mut().unwrap(), index, result.clone());
        *self.current_index.try_borrow_mut().unwrap() += 1;

        result
    }
}

pub fn mount_dom_component(element: Box<dyn VComponentElementT<VDom>>, root_dom_node: web_sys::HtmlElement, updater: Rc<RefCell<Updater>>) {
    let factory = DomMountFactory::new(root_dom_node);
    ComponentMount::new(element, None, Rc::new(factory), updater);
}