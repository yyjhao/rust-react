use std::rc::Rc;
use std::cell::RefCell;
use crate::scope::renderer::Renderer;
use crate::scope::scope::Scope;
use crate::scope::updater::update;

pub struct CallbackHandle<T> {
    pub func: Rc<dyn Fn(&mut Scope, T) -> ()>,
    pub renderer: Rc<RefCell<dyn Renderer>>,
}

impl<T> Clone for CallbackHandle<T> {
    fn clone(&self) -> CallbackHandle<T> {
        CallbackHandle {
            func: self.func.clone(),
            renderer: self.renderer.clone()
        }
    }
}

impl<T> PartialEq for CallbackHandle<T> {
    fn eq(&self, other: &CallbackHandle<T>) -> bool {
        Rc::ptr_eq(&self.func, &other.func) && Rc::ptr_eq(&self.renderer, &other.renderer)
    }
}


impl<T: 'static> CallbackHandle<T> {
    pub fn trigger(&self, arg: T) {
        let func = self.func.clone();
        update(&self.renderer, move |scope| {
            func(scope, arg)
        });
    }
}