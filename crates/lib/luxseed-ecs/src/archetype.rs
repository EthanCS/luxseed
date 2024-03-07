// use crate::{
//     component::{Component, ComponentId},
//     entity::Entity,
//     storage::{TableId, TableRow},
// };

// #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
// pub struct ArchetypeId(u32);

// impl ArchetypeId {
//     /// The empty archetype is the archetype that contains no components.
//     pub const EMPTY: Self = Self(0);
//     pub const INVALID: Self = Self(u32::MAX);

//     #[inline]
//     pub const fn new(index: usize) -> Self {
//         Self(index as u32)
//     }

//     #[inline]
//     pub const fn index(&self) -> usize {
//         self.0 as usize
//     }
// }

// /// Archetype is a collection of entities with the same set of components.
// pub struct Archetype {
//     id: ArchetypeId,
//     //types: Vec<ComponentType>,
//     len: u32,
// }

// impl Archetype {
//     // pub(crate) fn new(types: Vec<ComponentType>) -> Self {
//     //     todo!()
//     // }

//     // fn assert_types_order(types: &[ComponentType]) {
//     //     todo!()
//     // }

//     #[inline]
//     pub fn len(&self) -> u32 {
//         self.len
//     }

//     #[inline]
//     pub fn is_empty(&self) -> bool {
//         self.len == 0
//     }

//     pub fn has<T: Component>(&self) -> bool {
//         todo!()
//     }
// }

// #[derive(Debug, Copy, Clone, Eq, PartialEq)]
// pub struct ArchetypeRow(u32);

// impl ArchetypeRow {
//     pub const INVALID: Self = Self(u32::MAX);

//     #[inline]
//     pub const fn new(index: usize) -> Self {
//         Self(index as u32)
//     }

//     #[inline]
//     pub const fn index(&self) -> usize {
//         self.0 as usize
//     }
// }

// pub struct ArchetypeEntity {
//     entity: Entity,
//     table_row: TableRow,
// }

// impl ArchetypeEntity {
//     #[inline]
//     pub const fn id(&self) -> Entity {
//         self.entity
//     }

//     #[inline]
//     pub const fn table_row(&self) -> TableRow {
//         self.table_row
//     }
// }

// pub struct Archetypes {
//     archetypes: Vec<Archetype>,
// }

// impl Archetypes {
//     pub(crate) fn new() -> Self {
//         let mut archetypes = Self { archetypes: Vec::new() };
//         archetypes.get_or_add_archetype(TableId::EMPTY, Vec::new());
//         archetypes
//     }

//     pub fn get_or_add_archetype(
//         &mut self,
//         table_id: TableId,
//         components: Vec<ComponentId>,
//     ) -> ArchetypeId {
//         todo!()
//     }
// }
