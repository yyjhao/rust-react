use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use std::any::TypeId;
use wasm_bindgen::prelude::*;
use std::any::Any;

use downcast_rs::Downcast;

pub type RefObject<T> = Rc<RefCell<Option<T>>>;

pub trait ContextDef<T> {
    fn make_context(&self, initial_value: T) -> ContextStore<T>;
    fn def_id(&self) -> TypeId;
}

pub struct ContextConsumerHandle<T: 'static> {
    store: Rc<RefCell<dyn ContextStoreT>>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> ContextConsumerHandle<T> {
    pub fn get(&self) -> Ref<T> {
        Ref::map(self.store.borrow(), |s| {&s.downcast_ref::<ContextStore<T>>().unwrap().value})
    }
}

pub struct StateHandle<T> {
    state: RefCell<T>,
    renderer: Rc<RefCell<dyn Renderer>>
}

impl<T> StateHandle<T> {
    fn new(this: &Rc<RefCell<dyn Renderer>>, value: T) -> StateHandle<T> {
        StateHandle {
            state: RefCell::new(value),
            renderer: this.clone(),
        }
    }

    pub fn get(&self) -> std::cell::Ref<T> {
        self.state.borrow()
    }

    pub fn request_update(&self, new_value: T) {
        {
            *self.state.borrow_mut() = new_value;
        }
        self.renderer.borrow_mut().on_update();
    }
}

pub trait Renderer {
    fn on_update(&mut self) -> ();

}

pub struct ContextStore<T: 'static> {
    def_id: TypeId,
    value: T
}

impl<T: 'static> ContextStore<T> {
    pub fn new(def_id: TypeId, initial_value: T) -> ContextStore<T> {
        ContextStore {
            def_id,
            value: initial_value
        }
    }
}

pub trait ContextStoreT: Downcast {
    fn def_id(&self) -> TypeId;
}
impl_downcast!(ContextStoreT);

impl<T: 'static> ContextStoreT for ContextStore<T> {
    fn def_id(&self) -> TypeId {
        self.def_id
    }
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

type Effect = Box<dyn Fn() -> Option<Box<dyn FnOnce() -> ()>>>;

struct EffectStore<Basis: Eq, F: Fn() -> Option<C>, C: FnOnce() -> ()> {
    effect: F,
    cleanup: Option<C>,
    basis: Basis,
    pending_execution: bool
}

trait EffectStoreT: Downcast {
    fn execute(&mut self);
    fn cleanup(&mut self);
    fn is_pending(&self) -> bool;
}
impl_downcast!(EffectStoreT);

impl<T: Eq + 'static, F: Fn() -> Option<C> + 'static, C: FnOnce() -> () + 'static> EffectStoreT for EffectStore<T, F, C> {
    fn execute(&mut self) {
        self.cleanup();
        self.cleanup = (self.effect)();
        self.pending_execution = false;
    }
    fn cleanup(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }

    fn is_pending(&self) -> bool {
        self.pending_execution
    }
}

