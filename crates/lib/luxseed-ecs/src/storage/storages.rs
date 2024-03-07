use std::mem::MaybeUninit;

pub struct VecStorage<T>(Vec<MaybeUninit<T>>);

impl<T> Default for VecStorage<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
