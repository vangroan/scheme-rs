use std::collections::VecDeque;
use std::ptr::NonNull;

use crate::gc_box::{ErasedBox, GcBox};

pub trait Trace {
    fn trace(&self, tracer: &mut Tracer);

    fn trace_in_heap(&self);
}

pub struct Tracer {
    queue: VecDeque<NonNull<ErasedBox>>,
}

impl Tracer {
    pub(super) fn new() -> Self {
        Tracer {
            queue: VecDeque::new(),
        }
    }

    // TODO: Consider making this a public API for custom tracing.
    pub(crate) fn enqueue(&mut self, ptr: NonNull<ErasedBox>) {
        unsafe {
            if ptr.as_ref().header().is_marked() {
                return;
            }
        }
        self.queue.push_back(ptr);
    }

    pub(crate) fn try_dequeue(&mut self) -> Option<NonNull<ErasedBox>> {
        self.queue.pop_front()
    }

    pub(super) fn clear(&mut self) {
        self.queue.clear();
        self.queue.shrink_to_fit();
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement [`Trace`] for a type that is known to not contain any references
/// to other GC-managed objects.
macro_rules! impl_trace_terminal {
    ($type:ty) => {
        impl Trace for $type {
            fn trace(&self, _tracer: &mut Tracer) {}

            fn trace_in_heap(&self) {}
        }
    };
}

impl_trace_terminal!(());
impl_trace_terminal!(bool);
impl_trace_terminal!(i8);
impl_trace_terminal!(i16);
impl_trace_terminal!(i32);
impl_trace_terminal!(i64);
impl_trace_terminal!(isize);
impl_trace_terminal!(u8);
impl_trace_terminal!(u16);
impl_trace_terminal!(u32);
impl_trace_terminal!(u64);
impl_trace_terminal!(u128);
impl_trace_terminal!(usize);
impl_trace_terminal!(f32);
impl_trace_terminal!(f64);
impl_trace_terminal!(&str);

pub(crate) struct VTable {
    pub trace_fn: unsafe fn(NonNull<ErasedBox>, &mut Tracer),
    pub trace_in_heap_fn: unsafe fn(NonNull<ErasedBox>),
    pub drop_data_fn: unsafe fn(NonNull<ErasedBox>),
    pub dealloc_fn: unsafe fn(NonNull<ErasedBox>),
    pub size_fn: fn() -> usize,
}

pub(super) const fn vtable_of<T>() -> &'static VTable
where
    T: Trace + 'static,
{
    trait HasVTable: Trace + Sized + 'static {
        const VTABLE: &'static VTable;

        unsafe fn trace_fn(this: NonNull<ErasedBox>, tracer: &mut Tracer) {
            let this: NonNull<GcBox<Self>> = this.cast();
            Trace::trace(this.as_ref().as_ref(), tracer);
        }

        unsafe fn trace_in_heap_fn(this: NonNull<ErasedBox>) {
            let this: NonNull<GcBox<Self>> = this.cast();
            Trace::trace_in_heap(this.as_ref());
        }

        /// Drop only the payload. The header stays intact so recursive `Gc<T>` drops
        /// can still touch metadata on other unreachable nodes in this GC cycle.
        unsafe fn drop_data_fn(this: NonNull<ErasedBox>) {
            let this: NonNull<GcBox<Self>> = this.cast();

            // Multiple drops can be attempted on the same node if there are
            // cycles in the object graph.
            if this.as_ref().header().is_dropped() {
                return;
            }

            std::ptr::drop_in_place(&mut ((*this.as_ptr()).data));
            this.as_ref().header().set_dropped();
        }

        /// Deallocate a previously dropped node without running destructors again.
        unsafe fn dealloc_fn(this: NonNull<ErasedBox>) {
            let this: NonNull<GcBox<Self>> = this.cast();

            debug_assert!(
                this.as_ref().header().is_dropped(),
                "Attempted to deallocate a GcBox that has not been dropped"
            );

            std::alloc::dealloc(
                this.as_ptr().cast::<u8>(),
                std::alloc::Layout::new::<GcBox<Self>>(),
            );
        }

        /// Size of the downcasted type.
        fn size() -> usize {
            std::mem::size_of::<GcBox<Self>>()
        }
    }

    impl<T: Trace + 'static> HasVTable for T {
        const VTABLE: &'static VTable = &VTable {
            trace_fn: <T as HasVTable>::trace_fn,
            trace_in_heap_fn: <T as HasVTable>::trace_in_heap_fn,
            drop_data_fn: <T as HasVTable>::drop_data_fn,
            dealloc_fn: <T as HasVTable>::dealloc_fn,
            size_fn: <T as HasVTable>::size,
        };
    }

    T::VTABLE
}
