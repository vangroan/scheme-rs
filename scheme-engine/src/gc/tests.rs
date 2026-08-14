use super::{Gc, GcHeap, Root};

#[test]
fn test_gc_heap_alloc() {
    let mut heap = GcHeap::new();
    let root1 = heap.alloc(42);
    let root2 = heap.alloc("hello");

    // Check that the allocated values are correct
    assert_eq!(root1.as_ref(), &42);
    assert_eq!(root2.as_ref(), &"hello");

    heap.collect(); // Roots should survive collection.

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
}
