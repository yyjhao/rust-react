use crate::v_node::{ContextDef, ContextStore};
use std::any::TypeId;


pub struct ContextType {
    pub name: String,
    pub count: usize
}

pub struct Def {
}

pub static DEF: Def = Def {};

impl ContextDef<ContextType> for Def {
    fn def_id(&self) -> TypeId {
        TypeId::of::<Def>()
    }

    fn make_context(&self, initial_value: ContextType) -> ContextStore<ContextType> {
        ContextStore::new(self.def_id(), initial_value)
    }
}
