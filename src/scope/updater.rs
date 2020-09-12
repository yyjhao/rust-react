use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::scope::renderer::Renderer;
use crate::scope::scope::Scope;
use crate::scope::effect::EffectStoreT;

pub struct Updater {
    dirty_renderer: Vec<Weak<RefCell<dyn Renderer>>>,
}

impl Updater {
    pub fn new() -> Updater {
        Updater {
            dirty_renderer: vec![]
        }
    }

    pub fn mark_update(&mut self, renderer: &Rc<RefCell<dyn Renderer>>) -> usize {
        self.dirty_renderer.push(Rc::downgrade(renderer));
        self.dirty_renderer.len()
    }

    pub fn get_updatable(&mut self) -> Vec<Rc<RefCell<dyn Renderer>>> {
        let dirty_renderer = std::mem::replace(&mut self.dirty_renderer, vec![]);
        let mut unwrapped: Vec<Rc<RefCell<dyn Renderer>>> = dirty_renderer.into_iter().filter_map(|r| {
            r.upgrade()
        }).collect();
        unwrapped.dedup_by(|r1, r2| {
            std::ptr::eq(r1.as_ref(), r2.as_ref())
        });
        unwrapped
    }
}

pub fn update<T: FnOnce(&mut Scope)>(renderer: &Rc<RefCell<dyn Renderer>>, update_func: T) {
    let token = {
        let mut renderer_mut = renderer.try_borrow_mut().unwrap();
        let result = {
            let u_mut = renderer_mut.updater();
            let mut updater = u_mut.try_borrow_mut().unwrap();
            updater.mark_update(&renderer)
        };
        let scope = renderer_mut.scope_mut();
        update_func(scope);
        result
    };
    let updater = {
        let renderer_ref = renderer.try_borrow().unwrap();
        renderer_ref.updater().clone()
    };
    
    let updatable = {
        if token == 1 {
            updater.try_borrow_mut().unwrap().get_updatable()
        } else {
            vec![]
        }
    };
    let mut effects: Vec<Rc<dyn EffectStoreT>> = vec![];
    for r in updatable.into_iter() {
        let mut mut_r = r.try_borrow_mut().unwrap();
        mut_r.maybe_update();
        for e in mut_r.scope_mut().effect_hooks.hooks.iter() {
            effects.push(e.clone());
        }
    }
    for e in effects.into_iter() {
        e.execute();
    }
}