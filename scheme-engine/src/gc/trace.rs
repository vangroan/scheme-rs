use std::{
    collections::{HashSet, VecDeque},
    ptr::NonNull,
};

use crate::gc::heap::GcBox;

use super::heap::ErasedBox;

pub trait Trace {
    fn trace(&self, tracer: &mut Tracer);

    fn trace_in_heap(&self);
}

pub struct Tracer {
    queue: VecDeque<NonNull<ErasedBox>>,
    seen: HashSet<NonNull<ErasedBox>>,
}

impl Tracer {
    pub(super) fn new() -> Self {
        Tracer {
            queue: VecDeque::new(),
            seen: HashSet::new(),
        }
    }

    pub fn enqueue(&mut self, ptr: NonNull<ErasedBox>) {
        if self.seen.insert(ptr) {
            self.queue.push_back(ptr);
        }
    }

    pub(crate) fn try_dequeue(&mut self) -> Option<NonNull<ErasedBox>> {
        self.queue.pop_front()
    }

    pub(super) fn clear(&mut self) {
        self.seen.clear();
        self.queue.clear();

        self.seen.shrink_to_fit();
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
    pub drop_fn: unsafe fn(NonNull<ErasedBox>),
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

        /// The `drop_fn` is responsible for deallocating the memory of the object.
        ///
        /// Preserving the original type information is crucial for correctly
        /// dropping the object, especially if it has a custom `Drop` implementation.
        unsafe fn drop_fn(this: NonNull<ErasedBox>) {
            let this: NonNull<GcBox<Self>> = this.cast();

            // `T` drop called by Box
            drop(Box::from_raw(this.as_ptr()));
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
            drop_fn: <T as HasVTable>::drop_fn,
            size_fn: <T as HasVTable>::size,
        };
    }

    T::VTABLE
}
