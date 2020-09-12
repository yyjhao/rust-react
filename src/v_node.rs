use std::rc::Rc;
use std::cell::RefCell;
use downcast_rs::Downcast;
use crate::scope::{Scope, RefObject, ContextLink, ContextNode,ContextNodeT};

pub trait ComponentModel<VNativeNode, Ref> {
    fn render(self: &Self, scope: &mut Scope, self_ref: &RefObject<Ref>) -> VNode<VNativeNode>;
}

pub struct VComponentElement<VNativeNode, Model, Ref> where VNativeNode: 'static, Model: ComponentModel<VNativeNode, Ref> {
    pub component_model: Model,
    pub ref_object: RefObject<Ref>,
    phantom: std::marker::PhantomData<VNativeNode>
}

impl<VNativeNode, Model, Ref> VComponentElement<VNativeNode, Model, Ref> where VNativeNode: 'static, Model: ComponentModel<VNativeNode, Ref> {
    pub fn new(model: Model, ref_object: RefObject<Ref>) -> VComponentElement<VNativeNode, Model, Ref> {
        VComponentElement {
            component_model: model,
            ref_object,
            phantom: std::marker::PhantomData
        }
    }
}

pub trait VComponentElementT<VNativeNode: 'static>: Downcast {
    fn render(&self, scope: &mut Scope) -> VNode<VNativeNode>;
    fn same_component(&self, other: &(dyn VComponentElementT<VNativeNode> + 'static)) -> bool;
}
impl_downcast!(VComponentElementT<VNativeNode>);

impl<Model: ComponentModel<VNativeNode, Ref> + 'static, Ref: 'static, VNativeNode: 'static> VComponentElementT<VNativeNode> for VComponentElement<VNativeNode, Model, Ref> {
    fn render(&self, scope: &mut Scope) -> VNode<VNativeNode> {
        scope.mark_start_render();
        let result = self.component_model.render(scope, &self.ref_object);
        scope.mark_end_render();
        result
    }

    fn same_component(&self, other: &(dyn VComponentElementT<VNativeNode> + 'static)) -> bool {
        other.downcast_ref::<VComponentElement<VNativeNode, Model, Ref>>().is_some()
    }
}

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

pub enum VNode<VNativeNode: 'static> {
    Native(VNativeNode),
    Component(Box<dyn VComponentElementT<VNativeNode>>),
    Fragment(Vec<(String, VNode<VNativeNode>)>),
    Context(Box<dyn VContextT<VNativeNode>>),
}

impl<VNativeNode> VNode<VNativeNode> {
    fn component<Model: ComponentModel<VNativeNode, Ref> + 'static, Ref: 'static>(element: VComponentElement<VNativeNode, Model, Ref>) -> VNode<VNativeNode> where VNativeNode: 'static {
        VNode::Component(Box::new(
            element
        ))
    }
}

pub fn h<VNativeNode, Model: ComponentModel<VNativeNode, Ref> + 'static, Ref: 'static>(component_model: Model, ref_object: RefObject<Ref>) -> VNode<VNativeNode>
    where
        VNativeNode: 'static {
    VNode::component(VComponentElement::<VNativeNode, Model, Ref> {
            component_model,
            ref_object,
            phantom: std::marker::PhantomData
        })
}

pub fn ct<T: 'static, VNativeNode: 'static>(value: T, children: VNode<VNativeNode>) -> VNode<VNativeNode> {
    VNode::Context(Box::new(VContext {
        value,
        children: Box::new(children),
    }))
}