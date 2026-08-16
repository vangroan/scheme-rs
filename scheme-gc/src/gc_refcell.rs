use std::cell::{Cell, UnsafeCell};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use crate::trace::{Trace, Tracer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorrowState {
    Unused,
    Reading,
    Writing,
}

// Positive values represent the number of immutable borrows, negative values
// represent a mutable borrow, and zero represents no borrows.
type BorrowCounter = isize;
const UNUSED: BorrowCounter = 0;

#[inline(always)]
const fn is_writing(x: BorrowCounter) -> bool {
    x < UNUSED
}

#[inline(always)]
const fn is_reading(x: BorrowCounter) -> bool {
    x > UNUSED
}

pub struct GcRefCell<T: Trace> {
    borrow: Cell<BorrowCounter>,
    value: UnsafeCell<T>,
}

impl<T: Trace> GcRefCell<T> {
    pub fn new(value: T) -> Self {
        GcRefCell {
            borrow: Cell::new(UNUSED),
            value: UnsafeCell::new(value),
        }
    }

    pub fn state(&self) -> BorrowState {
        let borrow = self.borrow.get();
        if is_writing(borrow) {
            BorrowState::Writing
        } else if is_reading(borrow) {
            BorrowState::Reading
        } else {
            BorrowState::Unused
        }
    }

    pub fn borrow(&self) -> GcRef<'_, T> {
        match self.try_borrow() {
            Ok(gc_ref) => gc_ref,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn try_borrow(&self) -> Result<GcRef<'_, T>, GcBorrowError> {
        GcRef::new(self)
    }

    pub fn borrow_mut(&self) -> GcMut<'_, T> {
        match self.try_borrow_mut() {
            Ok(gc_mut) => gc_mut,
            Err(err) => panic!("{err}"),
        }
    }

    pub fn try_borrow_mut(&self) -> Result<GcMut<'_, T>, GcBorrowMutError> {
        GcMut::new(self)
    }
}

impl<T: Trace> Trace for GcRefCell<T> {
    fn trace(&self, tracer: &mut Tracer) {
        unsafe { self.value.get().as_ref().unwrap() }.trace(tracer);
    }

    fn trace_in_heap(&self) {
        // The outer GcBox can be marked as reachable, but the inner value
        // should not be traced if it is mutably borrowed. The current
        // collection could be triggered by an assignment to this inner value.
        //
        // By not tracing the inner value the root count will exceed the
        // internal references, and the inner value will be treated as a root.
        //
        // In the worst case an island of cyclical roots will survive until the
        // next collection, after the GcMut is dropped.
        if self.state() != BorrowState::Writing {
            // SAFETY: Pointer is valid because borrow is tracked at runtime.
            unsafe { self.value.get().as_ref().unwrap() }.trace_in_heap();
        }
    }
}

pub struct GcRef<'a, T: Trace> {
    value: NonNull<T>,
    borrow: &'a Cell<BorrowCounter>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Trace> GcRef<'a, T> {
    fn new(gc_refcell: &'a GcRefCell<T>) -> Result<Self, GcBorrowError> {
        let GcRefCell { borrow, value } = gc_refcell;

        if is_writing(borrow.get()) {
            return Err(GcBorrowError {});
        }

        borrow.set(borrow.get() + 1);

        // SAFETY: Not null because pointer comes from UnsafeCell.
        Ok(GcRef {
            value: unsafe { NonNull::new_unchecked(value.get()) },
            borrow,
            _marker: PhantomData,
        })
    }

    pub fn get(&self) -> &T {
        // SAFETY: Pointer is valid because borrow is tracked at runtime.
        unsafe { self.value.as_ref() }
    }
}

impl<T: Trace> Deref for GcRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Pointer is valid because borrow is tracked at runtime.
        unsafe { self.value.as_ref() }
    }
}

impl<T: Trace> Drop for GcRef<'_, T> {
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get() - 1);
    }
}

pub struct GcMut<'a, T: Trace> {
    value: NonNull<T>,
    borrow: &'a Cell<BorrowCounter>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: Trace> GcMut<'a, T> {
    fn new(gc_refcell: &'a GcRefCell<T>) -> Result<Self, GcBorrowMutError> {
        let GcRefCell { borrow, value } = gc_refcell;

        if is_writing(borrow.get()) || is_reading(borrow.get()) {
            return Err(GcBorrowMutError {});
        }

        borrow.set(borrow.get() - 1);

        // SAFETY: Not null because pointer comes from UnsafeCell.
        Ok(GcMut {
            value: unsafe { NonNull::new_unchecked(value.get()) },
            borrow,
            _marker: PhantomData,
        })
    }

    pub fn get(&self) -> &T {
        // SAFETY: Pointer is valid because borrow is tracked at runtime.
        unsafe { self.value.as_ref() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: Pointer is valid because borrow is tracked at runtime.
        unsafe { self.value.as_mut() }
    }
}

impl<T: Trace> Deref for GcMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Pointer is valid because borrow is tracked at runtime.
        unsafe { self.value.as_ref() }
    }
}

impl<T: Trace> DerefMut for GcMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Pointer is valid because borrow is tracked at runtime.
        unsafe { self.value.as_mut() }
    }
}

impl<T: Trace> Drop for GcMut<'_, T> {
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get() + 1);
    }
}

#[derive(Debug)]
pub struct GcBorrowError {}

impl std::error::Error for GcBorrowError {}

impl Display for GcBorrowError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "GcRefCell is already mutably borrowed")
    }
}

#[derive(Debug)]
pub struct GcBorrowMutError {}

impl std::error::Error for GcBorrowMutError {}

impl Display for GcBorrowMutError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "GcRefCell is already immutably borrowed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow() {
        let cell = GcRefCell::new(5);
        assert_eq!(cell.state(), BorrowState::Unused);

        let borrow1 = cell.borrow();
        assert_eq!(*borrow1, 5);
        assert_eq!(cell.state(), BorrowState::Reading);
        assert!(cell.try_borrow_mut().is_err());

        let borrow2 = cell.borrow();
        assert_eq!(*borrow2, 5);
        assert!(cell.try_borrow_mut().is_err());

        drop(borrow1);
        drop(borrow2);

        assert_eq!(cell.state(), BorrowState::Unused);
        assert!(cell.try_borrow().is_ok());
        assert!(cell.try_borrow_mut().is_ok());
    }

    #[test]
    fn test_borrow_try() {
        let cell = GcRefCell::new(7);
        assert_eq!(cell.state(), BorrowState::Unused);

        let borrow1 = cell.borrow_mut();
        assert_eq!(*borrow1, 7);
        assert!(cell.state() == BorrowState::Writing);

        assert!(cell.try_borrow().is_err());
        assert!(cell.try_borrow_mut().is_err());

        drop(borrow1);

        assert_eq!(cell.state(), BorrowState::Unused);
        assert!(cell.try_borrow().is_ok());
        assert!(cell.try_borrow_mut().is_ok());
    }
}
