use std::num::NonZeroUsize;
use std::{alloc::Layout, ptr::NonNull};

pub struct AnyVec {
    data: NonNull<u8>,
    capacity: usize,
    len: usize,
    layout: Layout,
    drop_fn: Option<unsafe fn(*mut u8)>,
}

impl AnyVec {
    pub fn new(layout: Layout, capacity: usize, drop: Option<unsafe fn(*mut u8)>) -> Self {
        let data = unsafe { std::alloc::alloc(layout) };
        Self {
            data: NonNull::new(data).expect("Failed to allocate memory"),
            capacity,
            len: 0,
            layout,
            drop_fn: drop,
        }
    }

    fn grow_exact(&mut self, increment: NonZeroUsize) {
        let new_capacity = self.capacity + increment.get();
        let new_layout =
            Layout::from_size_align(self.layout.size() * new_capacity, self.layout.align())
                .unwrap();
        let new_data = unsafe { std::alloc::realloc(self.data.as_ptr(), new_layout, new_capacity) };
        self.data = NonNull::new(new_data).expect("Failed to allocate memory");
        self.capacity = new_capacity;
    }

    pub fn clear(&mut self) {
        let len = self.len;
        self.len = 0;
        if let Some(drop) = self.drop_fn {
            let size = self.layout.size();
            for i in 0..len {
                unsafe {
                    drop(self.data.as_ptr().add(i * size));
                }
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        self.layout
    }
}

impl Drop for AnyVec {
    fn drop(&mut self) {
        self.clear();
    }
}
