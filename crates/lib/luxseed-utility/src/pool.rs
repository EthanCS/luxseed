use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    index: u16,
    generation: u16,
    pd: PhantomData<T>,
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self { index: u16::MAX, generation: u16::MAX, pd: PhantomData::<T>::default() }
    }
}

pub trait Handled {
    type HandleType;

    fn get_handle(&self) -> Option<Handle<Self::HandleType>>;
    fn set_handle(&mut self, handle: Option<Handle<Self::HandleType>>);
}

#[macro_export]
macro_rules! impl_handle {
    ($type:ty, $handle_type:ty, $handle:ident) => {
        impl luxseed_utility::pool::Handled for $type {
            type HandleType = $handle_type;

            fn get_handle(&self) -> Option<Handle<Self::HandleType>> {
                self.$handle
            }

            fn set_handle(&mut self, handle: Option<Handle<Self::HandleType>>) {
                self.$handle = handle;
            }
        }
    };
}

pub struct Pool<T: Handled>
where
    T::HandleType: Copy,
{
    items: Vec<T>,
    generations: Vec<u16>,
    free_indices: Vec<u16>,
    fp_init: Box<dyn Fn() -> T>,
}

impl<T: Handled> Pool<T>
where
    T::HandleType: Copy,
{
    pub fn with_size<F>(size: usize, init: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        if size > u16::MAX as usize {
            panic!("{} exceeds max size", std::any::type_name::<Self>());
        }

        let mut items = Vec::with_capacity(size);
        items.resize_with(size, || init());

        let mut free_indices = Vec::with_capacity(size);
        for i in 0..size {
            free_indices.push(i as u16);
        }
        free_indices.reverse();

        let mut generations = Vec::with_capacity(size);
        for _ in 0..size {
            generations.push(0);
        }

        Self { items, generations, free_indices, fp_init: Box::new(init) }
    }

    #[inline]
    pub fn get(&self, handle: Handle<T::HandleType>) -> Option<&T> {
        if self.is_match(handle) {
            return self.items.get(handle.index as usize);
        }
        None
    }

    #[inline]
    pub fn get_mut(&mut self, handle: Handle<T::HandleType>) -> Option<&mut T> {
        if self.is_match(handle) {
            return self.items.get_mut(handle.index as usize);
        }
        None
    }

    #[inline]
    pub fn is_match(&self, handle: Handle<T::HandleType>) -> bool {
        if let Some(gen) = self.generations.get(handle.index as usize) {
            if *gen == handle.generation {
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn free(&mut self, handle: Handle<T::HandleType>) {
        if self.is_match(handle) {
            self.items.get_mut(handle.index as usize).unwrap().set_handle(None);
            self.free_indices.push(handle.index);
            self.generations[handle.index as usize] += 1;
        }
    }

    pub fn malloc(&mut self) -> (Handle<T::HandleType>, &mut T) {
        if let Some(index) = self.free_indices.pop() {
            let handle: Handle<T::HandleType> =
                Handle { index, generation: self.generations[index as usize], pd: PhantomData };
            let item = self.items.get_mut(handle.index as usize).unwrap();
            item.set_handle(Some(handle));
            return (handle, item);
        } else {
            self.double_size();
            return self.malloc();
        }
    }

    fn double_size(&mut self) {
        let old_len = self.generations.len();
        if old_len == u16::MAX as usize {
            panic!("Pool is already at max size");
        }

        let new_len = std::cmp::min(old_len * 2, u16::MAX as usize);
        for i in old_len..new_len {
            self.free_indices.push(i as u16);
        }
        self.free_indices.reverse();
        self.generations.resize(new_len, 0);
        self.items.resize_with(new_len, || (self.fp_init)());
    }
}

#[macro_export]
macro_rules! define_resource_pool {
    ($name_pool:ident,$(($type:ty, $pool_name:ident, $default_size:expr)),*) => {
        pub struct $name_pool {
            $(
                pub $pool_name: Pool<$type>,
            )*
        }

        impl $name_pool {
            pub fn new() -> Self {
                Self {
                    $(
                        $pool_name: Pool::with_size($default_size, || <$type>::default()),
                    )*
                }
            }
        }
    };
}

// pub struct PoolSOA<T: StructOfArray + Handled>
// where
//     T::HandleType: Copy,
// {
//     items: T,
//     inner: PoolInner<T::HandleType>,
//     fp_init: Box<dyn Fn() -> T::Item>,
// }

// impl<T: StructOfArray + Handled> PoolSOA<T>
// where
//     T::HandleType: Copy,
// {
//     impl_pool_common!();

//     pub fn with_size<F>(size: usize, init: F) -> Self
//     where
//         F: Fn() -> T::Item + 'static,
//     {
//         if size > u16::MAX as usize {
//             panic!("{} exceeds max size", std::any::type_name::<Self>());
//         }

//         let mut items = <T as StructOfArray>::with_capacity(size);
//         items.resize_with(size, || init());

//         Self {
//             items,
//             inner: PoolInner::with_size(size),
//             fp_init: Box::new(init),
//         }
//     }

//     pub fn get(&self, handle: Handle<T::HandleType>) -> Option<&T::Item> {
//         if self.is_match(handle) {
//             return self.items.get(handle.index as usize);
//         }
//         None
//     }

//     pub fn get_mut(&mut self, handle: Handle<T::HandleType>) -> Option<&mut T::Item> {
//         if self.is_match(handle) {
//             return self.items.get_mut(handle.index as usize);
//         }
//         None
//     }
// }

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::define_rhi_resource;

    // define_rhi_resource!(Test);

    // #[test]
    // fn test_pool() {
    //     let mut pool: Pool<i32, Test> = Pool::with_size(2, || 0);

    //     // Test malloc
    //     let handle1 = pool.malloc();
    //     let handle2 = pool.malloc();
    //     let handle3 = pool.malloc();
    //     assert_eq!(handle1.index, 0);
    //     assert_eq!(handle2.index, 1);
    //     // assert!(handle3.is_none());

    //     // Test get and get_mut
    //     // *pool.get_mut(handle1).unwrap() = 1;
    //     // *pool.get_mut(handle2).unwrap() = 2;
    //     // assert_eq!(*pool.get(handle1).unwrap(), 1);
    //     // assert_eq!(*pool.get(handle2).unwrap(), 2);
    //     // assert!(pool.get(handle3).is_none());
    //     // assert!(pool.get_mut(handle3).is_none());

    //     // // Test free
    //     // pool.free(handle1);
    //     // let handle4 = pool.malloc();
    //     // assert_eq!(handle4.index, 0);
    //     // assert_eq!(*pool.get(handle4).unwrap(), 0);
    // }
}
