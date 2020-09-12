use std::rc::Rc;
use std::cell::{RefCell, Cell};
use downcast_rs::Downcast;

pub struct EffectStore<Basis: Eq, F: Fn() -> Option<C>, C: FnOnce() -> ()> {
    pub effect: F,
    pub cleanup: Rc<RefCell<Option<C>>>,
    pub basis: Basis,
    pub pending_execution: Cell<bool>
}

impl<Basis: Eq, F: Fn() -> Option<C>, C: FnOnce() -> ()> EffectStore<Basis, F, C> {
    pub fn update(&self, new_effect: F, new_basis: Basis) -> Self {
        EffectStore {
            effect: new_effect,
            basis: new_basis,
            cleanup: self.cleanup.clone(),
            pending_execution: Cell::new(true)
        }
    }
}

pub trait EffectStoreT: Downcast {
    fn execute(&self);
    fn cleanup(&self);
    fn is_pending(&self) -> bool;
}
impl_downcast!(EffectStoreT);

impl<T: Eq + 'static, F: Fn() -> Option<C> + 'static, C: FnOnce() -> () + 'static> EffectStoreT for EffectStore<T, F, C> {
    fn execute(&self) {
        self.cleanup();
        *self.cleanup.borrow_mut() = (self.effect)();
        self.pending_execution.replace(false);
    }
    fn cleanup(&self) {
        if let Some(cleanup) = self.cleanup.borrow_mut().take() {
            cleanup();
        }
    }

    fn is_pending(&self) -> bool {
        self.pending_execution.get()
    }
}

impl EffectStoreT for () {
    fn execute(&self) {
        panic!("Should not")
    }
    fn cleanup(&self) {
        panic!("Should not")
    }

    fn is_pending(&self) -> bool {
        panic!("Should not")
    }
}