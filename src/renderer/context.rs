use crate::v_node::{VNode, VContextT, NodeComparisonResult};
use crate::scope::{Updater, ContextLink, ContextNodeT};
use std::rc::Rc;
use std::cell::{RefCell};
use crate::renderer::native::NativeMountFactory;
use crate::renderer::mount::Mount;

pub struct ContextMount<VNativeNode: 'static> {
    updater: Rc<RefCell<Updater>>,
    pub context_link: Rc<dyn ContextNodeT>,
    children_mount: Option<Box<Mount<VNativeNode>>>,
    pub native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>,
}

impl<VNativeNode> ContextMount<VNativeNode> {
    pub fn new(c: Box<dyn VContextT<VNativeNode>>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>, updater: Rc<RefCell<Updater>>) -> ContextMount<VNativeNode> {
        let (context_link, children) = c.to_context_link(context_link);
        let mut result = ContextMount {
            updater,
            native_mount_factory: native_mount_factory.component_native_mount_factory(),
            context_link,
            children_mount: None
        };

        result.rerender(children);
        result
    }

    fn rerender(&mut self, children: VNode<VNativeNode>) {
        self.children_mount = Some(Box::new(if let Some(children_mount) = self.children_mount.take() {
            children_mount.update(children, self.native_mount_factory.clone(), self.updater.clone())
        } else {
            Mount::new(children, Some(self.context_link.clone()), self.native_mount_factory.clone(), self.updater.clone())
        }));
    }

    pub fn update(&mut self, n: Box<dyn VContextT<VNativeNode>>) {
        match n.compare(self.context_link.clone()) {
            NodeComparisonResult::Equal => {
                let children = n.push_value(self.context_link.clone());
                self.native_mount_factory.reset_scanner();
                self.rerender(children);
            },
            NodeComparisonResult::SameType => {
                let children = n.push_value(self.context_link.clone());
                self.native_mount_factory.reset_scanner();
                self.context_link.trigger_update();
                self.rerender(children);
            },
            NodeComparisonResult::DifferentType => {
                let (context_link, children) = n.to_context_link(self.context_link.parent().clone());
                self.unmount();
                self.context_link = context_link;
                self.rerender(children);
            }
        }
    }

    pub fn unmount(&mut self) {
        self.native_mount_factory.clone().on_unmount();
        if let Some(mut content) = self.children_mount.take() {
            content.unmount();
        }
    }
}
