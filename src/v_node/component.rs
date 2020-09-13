use downcast_rs::Downcast;
use crate::scope::{ComponentScope, Scope, RefObject};
use crate::v_node::v_node::VNode;

pub trait ComponentModel<VNativeNode, Ref> {
    fn render(self: &Self, scope: &mut ComponentScope, self_ref: &RefObject<Ref>) -> VNode<VNativeNode>;
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
        let result = self.component_model.render(&mut scope.component_scope, &self.ref_object);
        scope.mark_end_render();
        result
    }

    fn same_component(&self, other: &(dyn VComponentElementT<VNativeNode> + 'static)) -> bool {
        other.downcast_ref::<VComponentElement<VNativeNode, Model, Ref>>().is_some()
    }
}
