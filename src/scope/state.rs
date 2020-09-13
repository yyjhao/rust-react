use std::rc::Rc;
use downcast_rs::Downcast;
use crate::scope::scope::Scope;

pub struct StateStore<T: Clone + PartialEq + 'static> {
    pub value: T,
    pub update_func: Rc<dyn Fn(&mut Scope, Box<dyn FnOnce(&T) -> T>)>
}

impl<T: Clone + PartialEq + 'static> StateStore<T> {
    pub fn new(value: T, index: usize) -> StateStore<T> {
        StateStore {
            value,
            update_func: Rc::new(move |scope: &mut Scope, mapper| {
                scope.update_state(index, mapper)
            })
        }
    }
}
pub trait StateStoreT: Downcast {
}
impl_downcast!(StateStoreT);

impl<T: Clone + PartialEq + 'static> StateStoreT for StateStore<T> {

}