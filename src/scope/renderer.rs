use std::rc::Rc;
use std::cell::RefCell;
use crate::scope::scope::Scope;
use crate::scope::updater::Updater;

pub trait Renderer {
    fn maybe_update(&mut self);
    fn scope_mut(&mut self) -> &mut Scope;
    fn updater(&self) -> Rc<RefCell<Updater>>;
}