impl EffectStoreT for () {
    fn execute(&mut self) {
        panic!("Should not")
    }
    fn cleanup(&mut self) {
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
    pub renderer: Rc<RefCell<dyn Renderer>>,
    pub context_link: ContextLink,
    storage_hooks: HookList<Rc<dyn Any>>,
    effect_hooks: HookList<Box<dyn EffectStoreT>>,
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
            renderer,
            context_link,
            storage_hooks: HookList::new(),
            effect_hooks: HookList::new(),
            memo_hooks: HookList::new(),
            has_init: false
        }
    }

    pub fn reset(&mut self) {
        self.storage_hooks.clear();
        self.effect_hooks.clear();
        self.has_init = false;
    }

    fn mark_start_render(&mut self) {
        self.storage_hooks.current_index = 0;
        self.effect_hooks.current_index = 0;
    }

    fn mark_end_render(&mut self) {
        self.has_init = true;
    }

    pub fn use_state<T: 'static>(&mut self, default_value: T) -> Rc<StateHandle<T>> {
        if self.has_init {
            Rc::downcast::<StateHandle<T>>(self.storage_hooks.get().clone()).unwrap()
        } else {
            let handle = Rc::new(StateHandle::new(&self.renderer, default_value));
            self.storage_hooks.hooks.push(handle.clone());
            handle
        }
    }

    pub fn use_context<T>(&mut self, context_def: &'static dyn ContextDef<T>) -> Rc<ContextConsumerHandle<T>> {
        if self.has_init {
            Rc::downcast::<ContextConsumerHandle<T>>(self.storage_hooks.get().clone()).unwrap()
        } else {
            let handle = Rc::new(self.create_context_handle(context_def));
            self.storage_hooks.hooks.push(handle.clone());
            handle
        }
        
    }

    pub fn use_ref<T: 'static>(&mut self) -> Rc<RefCell<Option<T>>> {
        if self.has_init {
            Rc::downcast::<RefCell<Option<T>>>(self.storage_hooks.get().clone()).unwrap()
        } else {
            let handle = Rc::new(RefCell::new(None));
            self.storage_hooks.hooks.push(handle.clone());
            handle
        }
    }

    pub fn use_effect<Basis: Eq + 'static, C: FnOnce() -> () + 'static, F: Fn() -> Option<C> + 'static>(&mut self, effect: F, basis: Basis) {
        if self.has_init {
            let hook_ref = self.effect_hooks.get();
            let mut original_effect = std::mem::replace(hook_ref, Box::new(())).downcast::<EffectStore<Basis, F, C>>().ok().unwrap();
            // web_sys::console::log_1(&JsValue::from(effect.eq(original_effect.effect)));
            if !basis.eq(&original_effect.basis) {
                original_effect.effect = effect;
                original_effect.basis = basis;
                original_effect.pending_execution = true;
            }
            let _ = std::mem::replace(hook_ref, original_effect);
        } else {
            self.effect_hooks.hooks.push(Box::new(EffectStore {
                effect,
                cleanup: None,
                basis,
                pending_execution: true
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

    pub fn use_effect_always<C: FnOnce() -> () + 'static, F: Fn() -> Option<C> + 'static>(&mut self, effect: F) {
        if self.has_init {
            let hook_ref = self.effect_hooks.get();
            let mut original_effect = std::mem::replace(hook_ref, Box::new(())).downcast::<EffectStore<Option<()>, F, C>>().ok().unwrap();
            original_effect.effect = effect;
            original_effect.pending_execution = true;
            let _ = std::mem::replace(hook_ref, original_effect);
        } else {
            self.effect_hooks.hooks.push(Box::new(EffectStore::<Option<()>, F, C> {
                effect,
                cleanup: None,
                basis: None,
                pending_execution: true
            }));
        }
    }

    fn create_context_handle<T>(&self, context_def: &'static dyn ContextDef<T>) -> ContextConsumerHandle<T> {
        let context_link = &self.context_link;
        loop {
            let cl = context_link.as_ref().unwrap();
            let store_copy = cl.context_store.clone();
            let store = cl.context_store.borrow();
            if store.def_id() != context_def.def_id() {
                continue
            }
            if store.downcast_ref::<ContextStore<T>>().is_some() {
                cl.renderers.borrow_mut().push(self.renderer.clone());
                return ContextConsumerHandle {
                    store: store_copy,
                    phantom: std::marker::PhantomData,
                }
            }
        }
    }

    pub fn execute_effects(&mut self) {
        for e in self.effect_hooks.hooks.iter_mut() {
            if e.is_pending() {
                e.execute();
            }
        }
    }

    pub fn cleanup(&mut self) {
        let effect_hooks = std::mem::take(&mut self.effect_hooks.hooks);
        for mut e in effect_hooks.into_iter() {
            e.cleanup();
        }
    }
}


pub trait ComponentDef<VNativeNode> {
    type Props;
    type Ref;
    fn name(&self) -> &'static str;
    fn render(&self, scope: &mut Scope, props: &Self::Props, self_ref: &RefObject<Self::Ref>) -> VNode<VNativeNode>;
}

pub struct VComponentElement<C, VNativeNode> where C: ComponentDef<VNativeNode> + 'static, VNativeNode: 'static {
    pub component_def: &'static C,
    pub props: C::Props,
    pub ref_object: RefObject<C::Ref>
}

pub trait VComponentElementT<VNativeNode>: Downcast {
    fn render(&self, scope: &mut Scope) -> VNode<VNativeNode>;
    fn name(&self) -> &'static str;
    fn same_component(&self, other: &(dyn VComponentElementT<VNativeNode> + 'static)) -> bool;
}
impl_downcast!(VComponentElementT<VNativeNode>);

impl<C: ComponentDef<VNativeNode> + 'static, VNativeNode> VComponentElementT<VNativeNode> for VComponentElement<C, VNativeNode> {
    fn render(&self, scope: &mut Scope) -> VNode<VNativeNode> {
        scope.mark_start_render();
        let result = self.component_def.render(scope, &self.props, &self.ref_object);
        scope.mark_end_render();
        result
    }

    fn name(&self) -> &'static str {
        self.component_def.name()
    }

    fn same_component(&self, other: &(dyn VComponentElementT<VNativeNode> + 'static)) -> bool {
        other.downcast_ref::<VComponentElement<C, VNativeNode>>().is_some()
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
    fn def_id(&self) -> TypeId;
}

impl<VNativeNode, T> VContextT<VNativeNode> for VContext<VNativeNode, T> {
    fn take(self: Box<Self>) -> VContextS<VNativeNode> {
        VContextS {
            store: Rc::new(RefCell::new(self.store)),
            children: self.children
        }
    }
    fn push_value(self: Box<Self>, store: Rc<RefCell<dyn ContextStoreT>>) -> VNode<VNativeNode> {
        let s = store.borrow_mut().downcast_ref::<ContextStore<T>>().unwrap() as *const ContextStore<T>;
        unsafe {
            let ss = s as *mut ContextStore<T>;
            (*ss).value = self.store.value
        };
        *self.children
    }
    fn def_id(&self) -> TypeId {
        self.store.def_id()
    }
}

pub enum VNode<VNativeNode: 'static> {
    Native(VNativeNode),
    Component(Box<dyn VComponentElementT<VNativeNode>>),
    Fragment(Vec<(String, VNode<VNativeNode>)>),
    Context(Box<dyn VContextT<VNativeNode>>),
}

impl<VNativeNode> VNode<VNativeNode> {
    fn component<C>(element: VComponentElement<C, VNativeNode>) -> VNode<VNativeNode> where C: ComponentDef<VNativeNode> + 'static, VNativeNode: 'static {
        VNode::Component(Box::new(
            element
        ))
    }
}

pub fn h<T, VNativeNode>(component_def: &'static T, props: T::Props, ref_object: RefObject<T::Ref>) -> VNode<VNativeNode>
    where
        T: ComponentDef<VNativeNode> + 'static,
        VNativeNode: 'static {
    VNode::component(VComponentElement::<T, VNativeNode> {
            component_def,
            props: props,
            ref_object
        })
}

pub fn ct<T, VNativeNode: 'static>(context_def: &'static dyn ContextDef<T>, value: T, children: VNode<VNativeNode>) -> VNode<VNativeNode> {
    VNode::Context(Box::new(VContext {
        store: ContextStore {
            def_id: context_def.def_id(),
            value,
        },
        children: Box::new(children),
    }))
}