use std::cell::Cell;

use scheme_gc::prelude::*;
use scheme_gc::GcRefCell;

thread_local! {
    static DROP_COUNT: Cell<u32> = Cell::new(0);
}

fn drop_count() -> u32 {
    DROP_COUNT.with(|count| count.get())
}

fn clear_drop_count() {
    DROP_COUNT.with(|count| {
        count.set(0);
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DropCounter(u32);

impl DropCounter {
    fn value(&self) -> u32 {
        self.0
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        DROP_COUNT.with(|count| {
            let current = count.get();
            count.set(current + 1);
        });
    }
}

impl Trace for DropCounter {
    fn trace(&self, _tracer: &mut Tracer) {}
    fn trace_in_heap(&self) {}
}

/// When a GC allocated boject is mutably borrowed, it should not be traced by the GC,
/// but should not be deallocated.
#[test]
fn test_write_guard() {
    clear_drop_count();

    let mut heap = GcHeap::new();
    let cell = heap.alloc(GcRefCell::new(DropCounter(42)));
    let borrow = cell.borrow_mut();

    // The value should not be traced while it is mutably borrowed.
    heap.collect();
    assert_eq!(drop_count(), 0);

    assert_eq!(borrow.get().value(), 42);
    drop(borrow);

    heap.collect();
    assert_eq!(drop_count(), 0);
    assert_eq!(cell.borrow().get().value(), 42);

    drop(cell);
    heap.collect();
    assert_eq!(drop_count(), 1);
}

struct Node {
    next: GcRefCell<Option<Gc<Node>>>,
    data: u32,
}

impl Trace for Node {
    fn trace(&self, tracer: &mut Tracer) {
        Trace::trace(&self.next, tracer);
    }

    fn trace_in_heap(&self) {
        Trace::trace_in_heap(&self.next);
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        DROP_COUNT.with(|count| {
            let current = count.get();
            count.set(current + 1);
        });
    }
}

#[test]
fn test_nexted_write_guard() {
    clear_drop_count();

    let mut heap = GcHeap::new();
    let leaf = heap.alloc(GcRefCell::new(Node {
        next: GcRefCell::new(None),
        data: 1,
    }));
    let root = heap.alloc(GcRefCell::new(Node {
        next: GcRefCell::new(Some(leaf.borrow())),
        data: 2,
    }));

    *node1.borrow_mut().next.borrow_mut() = Some(node2.clone());

    // The nodes should not be traced while they are mutably borrowed.
    heap.collect();
    assert_eq!(drop_count(), 0);

    drop(node1);
    drop(node2);

    heap.collect();
    assert_eq!(drop_count(), 2);
}
