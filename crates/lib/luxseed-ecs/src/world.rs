// use crate::{
//     archetype::Archetypes,
//     bundle::Bundle,
//     component::{Component, ComponentId, Components},
//     entity::{Entities, Entity, EntityLocation},
// };

use crate::component::Component;

pub struct World {}

impl World {
    fn new() -> Self {
        todo!()
    }

    fn register<T: Component>(&mut self)
    where
        T::Storage: Default,
    {
        self.register_with_storage::<T, _>(Default::default);
    }

    fn register_with_storage<T: Component, F>(&mut self, storage: F)
    where
        F: FnOnce() -> T::Storage,
    {
        todo!()
    }
}
