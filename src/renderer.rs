use crate::v_node::{VNode, Renderer, Scope, VComponentElementT, VContextT, ContextLink, clone_context_link, ContextNode};
use std::rc::Rc;
use std::cell::{RefCell};
use std::collections::HashMap;
use downcast_rs::Downcast;

pub trait NativeMountFactory<VNativeNode: 'static>: Downcast {
    fn make_native_mount(self: Rc<Self>, native_node: VNativeNode, context_link: ContextLink) -> Rc<RefCell<dyn NativeMount<VNativeNode>>>;
    fn component_native_mount_factory(self: Rc<Self>) -> Rc<dyn NativeMountFactory<VNativeNode>>;

    fn reset_scanner(&self);
    fn maybe_update_native_mount_sequence(&self, mount: Rc<RefCell<dyn NativeMount<VNativeNode>>>);
    fn maybe_update_component_mount_sequence(&self, mount: Rc<dyn NativeMountFactory<VNativeNode>>);
    fn on_unmount(self: Rc<Self>);
}
impl_downcast!(NativeMountFactory<VNativeNode>);

pub struct ComponentMount<VNativeNode: 'static> {
    scope: Option<Scope>,
    element: Box<dyn VComponentElementT<VNativeNode>>,
    content: Option<Mount<VNativeNode>>,
    native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>,
}

impl<VNativeNode: 'static> ComponentMount<VNativeNode> {
    pub fn new(element: Box<dyn VComponentElementT<VNativeNode>>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>) -> Rc<RefCell<ComponentMount<VNativeNode>>> {
        let renderer = Rc::new(RefCell::new(ComponentMount {
            scope: None,
            element,
            content: None,
            native_mount_factory: native_mount_factory.component_native_mount_factory(),
        }));

        let r: Rc<RefCell<dyn Renderer>> = renderer.clone();

        let scope = Scope::new(r, context_link);

        let r = renderer.clone();
        let mut renderer_mut = r.borrow_mut();

        renderer_mut.scope = Some(scope);
        renderer_mut.rerender();

        renderer
    }

    fn update(&mut self, element: Box<dyn VComponentElementT<VNativeNode>>) {
        let mut scope = self.scope.take().unwrap();
        if element.same_component(self.element.as_ref()) {
            self.scope = Some(scope);
            self.element = element;
            self.native_mount_factory.reset_scanner();
            self.rerender()
        } else {
            self.unmount();
            scope.reset();
            self.element = element;
            self.scope = Some(scope);
            self.rerender();
        }
    }

    fn rerender(&mut self) -> () {
        self.scope.as_mut().unwrap().update_flag = false;
        let render_result = self.element.render(&mut self.scope.as_mut().unwrap());
        if let Some(current_mount) = self.content.take() {
            self.content = Some(current_mount.update(render_result, self.native_mount_factory.clone()))
        } else {
            self.content = Some(Mount::new(render_result, clone_context_link(&self.scope.as_ref().unwrap().context_link), self.native_mount_factory.clone()));
        }
        self.scope.as_mut().unwrap().execute_effects();
        self.maybe_update();
    }

    pub fn unmount(&mut self) -> () {
        if let Some(mut content) = self.content.take() {
            content.unmount();
        }
        self.native_mount_factory.clone().on_unmount();
        self.scope.as_mut().unwrap().cleanup();
        self.scope = None;
        self.content = None;
    }

    pub fn consume_update(&mut self) {
        match self.scope.as_mut() {
            Some(scope) => {
                if scope.update_flag {
                    self.native_mount_factory.reset_scanner();
                    self.rerender();
                }
            }
            None => {

            }
        }
    }
}


pub struct FragmentMount<VNativeNode: 'static> {
    content: Vec<(String, Mount<VNativeNode>)>,
    context_link: ContextLink,
    native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>
}

impl<VNativeNode: 'static> FragmentMount<VNativeNode> {
    pub fn new(fragment: Vec<(String, VNode<VNativeNode>)>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>) -> FragmentMount<VNativeNode> {
        let mut renderer = FragmentMount {
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
                (key, old_mount.update(node, self.native_mount_factory.clone()))
            } else {
                (key, Mount::new(node, clone_context_link(&self.context_link), self.native_mount_factory.clone()))
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

    fn update(&mut self, fragment: Vec<(String, VNode<VNativeNode>)>) -> () {
        self.rerender(fragment)
    }
}

pub struct ContextMount<VNativeNode: 'static> {
    context_link: Rc<ContextNode>,
    children_mount: Option<Box<Mount<VNativeNode>>>,
    native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>,
}

impl<VNativeNode> ContextMount<VNativeNode> {
    pub fn new(c: Box<dyn VContextT<VNativeNode>>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>) -> ContextMount<VNativeNode> {
        let context = c.take();
        let mut result = ContextMount {
            native_mount_factory: native_mount_factory.component_native_mount_factory(),
            context_link: Rc::new(ContextNode {
                parent: context_link,
                context_store: context.store,
                renderers: RefCell::new(vec![])
            }),
            children_mount: None
        };

        result.rerender(*context.children);
        result
    }

