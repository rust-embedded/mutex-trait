//! Low level definition of a Mutex.
//!
//! This crate provides:
//!
//! - A `Mutex` trait that is to be used as the foundation of exclusive access to the data
//!   contained within it.
//! - Helper traits and implementations which allows for multiple locks to be taken at once.
//!
//! RFC that added this trait: [RFC #377](https://github.com/rust-embedded/wg/blob/master/rfcs/0377-mutex-trait.md)
//!
//! # Example
//!
//! ```
//! use mutex_trait::prelude::*;
//!
//! // A function taking 2 mutexes
//! fn normal_lock(
//!     a: &mut impl Mutex<Data = i32>,
//!     b: &mut impl Mutex<Data = i32>,
//! ) {
//!     // Taking each lock separately
//!     a.lock(|a| {
//!         b.lock(|b| {
//!             *a += 1;
//!             *b += 1;
//!         });
//!     });
//!
//!     // Or both at once
//!     (a, b).lock(|a, b| {
//!         *a += 1;
//!         *b += 1;
//!     });
//! }
//! ```
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate is guaranteed to compile on stable Rust 1.31 and up. It *might*
//! compile with older versions but that may change in any new patch release.

#![no_std]
#![deny(missing_docs)]

use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

/// Makes locks work on N-tuples, locks the mutexes from left-to-right in the tuple. These are
/// used to reduce rightward drift in code and to help make intentions clearer.
///
/// # Example
///
/// ```
/// use mutex_trait::prelude::*;
///
/// fn normal_lock(
///     a: &mut impl Mutex<Data = i32>,
///     b: &mut impl Mutex<Data = i32>,
///     c: &mut impl Mutex<Data = i32>
/// ) {
///     // A lot of rightward drift...
///     a.lock(|a| {
///         b.lock(|b| {
///             c.lock(|c| {
///                 *a += 1;
///                 *b += 1;
///                 *c += 1;
///             });
///         });
///     });
/// }
/// ```
///
/// Has a shorthand as:
///
/// ```
/// use mutex_trait::prelude::*;
///
/// fn tuple_lock(
///     a: &mut impl Mutex<Data = i32>,
///     b: &mut impl Mutex<Data = i32>,
///     c: &mut impl Mutex<Data = i32>
/// ) {
///     // Look! Single indent and less to write
///     (a, b, c).lock(|a, b, c| {
///         *a += 1;
///         *b += 1;
///         *c += 1;
///     });
/// }
/// ```
pub mod prelude {
    pub use crate::Mutex;

    macro_rules! lock {
        ($e:ident, $fun:block) => {
            $e.lock(|$e| $fun )
        };
        ($e:ident, $($es:ident),+, $fun:block) => {
            $e.lock(|$e| lock!($($es),*, $fun))
        };
    }

    macro_rules! make_tuple_impl {
        ($name:ident, $($es:ident),+) => {
            /// Auto-generated tuple implementation, see [`Mutex`](../trait.Mutex.html) for details.
            pub trait $name {
                $(
                    /// Data protected by the mutex.
                    type $es;
                )*

                /// Creates a critical section and grants temporary access to the protected data.
                fn lock<R>(&mut self, f: impl FnOnce($(&mut Self::$es),*) -> R) -> R;
            }

            impl<$($es),+> $name for ($($es,)+)
            where
                $($es: crate::Mutex),*
            {
                $(
                    type $es = $es::Data;
                )*

                #[allow(non_snake_case)]
                fn lock<R>(&mut self, f: impl FnOnce($(&mut Self::$es),*) -> R) -> R {
                    let ($(
                        $es,
                    )*) = self;

                    lock!($($es),*, { f($($es),*) })
                }
            }
        };
    }

