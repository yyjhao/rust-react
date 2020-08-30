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

pub struct Scope {
    pub renderer: Rc<RefCell<dyn Renderer>>,
    pub context_link: ContextLink,
    hooks: Vec<Rc<dyn Any>>,
    current_hook_index: usize,
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
            hooks: vec![],
            current_hook_index: 0,
            has_init: false
        }
    }

    pub fn reset(&mut self) {
        self.hooks.clear();
        self.current_hook_index = 0;
        self.has_init = false;
    }

    fn mark_start_render(&mut self) {
        self.current_hook_index = 0;
    }

    fn mark_end_render(&mut self) {
        self.has_init = true;
    }

    pub fn use_state<T: 'static>(&mut self, default_value: T) -> Rc<StateHandle<T>> {
        if self.has_init {
            self.current_hook_index += 1;
            Rc::downcast::<StateHandle<T>>(self.hooks.get(self.current_hook_index - 1).unwrap().clone()).unwrap()
        } else {
            let handle = Rc::new(StateHandle::new(&self.renderer, default_value));
            self.hooks.push(handle.clone());
            handle
        }
    }

    pub fn use_context<T>(&mut self, context_def: &'static dyn ContextDef<T>) -> Rc<ContextConsumerHandle<T>> {
        if self.has_init {
            self.current_hook_index += 1;
            Rc::downcast::<ContextConsumerHandle<T>>(self.hooks.get(self.current_hook_index - 1).unwrap().clone()).unwrap()
        } else {
            let handle = Rc::new(self.create_context_handle(context_def));
            self.hooks.push(handle.clone());
            handle
        }
        
    }

    pub fn use_ref<T: 'static>(&mut self) -> Rc<RefCell<Option<T>>> {
        if self.has_init {
            self.current_hook_index += 1;
            Rc::downcast::<RefCell<Option<T>>>(self.hooks.get(self.current_hook_index - 1).unwrap().clone()).unwrap()
        } else {
            let handle = Rc::new(RefCell::new(None));
            self.hooks.push(handle.clone());
            handle
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