use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::any::Any;
use crate::scope::renderer::Renderer;
use crate::scope::context::{ContextLink, ContextConsumerHandleT, ContextConsumerHandle, ContextNode, clone_context_link};
use crate::scope::state::{StateStoreT, StateStore};
use crate::scope::effect::{EffectStoreT, EffectStore};
use crate::scope::memo::{MemoStoreT, MemoStore};
use crate::scope::callback::{CallbackHandle, CallbackStoreT, CallbackStore};

pub type RefObject<T> = Rc<RefCell<Option<T>>>;


pub struct HookList<Hook> {
    pub hooks: Vec<Hook>,
    current_index: usize,
}

impl<Hook> HookList<Hook> {
    fn new() -> HookList<Hook> {
        HookList {
            hooks: vec![],
            current_index: 0
        }
    }

    fn clear(&mut self) {
        self.hooks.clear();
        self.current_index = 0;
    }

    fn get(&mut self) -> &mut Hook {
        self.current_index += 1;
        self.hooks.get_mut(self.current_index - 1).unwrap()
    }
}

pub struct Scope {
    update_flag: bool,
    renderer: Rc<RefCell<dyn Renderer>>,
    context_link: ContextLink,
    callback_store: HookList<Box<dyn CallbackStoreT>>,
    state_hooks: HookList<Box<dyn StateStoreT>>,
    ref_hooks: HookList<Rc<dyn Any>>,
    context_hooks: HookList<Rc<dyn ContextConsumerHandleT>>,
    effect_hooks: HookList<Rc<dyn EffectStoreT>>,
    memo_hooks: HookList<Box<dyn MemoStoreT>>,
    has_init: bool
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.reset();
    }
}

impl Scope {
    pub fn new(renderer: Rc<RefCell<dyn Renderer>>, context_link: ContextLink) -> Scope {
        Scope {
            update_flag: false,
            renderer,
            context_link,
            callback_store: HookList::new(),
            state_hooks: HookList::new(),
            ref_hooks: HookList::new(),
            effect_hooks: HookList::new(),
            memo_hooks: HookList::new(),
            context_hooks: HookList::new(),
            has_init: false
        }
    }

    pub fn reset(&mut self) {
        self.state_hooks.clear();
        self.effect_hooks.clear();
        self.callback_store.clear();
        self.memo_hooks.clear();
        self.context_hooks.clear();
        self.ref_hooks.clear();
        self.has_init = false;
    }

    pub fn mark_update(&mut self) {
        self.update_flag = true;
    }

    pub fn clear_update(&mut self) {
        self.update_flag = false;
    }

    pub fn has_update(&self) -> bool {
        self.update_flag
    }

    pub fn effects_iter(&self) -> std::slice::Iter<Rc<dyn EffectStoreT>> {
        self.effect_hooks.hooks.iter()
    }

    pub fn clone_context_link(&self) -> ContextLink {
        clone_context_link(&self.context_link)
    }

    pub fn mark_start_render(&mut self) {
        self.state_hooks.current_index = 0;
        self.effect_hooks.current_index = 0;
        self.ref_hooks.current_index = 0;
        self.context_hooks.current_index = 0;
        self.memo_hooks.current_index = 0;
        self.callback_store.current_index = 0;
    }

    pub fn mark_end_render(&mut self) {
        self.has_init = true;
    }

