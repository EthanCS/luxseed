use crate::EcsError;
use std::{
    mem,
    num::NonZeroU32,
    sync::atomic::{AtomicIsize, Ordering},
};

#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: NonZeroU32,
}

#[derive(Copy, Clone)]
pub(crate) struct EntityLocation {
    pub archetype: u32,
    pub index: u32,
}

#[derive(Copy, Clone)]
pub(crate) struct EntityMeta {
    pub generation: NonZeroU32,
    pub location: EntityLocation,
}

impl EntityMeta {
    const EMPTY: EntityMeta = EntityMeta {
        generation: match NonZeroU32::new(1) {
            Some(x) => x,
            None => unreachable!(),
        },
        location: EntityLocation {
            archetype: 0,
            index: u32::MAX,
        },
    };
}

#[derive(Default)]
pub(crate) struct Entities {
    pub metas: Vec<EntityMeta>,
    pending: Vec<u32>,
    free_cursor: AtomicIsize,
    len: u32,
}

impl Entities {
    /// Directly allocate a new entity. Make sure `flush_reserved_entities` has been called before if you reserve entity.
    pub fn alloc_entity(&mut self) -> Entity {
        self.verify_flushed();

        self.len += 1;
        if let Some(id) = self.pending.pop() {
            let new_free_cursor = self.pending.len() as isize;
            *self.free_cursor.get_mut() = new_free_cursor;
            Entity {
                generation: self.metas[id as usize].generation,
                id,
            }
        } else {
            let id = u32::try_from(self.metas.len()).expect("too many entities");
            self.metas.push(EntityMeta::EMPTY);
            Entity {
                generation: NonZeroU32::new(1).unwrap(),
                id,
            }
        }
    }

    /// Free an entity. Make sure `flush_reserved_entities` has been called before if you reserve entity.
    ///
    /// Returns the memory location of the freed entity.
    pub fn free_entity(&mut self, entity: Entity) -> Result<EntityLocation, EcsError> {
        self.verify_flushed();

        let meta = self
            .metas
            .get_mut(entity.id as usize)
            .ok_or(EcsError::EntityNotFound)?;
        if meta.generation != entity.generation || meta.location.index == u32::MAX {
            return Err(EcsError::EntityNotFound);
        }

        // Replace the old location with an empty one and return the old location
        let old = mem::replace(&mut meta.location, EntityMeta::EMPTY.location);

        // Increment the generation to invalidate any references to this entity
        meta.generation = NonZeroU32::new(u32::from(meta.generation).wrapping_add(1))
            .unwrap_or_else(|| NonZeroU32::new(1).unwrap());

        self.pending.push(entity.id);
        let new_free_cursor = self.pending.len() as isize;
        *self.free_cursor.get_mut() = new_free_cursor;
        self.len -= 1;

        Ok(old)
    }

    /// Reserve a single entity for later allocation by using `flush_reserved_entities`.
    pub fn reserve_entity(&self) -> Entity {
        let n = self.free_cursor.fetch_sub(1, Ordering::Relaxed);
        if n > 0 {
            let id = self.pending[(n - 1) as usize];
            Entity {
                generation: self.metas[id as usize].generation,
                id,
            }
        } else {
            let id = u32::try_from(self.metas.len() as isize - n).expect("too many entities");
            Entity {
                generation: NonZeroU32::new(1).unwrap(),
                id,
            }
        }
    }

    /// Flush all reserved entities, and call `init` for each of them to initialize their memory location.
    pub fn flush_reserved_entities(&mut self, mut init: impl FnMut(u32, &mut EntityLocation)) {
        let free_cursor = *self.free_cursor.get_mut();

        let new_free_cursor = if free_cursor >= 0 {
            free_cursor as usize
        } else {
            // negative free_cursor means how many more we have to allocate
            let old_meta_len = self.metas.len();
            let new_meta_len = old_meta_len + -free_cursor as usize;
            self.metas.resize(new_meta_len, EntityMeta::EMPTY);

            self.len += -free_cursor as u32;
            for (id, meta) in self.metas.iter_mut().enumerate().skip(old_meta_len) {
                init(id as u32, &mut meta.location);
            }

            *self.free_cursor.get_mut() = 0;
            0
        };

        self.len += (self.pending.len() - new_free_cursor) as u32;
        for id in self.pending.drain(new_free_cursor..) {
            init(id, &mut self.metas[id as usize].location);
        }
    }

