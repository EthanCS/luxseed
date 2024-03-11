use std::{alloc::Layout, any::TypeId, collections::HashMap};

#[derive(Debug, Copy, Clone)]
pub struct TypeMeta {
    type_id: TypeId,
    type_name: &'static str,
    layout: Layout,
    drop_fn: unsafe fn(*mut u8),
}

impl TypeMeta {
    pub fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            layout: Layout::new::<T>(),
            drop_fn: |ptr| unsafe { std::ptr::drop_in_place(ptr as *mut T) },
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn drop_fn(&self) -> unsafe fn(*mut u8) {
        self.drop_fn
    }
}

impl PartialEq for TypeMeta {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for TypeMeta {}

impl PartialOrd for TypeMeta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TypeMeta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.layout
            .align()
            .cmp(&other.layout.align())
            .reverse()
            .then_with(|| self.type_id.cmp(&other.type_id))
    }
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
