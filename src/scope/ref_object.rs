use downcast_rs::Downcast;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

pub type NilRef = Option<RefObject<()>>;

pub struct RefObject<T> {
    inner: Rc<RefCell<Option<T>>>
}

impl<T> RefObject<T> {
    pub fn new() -> RefObject<T> {
        RefObject {
            inner: Rc::new(RefCell::new(None))
        }
    }

    pub fn borrow_mut(&self) -> RefMut<Option<T>> {
        self.inner.try_borrow_mut().unwrap()
    }

    pub fn replace(&self, value: Option<T>) {
        self.inner.replace(value);
    }
}

impl<T> PartialEq for RefObject<T> {
   fn eq(&self, other: &Self) -> bool {
       Rc::ptr_eq(&self.inner, &other.inner)
   }
}

impl<T> Clone for RefObject<T> {
   fn clone(&self) -> Self {
        RefObject {
            inner: self.inner.clone()
        } 
   }
}


pub trait RefObjectT: Downcast {

}
impl_downcast!(RefObjectT);

impl<T: 'static> RefObjectT for RefObject<T> {

}