    fn rerender(&mut self, children: VNode<VNativeNode>) {
        self.children_mount = Some(Box::new(if let Some(children_mount) = self.children_mount.take() {
            children_mount.update(children, self.native_mount_factory.clone())
        } else {
            Mount::new(children, Some(self.context_link.clone()), self.native_mount_factory.clone())
        }));
    }

    fn update(&mut self, n: Box<dyn VContextT<VNativeNode>>) {
        if n.def_id() == self.context_link.context_store.borrow().def_id() {
            let children = n.push_value(self.context_link.context_store.clone());
            self.native_mount_factory.reset_scanner();
            for r in self.context_link.renderers.borrow().iter() {
                let mut mut_r = r.borrow_mut();
                mut_r.mark_update();
            }
            self.rerender(children);
            for r in self.context_link.renderers.borrow().iter() {
                let mut mut_r = r.borrow_mut();
                mut_r.maybe_update();
            }
        } else {
            let node = n.take();
            let parent = self.context_link.parent.clone();
            self.unmount();
            self.context_link = Rc::new(ContextNode {
                parent,
                context_store: node.store,
                renderers: RefCell::new(vec![])
            });
            self.rerender(*node.children);
        }
    }

    fn unmount(&mut self) {
        self.native_mount_factory.clone().on_unmount();
        if let Some(mut content) = self.children_mount.take() {
            content.unmount();
        }
    }
}

impl<VNativeNode: 'static> Renderer for ComponentMount<VNativeNode> {
    fn maybe_update(&mut self) {
        self.consume_update();
    } 

    fn trigger_update(&mut self, update_func: Box<dyn FnOnce(&mut Scope)>) {
        update_func(self.scope.as_mut().unwrap());
    }

    fn mark_update(&mut self) {
        self.scope.as_mut().unwrap().update_flag = true;
    }
}

pub trait NativeMount<VNativeNode> : Downcast {
    fn get_context_link(&self) -> &ContextLink;
    fn update(&mut self, new_element: VNativeNode, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>);
    fn unmount(&mut self);
}
impl_downcast!(NativeMount<VNativeNode>);

pub enum Mount<VNativeNode: 'static> {
    Component(Rc<RefCell<ComponentMount<VNativeNode>>>),
    Native(Rc<RefCell<dyn NativeMount<VNativeNode>>>),
    Fragment(FragmentMount<VNativeNode>),
    Context(ContextMount<VNativeNode>),
}

impl<VNativeNode: 'static> Mount<VNativeNode> {
    pub fn new(vnode: VNode<VNativeNode>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>) -> Mount<VNativeNode> {
        match vnode {
            VNode::Native(native) => Mount::Native(native_mount_factory.make_native_mount(native, context_link)),
            VNode::Fragment(fragment) => Mount::Fragment(FragmentMount::new(fragment, context_link, native_mount_factory)),
            VNode::Component(component) => Mount::Component(ComponentMount::new(component, context_link, native_mount_factory)),
            VNode::Context(context) => Mount::Context(ContextMount::new(context, context_link, native_mount_factory))
        }
    }

    pub fn update(self, vnode: VNode<VNativeNode>, parent_native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>) -> Mount<VNativeNode> {
        match (self, vnode) {
            (Mount::Native(native_mount), VNode::Native(native_element)) => {
                native_mount.borrow_mut().update(native_element, parent_native_mount_factory.clone());
                parent_native_mount_factory.maybe_update_native_mount_sequence(native_mount.clone());
                Mount::Native(native_mount)
            }
            (Mount::Fragment(mut fragment_mount), VNode::Fragment(fragment)) => {
                fragment_mount.update(fragment);
                Mount::Fragment(fragment_mount)
            }
            (Mount::Component(component_mount), VNode::Component(component_element)) => {
                {
                    parent_native_mount_factory.maybe_update_component_mount_sequence(component_mount.borrow().native_mount_factory.clone());
                }
                component_mount.borrow_mut().update(component_element);
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
                Mount::new(vnode, context_link, parent_native_mount_factory)
            }
        }
    }

    pub fn get_context_link(&self) -> ContextLink {
        match self {
            Mount::Native(native) => clone_context_link(native.borrow().get_context_link()),
            Mount::Fragment(fragment) => clone_context_link(&fragment.context_link),
            Mount::Component(component) => clone_context_link(&component.borrow().scope.as_ref().unwrap().context_link),
            Mount::Context(context) => Some(context.context_link.clone())
        }
    }

    pub fn unmount(&mut self) {
        match self {
            Mount::Native(native) => native.borrow_mut().unmount(),
            Mount::Fragment(fragment) => fragment.unmount(),
            Mount::Component(component) => component.borrow_mut().unmount(),
            Mount::Context(context) => context.unmount()
        }
    }
}
