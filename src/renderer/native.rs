use crate::scope::{Updater, ContextLink};
use std::rc::Rc;
use std::cell::{RefCell};
use downcast_rs::Downcast;

pub trait NativeMountFactory<VNativeNode: 'static>: Downcast {
    fn make_native_mount(self: Rc<Self>, native_node: VNativeNode, context_link: ContextLink, updater: Rc<RefCell<Updater>>) -> Rc<RefCell<dyn NativeMount<VNativeNode>>>;
    fn component_native_mount_factory(self: Rc<Self>) -> Rc<dyn NativeMountFactory<VNativeNode>>;

    fn reset_scanner(&self);
    fn maybe_update_native_mount_sequence(&self, mount: Rc<RefCell<dyn NativeMount<VNativeNode>>>);
    fn maybe_update_component_mount_sequence(&self, mount: Rc<dyn NativeMountFactory<VNativeNode>>);
    fn on_unmount(self: Rc<Self>);
}
impl_downcast!(NativeMountFactory<VNativeNode>);

pub trait NativeMount<VNativeNode> : Downcast {
    fn get_context_link(&self) -> &ContextLink;
    fn update(&mut self, new_element: VNativeNode, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>, updater: Rc<RefCell<Updater>>);
    fn unmount(&mut self);
}
impl_downcast!(NativeMount<VNativeNode>);

