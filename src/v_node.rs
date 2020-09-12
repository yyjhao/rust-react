use wasm_bindgen::prelude::*;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref, Cell};
use std::any::TypeId;
use std::any::Any;

use downcast_rs::Downcast;

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

pub type RefObject<T> = Rc<RefCell<Option<T>>>;

// pub trait ContextDef<T> {
//     fn make_context(&self, initial_value: T) -> ContextStore<T>;
//     fn def_id(&self) -> TypeId;
// }

pub struct ContextConsumerHandle<T: 'static> {
    store: Rc<RefCell<dyn ContextStoreT>>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> ContextConsumerHandle<T> {
    pub fn get(&self) -> Ref<T> {
        Ref::map(self.store.try_borrow().unwrap(), |s| {&s.downcast_ref::<ContextStore<T>>().unwrap().value})
    }
}

pub struct StateStore<T: Clone + PartialEq + 'static> {
    value: T,
    update_func: Rc<dyn Fn(&mut Scope, T)>
}

impl<T: Clone + PartialEq + 'static> StateStore<T> {
    fn new(value: T, index: usize) -> StateStore<T> {
        StateStore {
            value,
            update_func: Rc::new(move |scope: &mut Scope, new_value| {
                scope.update_state(index, new_value)
            })
        }
    }

    fn get(&self) -> T {
        self.value.clone()
    }

    pub fn request_update(&mut self, scope: &mut Scope, new_value: T) {
        scope.update_flag = new_value != self.value;
        self.value = new_value;
    }

    pub fn request_update_map<F: Fn(&T) -> T>(&mut self, scope: &mut Scope, mapper: F) {
        let new_value = mapper(&self.value);
        self.request_update(scope, new_value);
    }
}
pub trait StateStoreT: Downcast {
}
impl_downcast!(StateStoreT);

impl<T: Clone + PartialEq + 'static> StateStoreT for StateStore<T> {

}

static CONTEXT_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
fn get_id() -> usize { CONTEXT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }

pub type ContextDef<T> = std::marker::PhantomData<T>;

pub trait Renderer {
    fn maybe_update(&mut self);
    fn scope_mut(&mut self) -> &mut Scope;
    fn updater(&self) -> Rc<RefCell<Updater>>;
}

pub struct ContextStore<T: 'static> {
    value: T
}

impl<T: 'static> ContextStore<T> {
    pub fn new(initial_value: T) -> ContextStore<T> {
        ContextStore {
            value: initial_value
        }
    }
}

pub trait ContextStoreT: Downcast {
}
impl_downcast!(ContextStoreT);

impl<T: 'static> ContextStoreT for ContextStore<T> {
}

pub type ContextLink = Option<Rc<ContextNode>>;

pub struct ContextNode {
    pub parent: ContextLink,
    pub context_store: Rc<RefCell<dyn ContextStoreT>>,
    pub renderers: RefCell<Vec<Rc<RefCell<dyn Renderer>>>>
}

pub fn clone_context_link(context_link: &ContextLink) -> ContextLink {
    return context_link.as_ref().map(|l|{l.clone()})
}

struct EffectStore<Basis: Eq, F: Fn() -> Option<C>, C: FnOnce() -> ()> {
    effect: F,
    cleanup: Rc<RefCell<Option<C>>>,
    basis: Basis,
    pending_execution: Cell<bool>
}

impl<Basis: Eq, F: Fn() -> Option<C>, C: FnOnce() -> ()> EffectStore<Basis, F, C> {
    fn update(&self, new_effect: F, new_basis: Basis) -> Self {
        EffectStore {
            effect: new_effect,
            basis: new_basis,
            cleanup: self.cleanup.clone(),
            pending_execution: Cell::new(true)
        }
    }
}

trait EffectStoreT: Downcast {
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

struct MemoStore<F: Fn(&Input) -> Output, Input: 'static + Eq, Output> {
    factory: F,
    cached_output: Output,
    cached_input: Input,
}

trait MemoStoreT: Downcast {

}