    pub fn trigger_callback<T: 'static>(&mut self, index: usize, arg: T) {
        let store = std::mem::replace(&mut self.callback_store.hooks[index], Box::new(())).downcast::<CallbackStore<T>>().ok().unwrap();
        let callback = &store.func;
        (callback)(self, arg);
        self.callback_store.hooks[index] = store;
    }

    pub fn use_callback<T: 'static>(&mut self, callback: Box<dyn Fn(&mut Scope, T) -> ()>) -> CallbackHandle<T> {
        if self.has_init {
            let stored_callback = self.callback_store.get();
            *stored_callback = Box::new(CallbackStore {
                func: callback
            });
            CallbackHandle {
                index: self.callback_store.current_index - 1,
                renderer: self.renderer.clone(),
                phantom: std::marker::PhantomData
            }
        } else {
            self.callback_store.hooks.push(Box::new(CallbackStore {
                func: callback
            }));
            CallbackHandle {
                index: self.callback_store.hooks.len() - 1,
                renderer: self.renderer.clone(),
                phantom: std::marker::PhantomData
            }
        }
    }

    pub fn update_state<T: 'static + PartialEq + Clone>(&mut self, index: usize, mapper: Box<dyn FnOnce(&T) -> T>) {
        let store = self.state_hooks.hooks.get(index).unwrap().downcast_ref::<StateStore<T>>().unwrap() as *const StateStore<T>;
        unsafe {
            let ss = store as *mut StateStore<T>;
            (*ss).request_update_map(self, mapper);
        };
    }

    pub fn use_state<T: 'static + PartialEq + Clone>(&mut self, default_value: T) -> (T, Rc<dyn Fn(&mut Scope, Box<dyn FnOnce(&T) -> T>)->()>) {
        if self.has_init {
            let store = self.state_hooks.get().downcast_ref::<StateStore<T>>().unwrap();
            (store.get(), store.update_func.clone())
        } else {
            let store = StateStore::new(default_value.clone(), self.state_hooks.hooks.len());
            let update_func = store.update_func.clone();
            self.state_hooks.hooks.push(Box::new(store));
            (default_value, update_func)
        }
    }

    pub fn use_context<T: 'static>(&mut self) -> Rc<T> {
        if self.has_init {
            self.context_hooks.get().clone().downcast_rc::<ContextConsumerHandle<T>>().ok().unwrap().context_node.value.try_borrow().unwrap().clone()
        } else {
            let handle = Rc::new(self.create_context_handle::<T>());
            let result = handle.context_node.value.try_borrow().unwrap().clone();
            self.context_hooks.hooks.push(handle.clone());
            result
        }
        
    }

    pub fn use_ref<T: 'static>(&mut self) -> Rc<RefCell<Option<T>>> {
        if self.has_init {
            Rc::downcast::<RefCell<Option<T>>>(self.ref_hooks.get().clone()).unwrap()
        } else {
            let handle = Rc::new(RefCell::new(None));
            self.ref_hooks.hooks.push(handle.clone());
            handle
        }
    }

    pub fn use_effect<Basis: Eq + 'static, C: FnOnce() -> () + 'static, F: Fn() -> Option<C> + 'static>(&mut self, effect: F, basis: Basis) {
        if self.has_init {
            let hook_ref = self.effect_hooks.get();
            let original_effect = hook_ref.clone().downcast_rc::<EffectStore<Basis, F, C>>().ok().unwrap();
            if !basis.eq(&original_effect.basis) {
                *hook_ref = Rc::new(original_effect.update(effect, basis));
            }
        } else {
            self.effect_hooks.hooks.push(Rc::new(EffectStore {
                effect,
                cleanup: Rc::new(RefCell::new(None)),
                basis,
                pending_execution: Cell::new(true)
            }));
        }
    }

    pub fn use_memo<F: Fn(&Input) -> Output + 'static, Input: Eq + 'static, Output: 'static>(&mut self, factory: F, input: Input) -> &Output {
        if self.has_init {
            let hook_ref = self.memo_hooks.get();
            let mut original_memo = std::mem::replace(hook_ref, Box::new(())).downcast::<MemoStore<F, Input, Output>>().ok().unwrap();
            if original_memo.cached_input != input {
                original_memo.cached_output = factory(&input);
                original_memo.factory = factory;
            }
            let _ = std::mem::replace(hook_ref, original_memo);
            &hook_ref.downcast_ref::<MemoStore<F, Input, Output>>().unwrap().cached_output
        } else {
            let cached_output = factory(&input);
            self.memo_hooks.hooks.push(Box::new(MemoStore::<F, Input, Output> {
                factory,
                cached_input: input,
                cached_output
            }));
            &self.memo_hooks.hooks.last().unwrap().downcast_ref::<MemoStore<F, Input, Output>>().unwrap().cached_output
        }
    }

    pub fn use_effect_always<C: FnOnce() -> () + 'static, F: Fn() -> Option<C> + Clone + 'static>(&mut self, effect: F) {
        if self.has_init {
            let hook_ref = self.effect_hooks.get();
            let original_effect = hook_ref.clone().downcast_rc::<EffectStore<Option<()>, F, C>>().ok().unwrap();
            *hook_ref = Rc::new(original_effect.update(effect, None));
        } else {
            self.effect_hooks.hooks.push(Rc::new(EffectStore::<Option<()>, F, C> {
                effect,
                cleanup: Rc::new(RefCell::new(None)),
                basis: None,
                pending_execution: Cell::new(true)
            }));
        }
    }

    fn create_context_handle<T>(&self) -> ContextConsumerHandle<T> {
        let context_link = &self.context_link;
        loop {
            let cl = context_link.as_ref().unwrap();
            if let Some(casted) = cl.clone().downcast_rc::<ContextNode<T>>().ok() {
                casted.renderers.try_borrow_mut().unwrap().push(self.renderer.clone());
                return ContextConsumerHandle {
                    context_node: casted
                }
            }
        }
    }

    pub fn cleanup(&mut self) {
        let effect_hooks = std::mem::take(&mut self.effect_hooks.hooks);
        for e in effect_hooks.into_iter() {
            e.cleanup();
        }
        for c in self.context_hooks.hooks.iter() {
            c.cleanup(self.renderer.clone());
        }
    }
}

