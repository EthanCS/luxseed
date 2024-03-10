use std::{any::TypeId, collections::HashMap};

#[derive(Copy, Clone)]
pub struct TypeMeta {
    type_id: TypeId,
}

pub struct Archetype {
    types: Vec<TypeMeta>,
}

impl Archetype {
    pub(crate) fn new(types: Vec<TypeMeta>) -> Self {
        Self { types }
    }
}

pub struct Archetypes {
    // sorted list of component types, and the index of the archetype
    mapping: HashMap<Box<[TypeId]>, u32>,
    archetypes: Vec<Archetype>,
}

impl Archetypes {
    pub fn new() -> Self {
        Self {
            // first create zero type archetype
            mapping: Some((Box::default(), 0)).into_iter().collect(),
            archetypes: vec![Archetype::new(Vec::new())],
        }
    }
}
