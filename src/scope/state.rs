use std::rc::Rc;
use downcast_rs::Downcast;
use crate::scope::scope::Scope;

pub struct StateHandle<T: Clone + PartialEq + 'static> {
    index: usize,
    phantom: std::marker::PhantomData<T>
}

impl<T: Clone + PartialEq + 'static> PartialEq for StateHandle<T> {
    fn eq(&self, other: &Self) -> bool {
       self.index == other.index
    }
}

impl<T: Clone + PartialEq + 'static> Copy for StateHandle<T> {

}

impl<T: Clone + PartialEq + 'static> Clone for StateHandle<T> {
    fn clone(&self) -> StateHandle<T> {
        *self
    }
}

impl<T: Clone + PartialEq + 'static> StateHandle<T> {
    pub fn update_map<F: FnOnce(&T) -> T>(&self, scope: &mut Scope, mapper: F) {
        scope.update_state_map(self.index, mapper)
    }

    pub fn update(&self, scope: &mut Scope, new_value: T) {
        scope.update_state(self.index, new_value)
    }
}

pub struct StateStore<T: Clone + PartialEq + 'static> {
    pub value: T,
    pub handle: StateHandle<T>
}

impl<T: Clone + PartialEq + 'static> StateStore<T> {
    pub fn new(value: T, index: usize) -> StateStore<T> {
        StateStore {
            value,
            handle: StateHandle {
                index,
                phantom: std::marker::PhantomData
            }
        }
    }
}
pub trait StateStoreT: Downcast {
}
impl_downcast!(StateStoreT);

impl<T: Clone + PartialEq + 'static> StateStoreT for StateStore<T> {

}