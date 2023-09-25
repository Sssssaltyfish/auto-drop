#![allow(unused_imports)]

pub use lazy_static::lazy_static;

pub use crate::auto_drop::AutoDrop;
pub use crate::globals::{drop_globals, register};

pub mod auto_drop;
pub mod globals;

pub use ::core::ops::Deref as __Deref;

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __auto_drop_internal {
    // optional visibility restrictions are wrapped in `()` to allow for
    // explicitly passing otherwise implicit information about private items
    ($(#[$attr:meta])* ($($vis:tt)*) static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __auto_drop_internal!(@MAKE TY, $(#[$attr])*, ($($vis)*), $N);
        __auto_drop_internal!(@TAIL, $N : $T = $e);
        auto_drop!($($t)*);
    };
    (@LAZY, $(#[$attr:meta])* ($($vis:tt)*) static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __auto_drop_internal!(@MAKE TY, $(#[$attr])*, ($($vis)*), $N);
        __auto_drop_internal!(@LAZY TAIL, $N : $T = $e);
        auto_drop_lazy!($($t)*);
    };
    (@COMMON TAIL, $N:ident, $T:ty, $INIT:tt) => {
        impl $crate::__Deref for $N {
            type Target = $T;
            fn deref(&self) -> &$T {
                #[inline(always)]
                fn __stability() -> &'static $T {
                    $INIT
                }
                __stability()
            }
        }
    };
    (@TAIL, $N:ident : $T:ty = $e:expr) => {
        __auto_drop_internal!{
            @COMMON TAIL, $N, $T,
            {
                static AUTO_DROP: $crate::auto_drop::AutoDrop<$T> = $crate::AutoDrop::new($e);
                AUTO_DROP.get()
            }
        }

    };
    (@LAZY TAIL, $N:ident : $T:ty = $e:expr) => {
        __auto_drop_internal!{
            @COMMON TAIL, $N, $T,
            {
                lazy_static! {
                    static ref AUTO_DROP: $crate::auto_drop::AutoDrop<$T> = $crate::AutoDrop::new($e);
                }
                AUTO_DROP.get()
            }
        }
        impl lazy_static::LazyStatic for $N {
            fn initialize(lazy: &Self) {
                let _ = &**lazy;
            }
        }
    };
    // `vis` is wrapped in `()` to prevent parsing ambiguity
    (@MAKE TY, $(#[$attr:meta])*, ($($vis:tt)*), $N:ident) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        $(#[$attr])*
        $($vis)* struct $N {__private_field: ()}
        #[doc(hidden)]
        $($vis)* static $N: $N = $N {__private_field: ()};
    };
    () => ()
}

#[macro_export]
macro_rules! auto_drop {
    ($(#[$attr:meta])* static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        // use `()` to explicitly forward the information about private items
        __auto_drop_internal!($(#[$attr])* () static $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __auto_drop_internal!($(#[$attr])* (pub) static $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __auto_drop_internal!($(#[$attr])* (pub ($($vis)+)) static $N : $T = $e; $($t)*);
    };
    () => ()
}

#[macro_export]
macro_rules! auto_drop_lazy {
    ($(#[$attr:meta])* static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        // use `()` to explicitly forward the information about private items
        __auto_drop_internal!(@LAZY, $(#[$attr])* () static $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __auto_drop_internal!(@LAZY, $(#[$attr])* (pub) static $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __auto_drop_internal!(@LAZY, $(#[$attr])* (pub ($($vis)+)) static $N : $T = $e; $($t)*);
    };
    () => ()
}

#[cfg(feature = "dtor")]
#[ctor::dtor]
unsafe fn cleanup() {
    drop_globals();
}

#[cfg(feature = "ctrlc")]
#[ctor::ctor]
unsafe fn setup_handler() {
    ctrlc::set_handler(|| drop_globals()).expect("failed to set ctrl-c handler");
}
