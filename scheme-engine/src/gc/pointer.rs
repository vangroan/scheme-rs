use crate::gc::heap::GcBox;
use crate::gc::trace::{Trace, Tracer};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

#[repr(transparent)]
pub struct Gc<T: Trace + 'static> {
    pub(crate) ptr: NonNull<GcBox<T>>,
    pub(crate) _marker: PhantomData<std::rc::Rc<T>>,
}

impl<T: Trace + 'static> Gc<T> {
    pub(crate) fn new(ptr: NonNull<GcBox<T>>) -> Self {
        Gc {
            ptr,
            _marker: PhantomData,
        }
    }

    pub(crate) unsafe fn inner(&self) -> &GcBox<T> {
        self.ptr.as_ref()
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.inner().as_ref() }
    }
}

impl<T: Trace + 'static> Trace for Gc<T> {
    fn trace(&self, _tracer: &mut Tracer) {}

    fn trace_in_heap(&self) {
        unsafe {
            // The box is considered a terminal of the object graph
            // in the context of the internal trace step.
            self.inner().header().incr_in_heap();
        }
    }
}

impl<T: Trace + 'static> Clone for Gc<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.inner().header().incr_strong();
        }
        Gc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: Trace + 'static> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner().as_ref() }
    }
}

impl<T: Trace + 'static> Drop for Gc<T> {
    fn drop(&mut self) {
        unsafe { self.inner().header().decr_strong() };
    }
}

#[deprecated(note = "Use `Gc` instead.")]
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
