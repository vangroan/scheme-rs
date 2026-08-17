use std::ptr::NonNull;

use crate::gc_header::GcHeader;
use crate::trace::{Trace, Tracer, VTable};

#[doc(hidden)]
#[repr(C)]
pub struct GcBox<T: Trace + 'static> {
    header: GcHeader,
    pub(crate) data: T,
}

impl<T: Trace + 'static> GcBox<T> {
    pub(crate) fn new(header: GcHeader, data: T) -> Self {
        GcBox { header, data }
    }

    pub(crate) fn header(&self) -> &GcHeader {
        &self.header
    }

    pub(crate) fn next_node(&self) -> Option<NonNull<ErasedBox>> {
        self.header.next_node()
    }

    pub(crate) fn vtable(&self) -> &'static VTable {
        self.header.vtable()
    }

    #[inline]
    pub fn as_ref(&self) -> &T {
        if self.header().is_dropped() {
            panic!("Attempted to access dropped GcBox");
        }
        &self.data
    }
}

impl<T: Trace + 'static> Trace for GcBox<T> {
    fn trace(&self, tracer: &mut Tracer) {
        self.data.trace(tracer);
    }

    fn trace_in_heap(&self) {
        self.data.trace_in_heap();
    }
}

/// A typed-erased value that ensures an upcasted value is not traced.
#[doc(hidden)]
pub struct Erased(());

impl Trace for Erased {
    fn trace(&self, _tracer: &mut Tracer) {
        unreachable!();
    }

    fn trace_in_heap(&self) {
        unreachable!();
    }
}

impl Drop for Erased {
    fn drop(&mut self) {
        unreachable!("erased box must be cast to its original type before dropping");
    }
}

#[doc(hidden)]
pub type ErasedBox = GcBox<Erased>;
