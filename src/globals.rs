use std::{collections::HashMap, sync::RwLock};

use lazy_static::lazy_static;

pub(crate) unsafe fn drop_as_type<T>(value: *mut ()) {
    ::std::ptr::drop_in_place(value as *mut T);
}

#[derive(Debug)]
pub(crate) struct GlobalRef {
    map: HashMap<*mut (), unsafe fn(*mut ())>,
}

impl GlobalRef {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub unsafe fn insert<T>(&mut self, r: *mut T) {
        self.map.insert(r as _, drop_as_type::<T>);
    }
}

unsafe impl Send for GlobalRef {}
unsafe impl Sync for GlobalRef {}

lazy_static! {
    pub(crate) static ref GLOBALS: RwLock<GlobalRef> = RwLock::new(GlobalRef::new());
}

pub unsafe fn drop_globals() {
    for (static_ref, drop_static) in GLOBALS.write().unwrap().map.drain() {
        drop_static(static_ref);
    }
}

pub unsafe fn register<T>(global: &'static mut T) {
    GLOBALS.write().unwrap().insert(global);
}

pub unsafe fn register_pointer<T>(global: *mut T) {
    GLOBALS.write().unwrap().insert(global);
}

pub fn print_globals() {
    println!("{:?}", &*GLOBALS);
}
