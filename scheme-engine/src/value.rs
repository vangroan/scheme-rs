//! Value pointer type.

#![allow(dead_code)] // Work-in-progress

use std::marker::PhantomData;

use crate::expr::Expr;

/// Assert that the pointer is aligned to 8 bytes to ensure that
/// the low 3 bits can be used for tagging.
macro_rules! assert_aligned {
    ($ptr:expr) => {
        assert_eq!(
            ($ptr as usize) & TAG_MASK,
            0,
            "ValuePtr must be aligned to 8 bytes"
        );
    };
}

const TAG_MASK: usize = 0b111;
const TAG_INT: usize = 0b001;

/// Dynamically typed value.
///
/// Implemented as a "tagged-pointer" type, where the low bits of the pointer
/// are used to store a tag indicating the type of the value.
#[repr(transparent)]
pub struct ValuePtr {
    /// The pointer to a heap-allocated value, or a tagged integer.
    /// Cast to `usize` to prevent the debugger from trying to dereference it.
    pub(crate) ptr: usize,
    _marker: PhantomData<*mut u8>, // Prevents Send/Sync
}

impl ValuePtr {
    /// The maximum integer value that can be stored in a [`ValuePtr`].
    pub const MAX_INT: isize = isize::MAX >> 1;

    /// The minimum integer value that can be stored in a [`ValuePtr`].
    pub const MIN_INT: isize = isize::MIN >> 1;

    /// Create a new [`ValuePtr`] representing the `nil` value.
    pub fn nil() -> Self {
        Self {
            ptr: 0,
            _marker: PhantomData,
        }
    }

    /// Create a new [`ValuePtr`] from a raw pointer.
    ///
    /// # Panics
    ///
    /// Panics if the pointer is not aligned to 8 bytes.
    pub fn from_ptr<T>(ptr: *mut T) -> Self {
        assert_aligned!(ptr);
        Self {
            ptr: ptr as usize,
            _marker: PhantomData,
        }
    }

    /// Create a new [`ValuePtr`] from an integer value.
    ///
    /// The value must be within the range of [`ValuePtr::MIN_INT`] and [`ValuePtr::MAX_INT`].
    ///
    /// # Panics
    ///
    /// Panics if the integer value is too large to be stored in a [`ValuePtr`].
    pub fn from_int(value: isize) -> Self {
        assert!(
            value <= Self::MAX_INT && value >= Self::MIN_INT,
            "Integer value out of range for ValuePtr"
        );

        // Only the lowest 1 bit is necessary to distinguish integers.
        let ptr = (value as usize) << 1 | TAG_INT;

        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Check whether the [`ValuePtr`] is a null pointer.
    pub fn is_nil(&self) -> bool {
        // Null pointers always have address 0: https://doc.rust-lang.org/std/ptr/fn.null.html
        self.ptr == 0
    }

    /// Check whether the [`ValuePtr`] is an integer value.
    pub fn is_int(&self) -> bool {
        (self.ptr & TAG_MASK) == TAG_INT
    }

    /// Extract the integer value from a [`ValuePtr`].
    ///
    /// # Panics
    ///
    /// Panics if the [`ValuePtr`] is not an integer.
    pub fn to_int(&self) -> isize {
        assert!(self.is_int(), "ValuePtr is not an integer");
        (self.ptr as isize) >> 1
    }

    /// Check whether the [`ValuePtr`] is tagged as a pointer to a
    /// heap-allocated value.
    ///
    /// The `nil` value is not considered a pointer, even though it is
    /// represented as a null pointer. This helps to distinguish between a null
    /// pointer and a valid pointer, to prevent accidental null dereference.
    pub fn is_ptr(&self) -> bool {
        (self.ptr & TAG_MASK) == 0 && !self.is_nil()
    }

    /// Materialize the [`ValuePtr`] into an [`Expr`].
    ///
    /// This increases the byte size of the value, but allows for easier
    /// type-safe manipulation of the value.
    pub fn to_expr(&self) -> Expr {
        // Note: Do we need reference counting on pointers to objects?
        todo!("Implement conversion from ValuePtr to Expr");
    }

    /// Cast the [`ValuePtr`] to a raw pointer of type `T`.
    ///
    /// # Safety
    ///
    /// The tags are checked to ensure a scalar value is not being cast to a
    /// pointer. However the caller must ensure that the pointer is valid and
    /// points to a value of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the [`ValuePtr`] is not a pointer. The `nil` value is not
    /// considered a pointer, even though it is represented as a null pointer.
    pub unsafe fn to_ptr<T>(&self) -> *mut T {
        assert!(self.is_ptr(), "ValuePtr is not a pointer");
        self.ptr as *mut T
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Ensure the current platform's null pointer casts safely to zero.
    #[test]
    fn test_invariant_nil_is_zero() {
        let ptr = std::ptr::null::<u8>();
        assert_eq!(ptr as usize, 0);
    }

    /// Ensure that the size of `ValuePtr` is the same as a pointer-sized integer.
    #[test]
    fn test_invariant_value_is_ptr_sized() {
        use std::mem::size_of;
        assert_eq!(size_of::<ValuePtr>(), size_of::<usize>());
        assert_eq!(size_of::<ValuePtr>(), size_of::<*mut u8>());
    }
}
