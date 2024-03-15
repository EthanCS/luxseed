use std::{alloc::Layout, any::TypeId, collections::HashMap, vec};

use crate::component::Component;

const ENTITY_MIN_INCREMENT: u32 = 64;

pub struct Archetype {
    types: Vec<TypeMeta>,
    type_ids: Box<[TypeId]>,
    len: u32,
    entities: Box<[u32]>,
}

impl Archetype {
    pub(crate) fn new(types: Vec<TypeMeta>) -> Self {
        Self::assert_types(&types);
        Self {
            type_ids: types.iter().map(|ty| ty.type_id()).collect(),
            types,
            len: 0,
            entities: Box::new([]),
        }
    }

    pub fn has<T: Component>(&self) -> bool {
        self.type_ids.contains(&TypeId::of::<T>())
    }

    /// Allocates a new entity in the archetype and returns the index
    pub(crate) fn alloc(&mut self, entity_id: u32) -> u32 {
        if self.len() == self.capacity() {
            self.grow(ENTITY_MIN_INCREMENT);
        }
        self.entities[self.len as usize] = entity_id;
        self.len += 1;
        self.len - 1
    }

    pub(crate) fn free(&mut self, index: u32, drop: bool) -> Option<u32> {
        todo!()
    }

    pub(crate) fn reserve(&mut self, additional: u32) {
        if self.len() + additional > self.capacity() {
            let increment = additional - (self.capacity() - self.len());
            self.grow(increment.max(ENTITY_MIN_INCREMENT));
        }
    }

    pub(crate) fn grow(&mut self, increment: u32) {
        self.grow_exact(self.capacity().max(increment));
    }

    fn grow_exact(&mut self, increment: u32) {
        let old_cap = self.capacity();
        let new_cap = old_cap + increment;
        let mut new_entities = vec![!0; new_cap as usize].into_boxed_slice();
        new_entities[..self.len() as usize].copy_from_slice(&self.entities);
        self.entities = new_entities;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    fn capacity(&self) -> u32 {
        self.entities.len() as u32
    }

    fn assert_types(types: &[TypeMeta]) {
        //todo!()
    }
}

pub(crate) struct Archetypes {
    // sorted list of component types, and the index of the archetype
    pub mapping: HashMap<Box<[TypeId]>, u32>,
    pub archetypes: Vec<Archetype>,
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
