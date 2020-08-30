use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::any::TypeId;
use wasm_bindgen::prelude::*;

use downcast_rs::Downcast;

pub type RefObject<T> = Rc<RefCell<Option<T>>>;

pub fn make_ref_object<T>(initial_value: Option<T>) -> RefObject<T> {
    Rc::new(RefCell::new(initial_value))
}

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
    pub context_link: ContextLink
}

impl Scope {
    pub fn use_state<T>(&self, default_value: T) -> StateHandle<T> {
        StateHandle::new(&self.renderer, default_value)
    }

    pub fn use_context<T>(&self, context_def: &'static dyn ContextDef<T>) -> ContextConsumerHandle<T> {
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


pub trait Component<VNativeNode> {
    type Props;
    type Ref;
    fn def(&self) -> &'static dyn ComponentDef<VNativeNode, Self>;
    fn render(self: Rc<Self>, props: &Self::Props, self_ref: &RefObject<Self::Ref>) -> VNode<VNativeNode>;
}

pub trait ComponentDef<VNativeNode, C: Component<VNativeNode>> {
    fn name(&self) -> &'static str;
    fn make(&self, scope: &mut Scope) -> Rc<C>;
}

pub struct VComponentElement<C, VNativeNode> where C: Component<VNativeNode> + 'static, VNativeNode: 'static {
    pub component_def: &'static dyn ComponentDef<VNativeNode, C>,
    pub props: C::Props,
    pub ref_object: RefObject<C::Ref>
}

pub trait ComponentT<VNativeNode>: Downcast {
}
impl_downcast!(ComponentT<VNativeNode>);

pub trait VComponentElementT<VNativeNode> {
    fn make(&self, scope: &mut Scope) -> Rc<dyn ComponentT<VNativeNode>>;
    fn render(&self, component: Rc<dyn ComponentT<VNativeNode>>) -> VNode<VNativeNode>;
    fn name(&self) -> &'static str;
    fn component_compatible(&self, component: &dyn ComponentT<VNativeNode>) -> bool;
}

impl<C: Component<VNativeNode> + 'static, VNativeNode> ComponentT<VNativeNode> for C {
}

impl<C: Component<VNativeNode>  + 'static, VNativeNode> VComponentElementT<VNativeNode> for VComponentElement<C, VNativeNode> {
    fn make(&self, scope: &mut Scope) -> Rc<dyn ComponentT<VNativeNode>> {
        self.component_def.make(scope)
    }

    fn render(&self, component: Rc<dyn ComponentT<VNativeNode>>) -> VNode<VNativeNode> {
        component.downcast_rc::<C>().map_err(|_| "Shouldn't happen.").unwrap().render(&self.props, &self.ref_object)
    }

    fn name(&self) -> &'static str {
        self.component_def.name()
    }

    fn component_compatible(&self, component: &dyn ComponentT<VNativeNode>) -> bool {
        component.downcast_ref::<C>().is_some()
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
    fn component<C>(element: VComponentElement<C, VNativeNode>) -> VNode<VNativeNode> where C: Component<VNativeNode> + 'static, VNativeNode: 'static {
        VNode::Component(Box::new(
            element
        ))
    }
}

pub fn h<T, C, Props, Ref, VNativeNode>(component_def: &'static T, props: Props, ref_object: RefObject<Ref>) -> VNode<VNativeNode>
    where
        C: Component<VNativeNode, Props = Props, Ref = Ref> + 'static + ComponentT<VNativeNode>,
        T: ComponentDef<VNativeNode, C> + 'static,
        Props: 'static,
        VNativeNode: 'static {
    VNode::component(VComponentElement::<C, VNativeNode> {
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