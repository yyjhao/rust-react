use crate::v_node::{VNode};
use crate::scope::{Updater, ContextLink, clone_context_link, Renderer, update};
use std::rc::Rc;
use std::cell::{RefCell};
use crate::renderer::component::ComponentMount;
use crate::renderer::native::{NativeMount, NativeMountFactory};
use crate::renderer::fragment::FragmentMount;
use crate::renderer::context::ContextMount;

pub enum Mount<VNativeNode: 'static> {
    Component(Rc<RefCell<ComponentMount<VNativeNode>>>),
    Native(Rc<RefCell<dyn NativeMount<VNativeNode>>>),
    Fragment(FragmentMount<VNativeNode>),
    Context(ContextMount<VNativeNode>),
}

impl<VNativeNode: 'static> Mount<VNativeNode> {
    pub fn new(vnode: VNode<VNativeNode>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>, updater: Rc<RefCell<Updater>>) -> Mount<VNativeNode> {
        match vnode {
            VNode::Native(native) => Mount::Native(native_mount_factory.make_native_mount(native, context_link, updater)),
            VNode::Fragment(fragment) => Mount::Fragment(FragmentMount::new(fragment, context_link, native_mount_factory, updater)),
            VNode::Component(component) => Mount::Component({
                let renderer = ComponentMount::new(component, context_link, native_mount_factory, updater);
                let r: Rc<RefCell<dyn Renderer>> = renderer.clone();
                update(&r, |_|{});
                renderer
            }),
            VNode::Context(context) => Mount::Context(ContextMount::new(context, context_link, native_mount_factory, updater))
        }
    }

    pub fn update(self, vnode: VNode<VNativeNode>, parent_native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>, updater: Rc<RefCell<Updater>>) -> Mount<VNativeNode> {
        match (self, vnode) {
            (Mount::Native(native_mount), VNode::Native(native_element)) => {
                native_mount.try_borrow_mut().unwrap().update(native_element, parent_native_mount_factory.clone(), updater);
                parent_native_mount_factory.maybe_update_native_mount_sequence(native_mount.clone());
                Mount::Native(native_mount)
            }
            (Mount::Fragment(mut fragment_mount), VNode::Fragment(fragment)) => {
                fragment_mount.update(fragment);
                Mount::Fragment(fragment_mount)
            }
            (Mount::Component(component_mount), VNode::Component(component_element)) => {
                {
                    parent_native_mount_factory.maybe_update_component_mount_sequence(component_mount.try_borrow().unwrap().native_mount_factory.clone());
                }
                component_mount.try_borrow_mut().unwrap().update(component_element);
                Mount::Component(component_mount)
            }
            (Mount::Context(mut context_mount), VNode::Context(context_node)) => {
                {
                    parent_native_mount_factory.maybe_update_component_mount_sequence(context_mount.native_mount_factory.clone());
                }
                context_mount.update(context_node);
                Mount::Context(context_mount)
            }
            (mut m, vnode) => {
                let context_link = m.get_context_link();
                m.unmount();
                Mount::new(vnode, context_link, parent_native_mount_factory, updater)
            }
        }
    }

    pub fn get_context_link(&self) -> ContextLink {
        match self {
            Mount::Native(native) => clone_context_link(native.try_borrow().unwrap().get_context_link()),
            Mount::Fragment(fragment) => clone_context_link(&fragment.context_link),
            Mount::Component(component) => component.try_borrow().unwrap().scope.as_ref().unwrap().clone_context_link(),
            Mount::Context(context) => Some(context.context_link.clone())
        }
    }

    pub fn unmount(&mut self) {
        match self {
            Mount::Native(native) => native.try_borrow_mut().unwrap().unmount(),
            Mount::Fragment(fragment) => fragment.unmount(),
            Mount::Component(component) => component.try_borrow_mut().unwrap().unmount(),
            Mount::Context(context) => context.unmount()
        }
    }
}
