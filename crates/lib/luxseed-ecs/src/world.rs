use crate::{
    archetype::Archetypes,
    component::{Component, ComponentBundle},
    entity::{Entities, Entity, EntityLocation},
    entity_ref::EntityRef,
    EcsError,
};

pub struct World {
    entities: Entities,
    archetypes: Archetypes,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::default(),
            archetypes: Archetypes::new(),
        }
    }

    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, EcsError> {
        let loc = self.entities.get(entity)?;
        Ok(EntityRef::new(
            entity,
            &self.archetypes.archetypes[loc.archetype as usize],
            loc.index,
        ))
    }

    pub fn spawn_entity(&mut self, bundle: impl ComponentBundle) -> Entity {
        self.flush();
        let entity = self.entities.alloc_entity();
        self.alloc_storage(entity, bundle);
        entity
    }

    fn alloc_storage(&mut self, entity: Entity, bundle: impl ComponentBundle) {
        let archetype_id: u32 = 0;

        let archetype = &mut self.archetypes.archetypes[archetype_id as usize];
        let index = archetype.alloc(entity.id);
        self.entities.metas[entity.id as usize].location = EntityLocation {
            archetype: archetype_id,
            index,
        };
        todo!();
    }

    pub fn reserve_entity(&self) -> Entity {
        self.entities.reserve_entity()
    }

    pub fn contain_entity(&self, entity: Entity) -> bool {
        self.entities.contains(entity)
    }

    pub fn destroy(&mut self, entity: Entity) -> Result<(), EcsError> {
        self.flush();
        let loc = self.entities.free_entity(entity)?;
        if let Some(moved) =
            self.archetypes.archetypes[loc.archetype as usize].free(loc.index, true)
        {
            self.entities.metas[moved as usize].location.index = loc.index;
        }
        Ok(())
    }

    pub fn add_component<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EcsError> {
        todo!()
    }

    pub fn flush(&mut self) {
        todo!()
    }
}
