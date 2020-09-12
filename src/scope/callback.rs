use std::rc::Rc;
use std::cell::RefCell;
use downcast_rs::Downcast;
use crate::scope::renderer::Renderer;
use crate::scope::scope::Scope;
use crate::scope::updater::update;

pub struct CallbackStore<T> {
    pub func: Box<dyn Fn(&mut Scope, T) -> ()>
}
pub trait CallbackStoreT: Downcast {

}
impl_downcast!(CallbackStoreT);
impl<T: 'static> CallbackStoreT for CallbackStore<T> {

}
impl CallbackStoreT for () {

}

#[derive(Clone)]
pub struct CallbackHandle<T> {
    pub index: usize,
    pub renderer: Rc<RefCell<dyn Renderer>>,
    pub phantom: std::marker::PhantomData<T>,
}



impl<T: 'static> CallbackHandle<T> {
    pub fn trigger(&self, arg: T) {
        let index = self.index;
        update(&self.renderer, move |scope| {
            scope.trigger_callback(index, arg)
        });
    }
}