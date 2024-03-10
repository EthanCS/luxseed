// use crate::{
//     archetype::Archetypes,
//     bundle::Bundle,
//     component::{Component, ComponentId, Components},
//     entity::{Entities, Entity, EntityLocation},
// };

use crate::entity::{Entities, Entity};

pub struct World {
    entities: Entities,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::default(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = self.entities.alloc_entity();
        entity
    }
}
