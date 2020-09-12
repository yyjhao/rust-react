use crate::v_node::{ContextDef, ContextStore};
use std::any::TypeId;


pub struct ContextType {
    pub name: String,
    pub count: usize
}

pub static def: ContextDef<ContextType> = std::marker::PhantomData;
