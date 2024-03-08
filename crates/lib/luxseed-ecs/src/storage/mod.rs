pub use self::storages::VecStorage;
use crate::entity::Index;

mod storages;

pub trait UnsafeStorage<T> {
    unsafe fn get(&self, id: Index) -> &T;
    unsafe fn get_mut(&mut self, id: Index) -> &mut T;
    unsafe fn insert(&mut self, id: Index, value: T);
    unsafe fn remove(&mut self, id: Index) -> T;
    unsafe fn drop(&mut self, id: Index) {
        unsafe { self.remove(id) };
    }
}
