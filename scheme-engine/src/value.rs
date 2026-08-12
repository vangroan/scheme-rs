//! Compact dynamically-typed value representation.
//!
//! [`ValuePtr`] fits in a single pointer-sized word by exploiting the fact that
//! heap allocations are always aligned to at least 8 bytes, leaving the lowest
//! 3 bits of any valid pointer permanently zero. Those bits are used as a type
//! tag, so a `ValuePtr` can represent several types without boxing:
//!
//! | `ptr & 0b111` | Type    | Payload                                  |
//! |---------------|---------|------------------------------------------|
//! | `0b000`       | pointer | address of heap object (`0` = nil)       |
//! | `0b001`       | integer | `(ptr as isize) >> 1` (63-bit signed)    |
//! | `0b010`       | bool    | `ptr >> 3` (0 = false, 1 = true)         |
//! | `0b100`       | void    | none (the entire word is `0b100`)        |
//! | `0b110`       | unused  | reserved for future use                  |
//!
//! Because integers use only bit 0 as their tag, any tag with bit 0 set
//! (`0b011`, `0b101`, `0b111`) is permanently reserved — the integer check
//! tests only bit 0, so those values would always be misidentified as integers.
//! This is a deliberate tradeoff: a 1-bit tag is cheaper to test and preserves
//! a full 63-bit signed integer range (`MIN_INT`..=`MAX_INT`).

use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::expr::Expr;

/// Assert that the pointer is aligned to 8 bytes to ensure that
/// the low 3 bits can be used for tagging.
///
/// Only asserts in debug builds, so it does not affect release performance.
/// Incorrect alignment should be caught by unit tests.
macro_rules! assert_aligned {
    ($ptr:expr) => {
        debug_assert_eq!(
            ($ptr as usize) & TAG_MASK,
            0,
            "ValuePtr must be aligned to 8 bytes"
        );
    };
}

const TAG_MASK: usize = 0b111;
const TAG_INT: usize = 0b001;
const TAG_BOOL: usize = 0b010;
const TAG_VOID: usize = 0b100;

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

    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    /// Create a new [`ValuePtr`] representing the `nil` value.
    pub fn nil() -> Self {
        Self {
            ptr: 0,
            _marker: PhantomData,
        }
    }

    /// Create a new [`ValuePtr`] representing the void/unspecified value.
    pub fn void() -> Self {
        Self {
            ptr: TAG_VOID,
            _marker: PhantomData,
        }
    }

    /// Create a new [`ValuePtr`] from a raw pointer.
    ///
    /// The pointer must be aligned to 8 bytes to allow for tagging.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the pointer is not aligned to 8 bytes.
    pub fn from_ptr<T>(ptr: NonNull<T>) -> Self {
        assert_aligned!(ptr.as_ptr());

        // We don't accept null pointers because the caller knows their intent better.
        Self {
            ptr: ptr.as_ptr() as usize,
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

    /// Create a new [`ValuePtr`] from a boolean value.
    pub fn from_bool(value: bool) -> Self {
        Self {
            ptr: (value as usize) << 3 | TAG_BOOL,
            _marker: PhantomData,
        }
    }

    // -------------------------------------------------------------------------
    // Tag Checks
    // -------------------------------------------------------------------------

    /// Check whether the [`ValuePtr`] is a null pointer.
    pub fn is_nil(&self) -> bool {
        // Null pointers always have address 0: https://doc.rust-lang.org/std/ptr/fn.null.html
        self.ptr == 0
    }

    /// Check whether the [`ValuePtr`] is the void/unspecified value.
    pub fn is_void(&self) -> bool {
        self.ptr == TAG_VOID
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

    /// Check whether the [`ValuePtr`] is an integer value.
    pub fn is_int(&self) -> bool {
        // Only bit 0 is the tag; bits 1+ carry the integer value.
        (self.ptr & TAG_INT) != 0
    }

    /// Check whether the [`ValuePtr`] is a boolean value.
    pub fn is_bool(&self) -> bool {
        (self.ptr & TAG_MASK) == TAG_BOOL
    }

    // -------------------------------------------------------------------------
    // Extract Values
    // -------------------------------------------------------------------------

    /// Extract the integer value from a [`ValuePtr`].
    ///
    /// # Panics
    ///
    /// Panics if the [`ValuePtr`] is not an integer.
    pub fn to_int(&self) -> isize {
        assert!(self.is_int(), "ValuePtr is not an integer");
        (self.ptr as isize) >> 1
    }

    /// Extract the boolean value from a [`ValuePtr`].
    ///
    /// # Panics
    ///
    /// Panics if the [`ValuePtr`] is not a boolean.
    pub fn to_bool(&self) -> bool {
        assert!(self.is_bool(), "ValuePtr is not a boolean");
        (self.ptr >> 3) != 0
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

    /// Materialize the [`ValuePtr`] into an [`Expr`].
    ///
    /// This increases the byte size of the value, but allows for easier
    /// type-safe manipulation of the value.
    pub fn to_expr(&self) -> Expr {
        // Note: Do we need reference counting on pointers to objects?
        todo!("Implement conversion from ValuePtr to Expr");
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

    #[test]
    fn test_nil() {
        let v = ValuePtr::nil();
        assert!(v.is_nil());
        assert!(!v.is_int());
        assert!(!v.is_bool());
        assert!(!v.is_void());
        assert!(!v.is_ptr());
    }

    #[test]
    fn test_int_roundtrip() {
        for n in [0, 1, -1, 42, -42, ValuePtr::MAX_INT, ValuePtr::MIN_INT] {
            let v = ValuePtr::from_int(n);
            assert!(v.is_int(), "is_int failed for {n}");
            assert_eq!(v.to_int(), n, "to_int failed for {n}");
        }
    }

    #[test]
    fn test_int_not_other_types() {
        // Use 1 to exercise a value with bit 1 set in the encoded form.
        let v = ValuePtr::from_int(1);
        assert!(!v.is_nil());
        assert!(!v.is_bool());
        assert!(!v.is_void());
        assert!(!v.is_ptr());
    }

    #[test]
    fn test_bool_true() {
        let v = ValuePtr::from_bool(true);
        assert!(v.is_bool());
        assert!(v.to_bool());
        assert!(!v.is_nil());
        assert!(!v.is_int());
        assert!(!v.is_void());
        assert!(!v.is_ptr());
    }

    #[test]
    fn test_bool_false() {
        let v = ValuePtr::from_bool(false);
        assert!(v.is_bool());
        assert!(!v.to_bool());
        assert!(!v.is_nil());
        assert!(!v.is_int());
        assert!(!v.is_void());
        assert!(!v.is_ptr());
    }

    #[test]
    fn test_void() {
        let v = ValuePtr::void();
        assert!(v.is_void());
        assert!(!v.is_nil());
        assert!(!v.is_int());
        assert!(!v.is_bool());
        assert!(!v.is_ptr());
    }

    #[test]
    fn test_ptr() {
        let raw = Box::into_raw(Box::new(42u64));
        let non_null = NonNull::new(raw).unwrap();
        let v = ValuePtr::from_ptr(non_null);
        assert!(v.is_ptr());
        assert!(!v.is_nil());
        assert!(!v.is_int());
        assert!(!v.is_bool());
        assert!(!v.is_void());
        unsafe {
            assert_eq!(*v.to_ptr::<u64>(), 42);
            drop(Box::from_raw(raw));
        }
    }
}
