use crate::v_node::{VComponentElementT, NodeComparisonResult};
use crate::scope::{Updater, Scope, ContextLink, Renderer};
use std::rc::Rc;
use std::cell::{RefCell};
use crate::renderer::native::NativeMountFactory;
use crate::renderer::mount::Mount;

pub struct ComponentMount<VNativeNode: 'static> {
    updater: Rc<RefCell<Updater>>,
    pub scope: Option<Scope>,
    element: Box<dyn VComponentElementT<VNativeNode>>,
    content: Option<Mount<VNativeNode>>,
    pub native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>,
}

impl<VNativeNode: 'static> ComponentMount<VNativeNode> {
    pub fn new(element: Box<dyn VComponentElementT<VNativeNode>>, context_link: ContextLink, native_mount_factory: Rc<dyn NativeMountFactory<VNativeNode>>, updater: Rc<RefCell<Updater>>) -> Rc<RefCell<ComponentMount<VNativeNode>>> {
        let renderer = Rc::new(RefCell::new(ComponentMount {
            updater,
            scope: None,
            element,
            content: None,
            native_mount_factory: native_mount_factory.component_native_mount_factory(),
        }));

        let r: Rc<RefCell<dyn Renderer>> = renderer.clone();

        let scope = Scope::new(r, context_link);

        let r = renderer.clone();
        let mut renderer_mut = r.try_borrow_mut().unwrap();

        renderer_mut.scope = Some(scope);
        renderer_mut.rerender();

        renderer
    }

    pub fn update(&mut self, element: Box<dyn VComponentElementT<VNativeNode>>) {
        let mut scope = self.scope.take().unwrap();
        match element.compare(self.element.as_ref()) {
            NodeComparisonResult::Equal => {
                self.element = element;
                self.scope = Some(scope);
            },
            NodeComparisonResult::SameType => {
                self.scope = Some(scope);
                self.element = element;
                self.native_mount_factory.reset_scanner();
                self.rerender();
            },
            NodeComparisonResult::DifferentType => {
                self.unmount();
                scope.cleanup();
                scope.reset();
                self.element = element;
                self.scope = Some(scope);
                self.rerender();
            }
        }
    }

    fn rerender(&mut self) -> () {
        self.scope.as_mut().unwrap().clear_update();
        let render_result = self.element.render(&mut self.scope.as_mut().unwrap());
        if let Some(current_mount) = self.content.take() {
            self.content = Some(current_mount.update(render_result, self.native_mount_factory.clone(), self.updater.clone()))
        } else {
            self.content = Some(Mount::new(render_result, self.scope.as_ref().unwrap().clone_context_link(), self.native_mount_factory.clone(), self.updater.clone()));
        }
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
        match self.scope.as_ref() {
            Some(scope) => {
                if scope.has_update() {
                    self.native_mount_factory.reset_scanner();
                    self.rerender();
                }
            }
            None => {

            }
        }
    }
}

impl<VNativeNode: 'static> Renderer for ComponentMount<VNativeNode> {
    fn maybe_update(&mut self) {
        self.consume_update();
    } 

    fn scope_mut(&mut self) -> &mut Scope {
        self.scope.as_mut().unwrap()
    }

    fn updater(&self) -> Rc<RefCell<Updater>> {
        self.updater.clone()
    }
}