impl<F: Fn(&Input) -> Output + 'static, Input: 'static + Eq, Output: 'static> MemoStoreT for MemoStore<F, Input, Output> {

}

impl MemoStoreT for () {

}
impl_downcast!(MemoStoreT);

struct HookList<Hook> {
    hooks: Vec<Hook>,
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
    pub update_flag: bool,
    pub renderer: Rc<RefCell<dyn Renderer>>,
    pub context_link: ContextLink,
    callback_store: HookList<Box<dyn CallbackStoreT>>,
    state_hooks: HookList<Box<dyn StateStoreT>>,
    ref_hooks: HookList<Rc<dyn Any>>,
    effect_hooks: HookList<Rc<dyn EffectStoreT>>,
    memo_hooks: HookList<Box<dyn MemoStoreT>>,
    has_init: bool
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.reset();
    }
}

pub struct CallbackStore<T> {
    func: Box<dyn Fn(&mut Scope, T) -> ()>
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
    index: usize,
    renderer: Rc<RefCell<dyn Renderer>>,
    phantom: std::marker::PhantomData<T>,
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

impl<T: 'static> CallbackHandle<T> {
    pub fn trigger(&self, arg: T) {
        let index = self.index;
        update(&self.renderer, move |scope| {
            scope.trigger_callback(index, arg)
        });
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
            has_init: false
        }
    }

    pub fn reset(&mut self) {
        self.state_hooks.clear();
        self.effect_hooks.clear();
        self.callback_store.clear();
        self.memo_hooks.clear();
        self.has_init = false;
    }

    fn mark_start_render(&mut self) {
        self.state_hooks.current_index = 0;
        self.effect_hooks.current_index = 0;
        self.ref_hooks.current_index = 0;
        self.memo_hooks.current_index = 0;
        self.callback_store.current_index = 0;
    }

    fn mark_end_render(&mut self) {
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

    fn update_state<T: 'static + PartialEq + Clone>(&mut self, index: usize, new_value: T) {
        let store = self.state_hooks.hooks.get(index).unwrap().downcast_ref::<StateStore<T>>().unwrap() as *const StateStore<T>;
        unsafe {
            let ss = store as *mut StateStore<T>;
            (*ss).request_update(self, new_value);
        };
    }

    pub fn use_state<T: 'static + PartialEq + Clone>(&mut self, default_value: T) -> (T, Rc<dyn Fn(&mut Scope, T)->()>) {
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

    pub fn use_context<T>(&mut self, context_def: &ContextDef<T>) -> Rc<ContextConsumerHandle<T>> {
        if self.has_init {
            Rc::downcast::<ContextConsumerHandle<T>>(self.ref_hooks.get().clone()).unwrap()
        } else {
            let handle = Rc::new(self.create_context_handle(context_def));
            self.ref_hooks.hooks.push(handle.clone());
            handle
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

    fn create_context_handle<T>(&self, context_def: &ContextDef<T>) -> ContextConsumerHandle<T> {
        let context_link = &self.context_link;
        loop {
            let cl = context_link.as_ref().unwrap();
            let store_copy = cl.context_store.clone();
            let store = cl.context_store.try_borrow().unwrap();
            if store.downcast_ref::<ContextStore<T>>().is_some() {
                cl.renderers.try_borrow_mut().unwrap().push(self.renderer.clone());
                return ContextConsumerHandle {
                    store: store_copy,
                    phantom: std::marker::PhantomData,
                }
            }
        }
    }

    pub fn cleanup(&mut self) {
        let effect_hooks = std::mem::take(&mut self.effect_hooks.hooks);
        for e in effect_hooks.into_iter() {
            e.cleanup();
        }
    }
}


pub type ComponentDef<VNativeNode, Props, Ref> = fn(scope: &mut Scope, props: &Props, self_ref: &RefObject<Ref>) -> VNode<VNativeNode>;

pub struct VComponentElement<VNativeNode, Props, Ref> where VNativeNode: 'static {
    pub component_def: ComponentDef<VNativeNode, Props, Ref>,
    pub props: Props,
    pub ref_object: RefObject<Ref>
}

