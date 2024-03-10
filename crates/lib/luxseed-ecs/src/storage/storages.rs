// use luxseed_utility::sync_unsafe_cell::SyncUnsafeCell;
// use std::{mem::MaybeUninit, ptr};

// use super::UnsafeStorage;
// use crate::entity::Index;

// pub struct VecStorage<T>(Vec<SyncUnsafeCell<MaybeUninit<T>>>);

// impl<T> Default for VecStorage<T> {
//     fn default() -> Self {
//         Self(Default::default())
//     }
// }

// impl<T> UnsafeStorage<T> for VecStorage<T> {
//     unsafe fn get(&self, id: Index) -> &T {
//         let ptr = unsafe { self.0.get_unchecked(id as usize).get() };
//         let maybe_uninit = unsafe { &*ptr };
//         unsafe { maybe_uninit.assume_init_ref() }
//     }

//     unsafe fn get_mut(&mut self, id: Index) -> &mut T {
//         let maybe_uninit = unsafe { self.0.get_unchecked_mut(id as usize).get_mut() };
//         unsafe { maybe_uninit.assume_init_mut() }
//     }

//     unsafe fn insert(&mut self, id: Index, value: T) {
//         // On 32-bit systems, we need to ensure that the index fits into a usize.
//         let id = if Index::BITS >= usize::BITS {
//             core::cmp::min(id, usize::MAX as Index) as usize
//         } else {
//             id as usize
//         };

//         if self.0.len() <= id {
//             let delta = if Index::BITS >= usize::BITS { id.saturating_add(1) } else { id + 1 }
//                 - self.0.len();
//             self.0.reserve(delta);
//             unsafe { self.0.set_len(id + 1) };
//         }
//         unsafe { self.0.get_unchecked_mut(id) }.get_mut().write(value);
//     }

//     unsafe fn remove(&mut self, id: Index) -> T {
//         let ret = unsafe { self.get(id) };
//         unsafe { ptr::read(ret) }
//     }
// }
