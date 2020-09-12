use crate::v_node::{ContextDef, ContextStore};
use std::any::TypeId;

#[derive(Clone, PartialEq)]
pub enum StyleType {
    Dark,
    Light
}

pub static def: ContextDef<StyleType> = std::marker::PhantomData;
