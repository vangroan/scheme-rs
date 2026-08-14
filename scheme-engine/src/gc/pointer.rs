use crate::gc::heap::GcBox;
use crate::gc::trace::{Trace, Tracer};
use std::ops::Deref;
use std::ptr::NonNull;

#[repr(transparent)]
pub struct Gc<T: Trace + 'static> {
    pub(super) ptr: NonNull<GcBox<T>>,
}

impl<T: Trace + 'static> Gc<T> {
    pub(super) unsafe fn inner(&self) -> &GcBox<T> {
        self.ptr.as_ref()
    }
}

impl<T: Trace + 'static> Trace for Gc<T> {
    fn trace(&self, tracer: &mut Tracer) {}

    fn trace_in_heap(&self) {
        unsafe {
            self.inner().header().incr_in_heap();
        }
    }
}

impl<T: Trace + 'static> Clone for Gc<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.inner().header().incr_strong();
        }
        Gc { ptr: self.ptr }
    }
}

impl<T: Trace + 'static> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner().as_ref() }
    }
}

#[repr(transparent)]
pub struct Root<T: Trace + 'static> {
    pub(super) ptr: NonNull<GcBox<T>>,
}

impl<T: Trace + 'static> Root<T> {
    pub(super) fn new(mut ptr: NonNull<GcBox<T>>) -> Self {
        unsafe {
            ptr.as_mut().header().incr_strong();
        }
        Root { ptr }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &self.ptr.as_ref().as_ref() }
    }
}

impl<T: Trace + 'static> Clone for Root<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.ptr.as_ref().header().incr_strong();
        }
        Root { ptr: self.ptr }
    }
}

impl<T: Trace + 'static> Drop for Root<T> {
    fn drop(&mut self) {
        unsafe { self.ptr.as_ref().header().decr_strong() };
    }
}
