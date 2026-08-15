use std::cell::Cell;
use std::ptr::NonNull;

use crate::gc_box::ErasedBox;
use crate::trace::{Trace, VTable, vtable_of};

const MARK_MASK: u32 = 1 << 31;
const COUNT_MASK: u32 = !MARK_MASK;
const MAX_COUNT: u32 = COUNT_MASK;

/// [`GcHeader::strong_count`] holds the number of external references to the
/// object. This count is incremented when a new `Gc<T>` is created and
/// decremented when a `Gc<T>` is dropped. When the strong count reaches zero,
/// the object is considered unreachable from the root set and can be collected
/// by the garbage collector.
///
/// [`GcHeader::in_heap_count`] holds the marker bit in the most significant bit
/// of the 32-bit integer. The remaining 31 bits are used to count the number of
/// internal references to the object.
#[repr(C)]
pub(crate) struct GcHeader {
    next: Cell<Option<NonNull<ErasedBox>>>,
    vtable: &'static VTable,
    strong_count: Cell<u32>,
    in_heap_count: Cell<u32>,
}

impl GcHeader {
    pub(crate) fn new<T: Trace + 'static>(next: Option<NonNull<ErasedBox>>) -> Self {
        GcHeader {
            next: Cell::new(next),
            vtable: vtable_of::<T>(),
            strong_count: Cell::new(0),
            in_heap_count: Cell::new(0),
        }
    }

    pub fn next_node(&self) -> Option<NonNull<ErasedBox>> {
        self.next.get()
    }

    pub fn set_next(&self, next: Option<NonNull<ErasedBox>>) {
        self.next.set(next);
    }

    pub fn vtable(&self) -> &'static VTable {
        self.vtable
    }

    pub fn incr_strong(&self) {
        let mark_and_count = self.strong_count.get();

        let count = (mark_and_count & COUNT_MASK) + 1;
        let dropped_flag = mark_and_count & MARK_MASK;

        // Ensure the strong count does not exceed the maximum internal heap count.
        if count == MAX_COUNT {
            panic!("strong_count overflow");
        }

        self.strong_count.set(dropped_flag | count);
    }

    pub fn decr_strong(&self) {
        let mark_and_count = self.strong_count.get();

        let count = (mark_and_count & COUNT_MASK) - 1;
        let dropped_flag = mark_and_count & MARK_MASK;

        self.strong_count.set(dropped_flag | count);
    }

    pub fn incr_in_heap(&self) {
        let mark_and_count = self.in_heap_count.get();
        let count = (mark_and_count & COUNT_MASK) + 1;
        let marked_flag = mark_and_count & MARK_MASK;

        if count == MAX_COUNT {
            panic!("in_heap_count overflow");
        }

        self.in_heap_count.set(marked_flag | count);
    }

    pub fn is_dropped(&self) -> bool {
        self.strong_count.get() & MARK_MASK != 0
    }

    pub fn set_dropped(&self) {
        self.strong_count.set(self.strong_count.get() | MARK_MASK);
    }

    pub fn reset_in_heap(&self) {
        self.in_heap_count.set(0);
    }

    pub fn is_marked(&self) -> bool {
        (self.in_heap_count.get() & MARK_MASK) != 0
    }

    pub fn mark(&self) {
        self.in_heap_count.set(self.in_heap_count.get() | MARK_MASK);
    }

    pub fn unmark(&self) {
        let mark_and_count = self.in_heap_count.get();
        let count = mark_and_count & COUNT_MASK;
        let marked_flag = mark_and_count & MARK_MASK;

        self.in_heap_count.set(marked_flag | count & !MARK_MASK);
    }

    pub fn is_rooted(&self) -> bool {
        // external-refs = total-refs - internal-refs
        self.in_heap_count.get() < self.strong_count.get()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::trace::vtable_of;

    use super::*;

    #[test]
    fn test_gc_header_preserve_dropped_flag() {
        let header = GcHeader {
            next: Cell::new(None),
            vtable: vtable_of::<()>(),
            strong_count: Cell::new(0),
            in_heap_count: Cell::new(0),
        };
        assert!(!header.is_dropped());

        header.set_dropped();
        assert!(header.is_dropped());

        header.incr_strong();
        assert!(header.is_dropped());
    }

    #[test]
    fn test_gc_header_preserve_marked_flag() {
        let header = GcHeader {
            next: Cell::new(None),
            vtable: vtable_of::<()>(),
            strong_count: Cell::new(0),
            in_heap_count: Cell::new(0),
        };
        assert!(!header.is_marked());

        header.mark();
        assert!(header.is_marked());

        header.incr_in_heap();

        assert!(header.is_marked());
    }
}
