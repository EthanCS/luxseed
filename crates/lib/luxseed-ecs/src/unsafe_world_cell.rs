use std::{cell::UnsafeCell, marker::PhantomData};

use crate::world::World;

#[derive(Copy, Clone)]
pub struct UnsafeWorldCell<'a>(*mut World, PhantomData<(&'a World, &'a UnsafeCell<World>)>);

impl<'a> UnsafeWorldCell<'a> {
    #[inline]
    pub(crate) fn new_readonly(world: &'a World) -> Self {
        UnsafeWorldCell(world as *const World as *mut World, PhantomData)
    }

    #[inline]
    pub(crate) fn new_mutable(world: &'a mut World) -> Self {
        UnsafeWorldCell(world as *mut World, PhantomData)
    }
}
