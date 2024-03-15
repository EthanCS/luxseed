use std::{marker::PhantomData, ptr::NonNull};

use crate::{archetype::Archetype, component::Component, entity::Entity};

pub struct EntityRef<'a> {
    entity: Entity,
    archetype: &'a Archetype,
    index: u32,
}

impl<'a> EntityRef<'a> {
    pub(crate) fn new(entity: Entity, archetype: &'a Archetype, index: u32) -> Self {
        Self {
            entity,
            archetype,
            index,
        }
    }

    #[inline]
    pub fn get<T: Component>(&self) -> Option<&'a T> {
        todo!()
    }
}

pub struct Ref<'a, T: ?Sized> {
    target: NonNull<T>,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<T: ?Sized + Sync> Send for Ref<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for Ref<'_, T> {}

pub trait ComponentRef<'a> {
    type Ref;
}

impl<'a, T: Component> ComponentRef<'a> for &'a T {
    type Ref = &'a T;
}
