use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::gc_box::GcBox;
use crate::trace::{Trace, Tracer};

/// Assert that the box being pointed to has not been dropped.
///
/// Can only detect the phase between drop and deallocation.
macro_rules! assert_not_dropped {
    ($gc:expr) => {
        if cfg!(debug_assertions) {
            let value = unsafe { ($gc).inner() };
            debug_assert!(!value.header().is_dropped());
        }
    };
}

#[repr(transparent)]
pub struct Gc<T: Trace + 'static> {
    ptr: NonNull<GcBox<T>>,
    _marker: PhantomData<std::rc::Rc<T>>,
}

impl<T: Trace + 'static> Gc<T> {
    pub(crate) fn new(ptr: NonNull<GcBox<T>>) -> Self {
        Gc {
            ptr,
            _marker: PhantomData,
        }
    }

    pub(crate) unsafe fn inner(&self) -> &GcBox<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.inner().as_ref() }
    }
}

impl<T: Trace + 'static> Trace for Gc<T> {
    fn trace(&self, tracer: &mut Tracer) {
        assert_not_dropped!(self);

        // To avoid excessive recursion we enqueue the box (edge of the object
        // graph) to be iteratively searched.
        tracer.enqueue(self.ptr.cast());
    }

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
