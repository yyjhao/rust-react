use std::rc::Rc;
use std::cell::RefCell;
use downcast_rs::Downcast;
use crate::scope::renderer::Renderer;
use crate::scope::updater::update;


pub struct ContextConsumerHandle<T: 'static> {
    pub context_node: Rc<ContextNode<T>>,
}

pub trait ContextConsumerHandleT: Downcast {
    fn cleanup(&self, renderer: Rc<RefCell<dyn Renderer>>);
}
impl_downcast!(ContextConsumerHandleT);

impl<T: 'static> ContextConsumerHandleT for ContextConsumerHandle<T> {
    fn cleanup(&self, renderer: Rc<RefCell<dyn Renderer>>) {
        let mut renderers = self.context_node.renderers.try_borrow_mut().unwrap();
        let index = renderers.iter().position(|r| {
            r.as_ptr() == renderer.as_ptr()
        }).unwrap();
        renderers.remove(index);
    }
}

pub type ContextLink = Option<Rc<dyn ContextNodeT>>;

pub struct ContextNode<T> {
    pub parent: ContextLink,
    pub value: RefCell<Rc<T>>,
    pub renderers: RefCell<Vec<Rc<RefCell<dyn Renderer>>>>
}

pub trait ContextNodeT: Downcast {
    fn trigger_update(&self);
    fn parent(&self) -> &ContextLink;
}
impl_downcast!(ContextNodeT);

impl<T: 'static> ContextNodeT for ContextNode<T> {
    fn trigger_update(&self) {
        for r in self.renderers.try_borrow().unwrap().iter() {
            update(r, |scope| {
                scope.mark_update();
            });
        }
    }

    fn parent(&self) -> &ContextLink {
        &self.parent
    }
}

pub fn clone_context_link(context_link: &ContextLink) -> ContextLink {
    return context_link.as_ref().map(|l|{l.clone()})
}