    /// Reserve entity meta space for `additional` entities.
    ///
    /// Make sure `flush_reserved_entities` has been called before if you reserve entity.
    pub fn reserve(&mut self, additional: u32) {
        self.verify_flushed();

        let freelist_size = *self.free_cursor.get_mut();
        let shortfall = additional as isize - freelist_size;
        if shortfall > 0 {
            self.metas.reserve(shortfall as usize);
        }
    }

    /// Reserved entity still counts
    pub fn contains(&self, entity: Entity) -> bool {
        match self.metas.get(entity.id as usize) {
            Some(meta) => {
                meta.generation == entity.generation
                    && (meta.location.index != u32::MAX
                        || self.pending[self.free_cursor.load(Ordering::Relaxed).max(0) as usize..]
                            .contains(&entity.id))
            }
            None => {
                // Check if this could have been obtained from `reserve_entity`
                let free = self.free_cursor.load(Ordering::Relaxed);
                entity.generation.get() == 1
                    && free < 0
                    && (entity.id as isize) < (free.abs() + self.metas.len() as isize)
            }
        }
    }

    /// Get the memory location of an entity.
    ///
    /// Return `(0, u32::MAX)` if the entity is reserved but not allocated.
    pub fn get(&self, entity: Entity) -> Result<EntityLocation, EcsError> {
        if self.metas.len() <= entity.id as usize {
            // Check if this could have been obtained from `reserve_entity`
            let free = self.free_cursor.load(Ordering::Relaxed);
            if entity.generation.get() == 1
                && free < 0
                && (entity.id as isize) < (free.abs() + self.metas.len() as isize)
            {
                return Ok(EntityLocation {
                    archetype: 0,
                    index: u32::max_value(),
                });
            } else {
                return Err(EcsError::EntityNotFound);
            }
        }

        let meta = self.metas[entity.id as usize];
        if meta.generation != entity.generation || meta.location.index == u32::MAX {
            return Err(EcsError::EntityNotFound);
        }
        Ok(meta.location)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Result<&mut EntityLocation, EcsError> {
        let meta = self
            .metas
            .get_mut(entity.id as usize)
            .ok_or(EcsError::EntityNotFound)?;
        if meta.generation != entity.generation || meta.location.index == u32::MAX {
            return Err(EcsError::EntityNotFound);
        }
        Ok(&mut meta.location)
    }

    pub fn clear(&mut self) {
        self.metas.clear();
        self.pending.clear();
        *self.free_cursor.get_mut() = 0;
        self.len = 0;
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    fn verify_flushed(&mut self) {
        debug_assert!(
            !self.needs_flush(),
            "flush() needs to be called before this operation is legal"
        );
    }

    fn needs_flush(&mut self) -> bool {
        *self.free_cursor.get_mut() != self.pending.len() as isize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn alloc_and_free() {
        let mut rng = StdRng::seed_from_u64(0xFEEDFACEDEADF00D);

        let mut e = Entities::default();
        let mut first_unused = 0u32;
        let mut id_to_gen: HashMap<u32, u32> = Default::default();
        let mut free_set: HashSet<u32> = Default::default();
        let mut len = 0;

        for _ in 0..100 {
            let alloc = rng.gen_bool(0.7);
            if alloc || first_unused == 0 {
                let entity = e.alloc_entity();
                e.metas[entity.id as usize].location.index = 0;
                len += 1;

                let id = entity.id;
                if !free_set.is_empty() {
                    // This should have come from the freelist.
                    assert!(free_set.remove(&id));
                } else if id >= first_unused {
                    first_unused = id + 1;
                }

                e.get_mut(entity).unwrap().index = 37;

                assert!(id_to_gen.insert(id, entity.generation.get()).is_none());
            } else {
                // Free a random ID, whether or not it's in use, and check for errors.
                let id = rng.gen_range(0..first_unused);

                let generation = id_to_gen.remove(&id);
                let entity = Entity {
                    id,
                    generation: NonZeroU32::new(
                        generation.unwrap_or_else(|| NonZeroU32::new(1).unwrap().get()),
                    )
                    .unwrap(),
                };

                assert_eq!(e.free_entity(entity).is_ok(), generation.is_some());
                if generation.is_some() {
                    len -= 1;
                }

                free_set.insert(id);
            }
            assert_eq!(e.len(), len);
        }
    }
}
