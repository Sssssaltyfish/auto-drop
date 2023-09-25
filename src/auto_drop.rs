use std::{cell::UnsafeCell, sync::Once};

use crate::globals::GLOBALS;

pub struct AutoDrop<T>(UnsafeCell<T>, Once);

impl<T> AutoDrop<T> {
    pub const fn new(v: T) -> Self {
        Self(UnsafeCell::new(v), Once::new())
    }

    #[inline(always)]
    pub fn get(&'static self) -> &T {
        self.1.call_once(|| unsafe {
            GLOBALS.write().unwrap().insert(self.0.get());
        });

        unsafe { &*self.0.get() }
    }
}

unsafe impl<T: Sync> Sync for AutoDrop<T> {}
