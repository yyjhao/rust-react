use std::rc::Rc;
use std::cell::RefCell;
use crate::scope::{ContextLink, ContextNode,ContextNodeT};
use crate::v_node::v_node::VNode;

pub struct VContext<VNativeNode: 'static, T: 'static> {
    pub value: T,
    pub children: Box<VNode<VNativeNode>>
}

pub trait VContextT<VNativeNode: 'static> {
    fn to_context_link(self: Box<Self>, parent: ContextLink) -> (Rc<dyn ContextNodeT>, VNode<VNativeNode>);
    fn push_value(self: Box<Self>, context_link: Rc<dyn ContextNodeT>) -> VNode<VNativeNode>;
    fn is_same_context(self: &Self, store: Rc<dyn ContextNodeT>) -> bool;
}

impl<VNativeNode, T> VContextT<VNativeNode> for VContext<VNativeNode, T> {
    fn to_context_link(self: Box<Self>, parent: ContextLink) -> (Rc<dyn ContextNodeT>, VNode<VNativeNode>) {
        (
            Rc::new(ContextNode {
                parent,
                value: RefCell::new(Rc::new(self.value)),
                renderers: RefCell::new(vec![])
            }),
            *self.children
        )
    }

    fn push_value(self: Box<Self>, context_node: Rc<dyn ContextNodeT>) -> VNode<VNativeNode> {
        let node = context_node.downcast_rc::<ContextNode<T>>().ok().unwrap();
        *node.value.try_borrow_mut().unwrap() = Rc::new(self.value);
        *self.children
    }

    fn is_same_context(self: &Self, context_node: Rc<dyn ContextNodeT>) -> bool {
        context_node.downcast_rc::<ContextNode<T>>().is_ok()
    }
}