pub trait VComponentElementT<VNativeNode: 'static>: Downcast {
    fn render(&self, scope: &mut Scope) -> VNode<VNativeNode>;
    fn same_component(&self, other: &(dyn VComponentElementT<VNativeNode> + 'static)) -> bool;
}
impl_downcast!(VComponentElementT<VNativeNode>);

impl<Props: 'static, Ref: 'static, VNativeNode: 'static> VComponentElementT<VNativeNode> for VComponentElement<VNativeNode, Props, Ref> {
    fn render(&self, scope: &mut Scope) -> VNode<VNativeNode> {
        scope.mark_start_render();
        let result = (self.component_def)(scope, &self.props, &self.ref_object);
        scope.mark_end_render();
        result
    }

    fn same_component(&self, other: &(dyn VComponentElementT<VNativeNode> + 'static)) -> bool {
        match other.downcast_ref::<VComponentElement<VNativeNode, Props, Ref>>() {
            Some(other_element) => other_element.component_def as usize == self.component_def as usize,
            None => false
        }
    }
}

pub struct VContext<VNativeNode: 'static, T: 'static> {
    pub store: ContextStore<T>,
    pub children: Box<VNode<VNativeNode>>
}

pub struct VContextS<VNativeNode: 'static> {
    pub store: Rc<RefCell<dyn ContextStoreT>>,
    pub children: Box<VNode<VNativeNode>>
}

pub trait VContextT<VNativeNode: 'static> {
    fn take(self: Box<Self>) -> VContextS<VNativeNode>;
    fn push_value(self: Box<Self>, store: Rc<RefCell<dyn ContextStoreT>>) -> VNode<VNativeNode>;
    fn is_same_context(self: &Self, store: Rc<RefCell<dyn ContextStoreT>>) -> bool;
}

impl<VNativeNode, T> VContextT<VNativeNode> for VContext<VNativeNode, T> {
    fn take(self: Box<Self>) -> VContextS<VNativeNode> {
        VContextS {
            store: Rc::new(RefCell::new(self.store)),
            children: self.children
        }
    }
    fn push_value(self: Box<Self>, store: Rc<RefCell<dyn ContextStoreT>>) -> VNode<VNativeNode> {
        let s = store.try_borrow_mut().unwrap().downcast_ref::<ContextStore<T>>().unwrap() as *const ContextStore<T>;
        unsafe {
            let ss = s as *mut ContextStore<T>;
            (*ss).value = self.store.value
        };
        *self.children
    }

    fn is_same_context(self: &Self, store: Rc<RefCell<dyn ContextStoreT>>) -> bool {
        store.try_borrow_mut().unwrap().downcast_ref::<ContextStore<T>>().is_some()
    }
}

pub enum VNode<VNativeNode: 'static> {
    Native(VNativeNode),
    Component(Box<dyn VComponentElementT<VNativeNode>>),
    Fragment(Vec<(String, VNode<VNativeNode>)>),
    Context(Box<dyn VContextT<VNativeNode>>),
}

impl<VNativeNode> VNode<VNativeNode> {
    fn component<Props: 'static, Ref: 'static>(element: VComponentElement<VNativeNode, Props, Ref>) -> VNode<VNativeNode> where VNativeNode: 'static {
        VNode::Component(Box::new(
            element
        ))
    }
}

pub fn h<VNativeNode, Props: 'static, Ref: 'static>(component_def: ComponentDef<VNativeNode, Props, Ref>, props: Props, ref_object: RefObject<Ref>) -> VNode<VNativeNode>
    where
        VNativeNode: 'static {
    VNode::component(VComponentElement::<VNativeNode, Props, Ref> {
            component_def,
            props: props,
            ref_object
        })
}

pub fn ct<T: 'static, VNativeNode: 'static>(context_def: &ContextDef<T>, value: T, children: VNode<VNativeNode>) -> VNode<VNativeNode> {
    VNode::Context(Box::new(VContext {
        store: ContextStore {
            value,
        },
        children: Box::new(children),
    }))
}