    // Generate tuple lock impls
    make_tuple_impl!(TupleExt01, T1);
    make_tuple_impl!(TupleExt02, T1, T2);
    make_tuple_impl!(TupleExt03, T1, T2, T3);
    make_tuple_impl!(TupleExt04, T1, T2, T3, T4);
    make_tuple_impl!(TupleExt05, T1, T2, T3, T4, T5);
    make_tuple_impl!(TupleExt06, T1, T2, T3, T4, T5, T6);
    make_tuple_impl!(TupleExt07, T1, T2, T3, T4, T5, T6, T7);
    make_tuple_impl!(TupleExt08, T1, T2, T3, T4, T5, T6, T7, T8);
    make_tuple_impl!(TupleExt09, T1, T2, T3, T4, T5, T6, T7, T8, T9);
    make_tuple_impl!(TupleExt10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
    make_tuple_impl!(TupleExt11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
    make_tuple_impl!(TupleExt12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
}

/// Any object implementing this trait guarantees exclusive access to the data contained
/// within the mutex for the duration of the lock.
pub trait Mutex {
    /// Data protected by the mutex.
    type Data;

    /// Creates a critical section and grants temporary access to the protected data.
    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
}

// `lock` will now work on any mutable reference to a lock
impl<L> Mutex for &'_ mut L
where
    L: Mutex,
{
    type Data = L::Data;

    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        L::lock(self, f)
    }
}

// A RefCell is a lock in single threaded applications
impl<T> Mutex for &'_ RefCell<T> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut self.borrow_mut())
    }
}

/// Wraps a `T` and provides exclusive access via a `Mutex` impl.
///
/// This provides an no-op `Mutex` implementation for data that does not need a real mutex.
#[derive(Debug)]
pub struct Exclusive<'a, T>(&'a mut T);

impl<'a, T> Exclusive<'a, T> {
    /// Creates a new `Exclusive` object wrapping `data`.
    pub fn new(data: &'a mut T) -> Self {
        Exclusive(data)
    }

    /// Consumes this `Exclusive` instance and returns the wrapped value.
    pub fn into_inner(self) -> &'a mut T {
        self.0
    }
}

impl<'a, T> From<&'a mut T> for Exclusive<'a, T> {
    fn from(data: &'a mut T) -> Self {
        Exclusive(data)
    }
}

impl<'a, T> Deref for Exclusive<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<'a, T> DerefMut for Exclusive<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'a, T> Mutex for Exclusive<'a, T> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        f(self.0)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::prelude::*;
    use crate::Exclusive;

    fn compile_test_single_move(mut a: impl Mutex<Data = i32>) {
        a.lock(|a| {
            *a += 1;
        });
    }

    fn compile_test_single_reference(a: &mut impl Mutex<Data = i32>) {
        a.lock(|a| {
            *a += 1;
        });
    }

    fn compile_test_double_move(mut a: impl Mutex<Data = i32>, mut b: impl Mutex<Data = i32>) {
        a.lock(|a| {
            *a += 1;
        });

        b.lock(|b| {
            *b += 1;
        });

        (a, b).lock(|a, b| {
            *a += 1;
            *b += 1;
        });
    }

    fn compile_test_double_reference(
        a: &mut impl Mutex<Data = i32>,
        b: &mut impl Mutex<Data = i32>,
    ) {
        a.lock(|a| {
            *a += 1;
        });

        b.lock(|b| {
            *b += 1;
        });

        (a, b).lock(|a, b| {
            *a += 1;
            *b += 1;
        });
    }

    fn compile_test_move_and_reference(
        mut a: impl Mutex<Data = i32>,
        b: &mut impl Mutex<Data = i32>,
    ) {
        a.lock(|a| {
            *a += 1;
        });

        b.lock(|b| {
            *b += 1;
        });

        (a, b).lock(|a, b| {
            *a += 1;
            *b += 1;
        });
    }

    #[test]
    fn refcell_lock() {
        let a = core::cell::RefCell::new(0);
        let b = core::cell::RefCell::new(0);

        (&a).lock(|a| {
            *a += 1;
        });

        (&b).lock(|b| {
            *b += 1;
        });

        (&a, &b).lock(|a, b| {
            *a += 1;
            *b += 1;
        });
    }

    #[test]
    fn exclusive() {
        let mut var = 0;
        let mut excl = Exclusive(&mut var);

        excl.lock(|val| *val += 1);

        assert_eq!(*excl.into_inner(), 1);
    }
}
