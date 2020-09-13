use crate::v_node::{VNode};
use crate::scope::{Updater, ContextLink, clone_context_link};
use std::rc::Rc;
use std::cell::{RefCell};
use std::collections::HashMap;
use crate::renderer::native::NativeMountFactory;
use crate::renderer::mount::Mount;

pub struct FragmentMount<VNativeNode: 'static> {
    updater: Rc<RefCell<Updater>>,
    content: Vec<(String, Mount<VNativeNode>)>,
    pub context_link: ContextLink,
    native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>
}

impl<VNativeNode: 'static> FragmentMount<VNativeNode> {
    pub fn new(fragment: Vec<(String, VNode<VNativeNode>)>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>, updater: Rc<RefCell<Updater>>) -> FragmentMount<VNativeNode> {
        let mut renderer = FragmentMount {
            updater,
            content: vec![],
            native_mount_factory,
            context_link
        };

        renderer.rerender(fragment);

        renderer
    }

    fn rerender(&mut self, fragment: Vec<(String, VNode<VNativeNode>)>) -> () {
        let mut map = HashMap::new();
        map.reserve(self.content.len());
        let content = std::mem::take(&mut self.content);
        for (key, mount) in content.into_iter() {
            map.insert(key, mount);
        }
        self.content = fragment.into_iter().map(|(key, node)| {
            if map.contains_key(&key) {
                let old_mount = map.remove(&key).unwrap();
                (key, old_mount.update(node, self.native_mount_factory.clone(), self.updater.clone()))
            } else {
                (key, Mount::new(node, clone_context_link(&self.context_link), self.native_mount_factory.clone(), self.updater.clone()))
            }
        }).collect();
        for (_, mut old_mount) in map.into_iter() {
            old_mount.unmount();
        }
    }


    pub fn unmount(&mut self) -> () {
        let content = std::mem::take(&mut self.content);
        for (_, mut old_mount) in content.into_iter() {
            old_mount.unmount();
        }
    }

    pub fn update(&mut self, fragment: Vec<(String, VNode<VNativeNode>)>) -> () {
        self.rerender(fragment)
    }
}