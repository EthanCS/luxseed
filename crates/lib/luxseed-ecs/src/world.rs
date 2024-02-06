use crate::{
    archetype::Archetypes,
    bundle::Bundle,
    component::{Component, ComponentId, Components},
    entity::{Entities, Entity, EntityLocation},
};

pub struct World {
    pub(crate) entities: Entities,
    pub(crate) components: Components,
    pub(crate) archetypes: Archetypes,
}

impl World {
    pub fn new() -> Self {
        todo!()
    }

    pub fn spawn(&mut self, components: impl Bundle) -> Entity {
        todo!()
    }

    fn spawn_inner(&mut self, entity: Entity, components: impl Bundle) {
        todo!()
    }

    #[inline]
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        todo!()
    }

    #[inline]
    pub fn components(&self) -> &Components {
        &self.components
    }

    #[inline]
    pub fn init_component<T: Component>(&mut self) -> ComponentId {
        self.components.init_component::<T>()
    }

    #[inline]
    pub fn component_id<T: Component>(&self) -> Option<ComponentId> {
        self.components.component_id::<T>()
    }
}
