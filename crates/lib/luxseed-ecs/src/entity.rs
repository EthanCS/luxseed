// use crate::{
//     archetype::{ArchetypeId, ArchetypeRow},
//     storage::{TableId, TableRow},
// };
// use std::sync::atomic::AtomicI64;
// use std::{fmt, num::NonZeroU64};

// /// EntityId is a unique identifier for an entity.
// /// <- 46 index -> <- 16 gen -> <- 2 meta ->
// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[repr(transparent)]
// pub struct EntityId(pub(super) NonZeroU64);

// impl EntityId {
    
// }

// #[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
// pub struct Entity {
//     index: u32,
//     generation: NonZeroU32,
// }

// impl fmt::Debug for Entity {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Entity({:?}, {:?})", self.index, self.generation)
//     }
// }

// impl Entity {
//     #[inline(always)]
//     pub fn from_index(index: u32) -> Self {
//         Self::from_index_and_generation(index, NonZeroU32::MIN)
//     }

//     #[inline(always)]
//     pub fn from_index_and_generation(index: u32, generation: NonZeroU32) -> Self {
//         Self { index, generation }
//     }

//     #[inline]
//     pub const fn index(&self) -> u32 {
//         self.index
//     }
// }

// #[derive(Debug)]
// pub struct Entities {
//     meta: Vec<EntityMeta>,
//     pending: Vec<u32>,
//     free_cursor: AtomicI64,
//     len: u32,
// }

// impl Entities {
//     pub(crate) fn new() -> Self {
//         Self { meta: Vec::new(), pending: Vec::new(), free_cursor: AtomicI64::new(0), len: 0 }
//     }

//     pub fn alloc(&mut self) -> Entity {
//         self.len += 1;
//         if let Some(index) = self.pending.pop() {
//             let meta = &mut self.meta[index as usize];
//             let generation = meta.generation;
//             meta.location = EntityLocation::INVALID;
//             return Entity { index, generation };
//         } else {
//             let index = u32::try_from(self.meta.len()).expect("entity count overflow");
//             self.meta.push(EntityMeta::EMPTY);
//             return Entity::from_index(index);
//         }
//     }

//     pub fn free(&mut self, entity: Entity) -> Option<EntityLocation> {
//         let meta = &self.meta[entity.index() as usize];
//         if meta.generation != entity.generation {
//             return None;
//         }
//         todo!()
//     }
// }

// #[derive(Copy, Clone, Debug)]
// #[repr(C)]
// struct EntityMeta {
//     pub generation: NonZeroU32,
//     pub location: EntityLocation,
// }

// impl EntityMeta {
//     /// Only used for the initial state of pending entity
//     const EMPTY: Self = Self { generation: NonZeroU32::MIN, location: EntityLocation::INVALID };
// }

// #[derive(Copy, Clone, Debug)]
// #[repr(C)]
// pub struct EntityLocation {
//     pub archetype_id: ArchetypeId,
//     pub archetype_row: ArchetypeRow,
//     pub table_id: TableId,
//     pub table_row: TableRow,
// }

// impl EntityLocation {
//     pub const INVALID: Self = Self {
//         archetype_id: ArchetypeId::INVALID,
//         archetype_row: ArchetypeRow::INVALID,
//         table_id: TableId::INVALID,
//         table_row: TableRow::INVALID,
//     };
// }

// #[derive(Debug)]
// pub struct EntityManager {}

// impl EntityManager {
//     pub fn create_entity(&mut self) -> Entity {
//         todo!()
//     }
// }
