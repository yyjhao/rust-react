use crate::v_node::component::{ComponentModel, VComponentElementT, VComponentElement};
use crate::v_node::context::VContextT;

pub enum VNode<VNativeNode: 'static> {
    Native(VNativeNode),
    Component(Box<dyn VComponentElementT<VNativeNode>>),
    Fragment(Vec<(String, VNode<VNativeNode>)>),
    Context(Box<dyn VContextT<VNativeNode>>),
}

impl<VNativeNode> VNode<VNativeNode> {
    pub fn component<Model: ComponentModel<VNativeNode, Ref> + 'static, Ref: 'static>(element: VComponentElement<VNativeNode, Model, Ref>) -> VNode<VNativeNode> where VNativeNode: 'static {
        VNode::Component(Box::new(
            element
        ))
    }
}
