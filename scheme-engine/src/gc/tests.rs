use std::cell::Cell;

use crate::gc::Trace;
use std::cell::RefCell;

use super::{Gc, GcHeap};

#[test]
fn test_gc_heap_alloc() {
    let mut heap = GcHeap::new();
    let root1 = heap.alloc(42);
    let root2 = heap.alloc("hello");

    // Check that the allocated values are correct
    assert_eq!(root1.as_ref(), &42);
    assert_eq!(root2.as_ref(), &"hello");

    heap.collect(); // Roots should survive collection.
    println!("After first collection: {:?}", heap.stats());

    assert_eq!(root1.as_ref(), &42);
    assert_eq!(root2.as_ref(), &"hello");
}

#[test]
fn test_gc_heap_collect() {
    let mut heap = GcHeap::new();
    let root1 = heap.alloc(42);
    let root2 = heap.alloc("hello");

    // Check that the allocated values are correct
    assert_eq!(root1.as_ref(), &42);
    assert_eq!(root2.as_ref(), &"hello");

    drop(root1);
    drop(root2);

    heap.collect();
    println!("After first collection: {:?}", heap.stats());

    assert_eq!(heap.stats().allocated_objects, 0);
    assert_eq!(heap.stats().allocated_bytes, 0);
}

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

struct DropCounter;

impl Trace for DropCounter {
    fn trace(&self, _tracer: &mut super::Tracer) {}
    fn trace_in_heap(&self) {}
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        DROP_COUNT.with(|count| {
            let current = count.get();
            count.set(current + 1);
        });
    }
}

#[test]
fn test_gc_heap_drop() {
    clear_drop_count();

    let mut heap = GcHeap::new();
    let root = heap.alloc(DropCounter);

    drop(root);
    heap.collect();

    assert_eq!(drop_count(), 1);
}

struct Node {
    next: RefCell<Option<Gc<Node>>>,
    data: u32,
}

impl Node {
    fn new(data: u32) -> Self {
        Node {
            next: RefCell::new(None),
            data,
        }
    }

    fn with_next(self, next: Gc<Node>) -> Self {
        self.next.replace(Some(next));
        self
    }

    fn next(&self) -> Option<Gc<Node>> {
        self.next.borrow().clone()
    }
}

impl Trace for Node {
    fn trace(&self, tracer: &mut super::Tracer) {
        if let Some(ref next) = self.next.borrow().as_ref() {
            next.trace(tracer);
        }
    }

    fn trace_in_heap(&self) {
        if let Some(ref next) = self.next.borrow().as_ref() {
            next.trace_in_heap();
        }
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
fn test_gc_heap_single_level() {
    clear_drop_count();

    let mut heap = GcHeap::new();

    let root = {
        // Ensure the stack drops leaf before collect.
        let leaf = heap.alloc(Node::new(42));
        heap.alloc(Node::new(7).with_next(leaf))
    };

    heap.collect();
    assert_eq!(drop_count(), 0);

    // Leaf node should not be dropped because it is reachable from the root.
    assert_eq!(root.as_ref().data, 7);
    assert_eq!(root.as_ref().next().unwrap().as_ref().data, 42);
}

#[test]
fn test_gc_heap_cycle_detection() {
    clear_drop_count();

    let mut heap = GcHeap::new();

    let node1 = heap.alloc(Node::new(7));
    let node2 = heap.alloc(Node::new(11).with_next(node1.clone()));
    node1.as_ref().next.replace(Some(node2.clone()));

    heap.collect();
    assert_eq!(drop_count(), 0);
}
