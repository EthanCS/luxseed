use std::collections::HashMap;

use crate::{component::ComponentId, entity::Entity};

/// Column is a collection of components of the same type. It is a part of an archetype.
pub struct Column {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//#[repr(transparent)]
pub struct TableId(u32);

impl TableId {
    /// The empty archetype is the archetype that contains no components.
    pub const EMPTY: Self = Self(0);
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn from_usize(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn from_u32(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableRow(u32);

impl TableRow {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn from_usize(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn from_u32(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

pub struct Table {
    columns: HashMap<ComponentId, Column>,
    entities: Vec<Entity>,
}

impl Table {
    /// Returns a slice of entities in the table.
    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    #[inline]
    pub fn get_column(&self, component_id: ComponentId) -> Option<&Column> {
        self.columns.get(&component_id)
    }

    #[inline]
    pub fn get_column_mut(&mut self, component_id: ComponentId) -> Option<&mut Column> {
        self.columns.get_mut(&component_id)
    }

    #[inline]
    pub fn has_column(&self, component_id: ComponentId) -> bool {
        self.columns.contains_key(&component_id)
    }

    pub fn alloc(&mut self, entity: Entity) -> TableRow {
        todo!()
    }
}

pub struct Tables {
    tables: Vec<Table>,
    table_ids: HashMap<Vec<ComponentId>, usize>,
}
