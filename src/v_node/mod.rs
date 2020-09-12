mod v_node;
mod component;
mod context;

pub use crate::v_node::component::{ComponentModel, VComponentElementT, VComponentElement};
pub use crate::v_node::context::{VContextT, VContext};
pub use crate::v_node::v_node::VNode;
use crate::scope::RefObject;

pub fn h<VNativeNode, Model: ComponentModel<VNativeNode, Ref> + 'static, Ref: 'static>(component_model: Model, ref_object: RefObject<Ref>) -> VNode<VNativeNode>
    where
        VNativeNode: 'static {
    VNode::component(VComponentElement::new(
            component_model,
            ref_object,
    ))
}

pub fn ct<T: 'static, VNativeNode: 'static>(value: T, children: VNode<VNativeNode>) -> VNode<VNativeNode> {
    VNode::Context(Box::new(VContext {
        value,
        children: Box::new(children),
    }))
}