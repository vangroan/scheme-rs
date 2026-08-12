//! Value pointer type.

#![allow(dead_code)] // Work-in-progress

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
    pub(crate) ptr: *mut u8,
}

impl ValuePtr {
    /// Check whether the [`ValuePtr`] is a null pointer.
    pub fn is_nil(&self) -> bool {
        ((self.ptr as usize & TAG_MASK) == 0) && self.ptr.is_null()
    }

    /// Check whether the [`ValuePtr`] is a pointer to a heap-allocated value.
    pub fn is_ptr(&self) -> bool {
        (self.ptr as usize & TAG_MASK) == 0
    }

    /// Materialize the [`ValuePtr`] into an [`Expr`].
    ///
    /// This increases the byte size of the value, but allows for easier
    /// type-safe manipulation of the value.
    pub fn to_expr(&self) -> Expr {
        // Note: Do we need reference counting on pointers to objects?
        todo!("Implement conversion from ValuePtr to Expr");
    }
}
