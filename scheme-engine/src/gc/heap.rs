use std::cell::Cell;
use std::ptr::NonNull;

use crate::gc::trace::{vtable_of, Trace, Tracer, VTable};

use super::Root;

// ========================================================================== //
//                                                                            //
// ██   ██ ███████  █████  ██████                                             //
// ██   ██ ██      ██   ██ ██   ██                                            //
// ███████ █████   ███████ ██████                                             //
// ██   ██ ██      ██   ██ ██                                                 //
// ██   ██ ███████ ██   ██ ██                                                 //
//                                                                            //
// ========================================================================== //

#[derive(Default, Debug)]
struct GcStats {
    allocated: usize, // bytes
}

pub struct GcHeap {
    stats: GcStats,
    head: Option<NonNull<ErasedBox>>, // linked-list
}

impl GcHeap {
    pub fn new() -> Self {
        GcHeap {
            stats: GcStats::default(),
            head: None,
        }
    }

    pub fn alloc<T>(&mut self, data: T) -> Root<T>
    where
        T: Trace + 'static,
    {
        let boxed = Box::new(GcBox {
            header: GcHeader {
                next: Cell::new(self.head),
                vtable: vtable_of::<T>(),
                strong_count: Cell::new(1),
                in_heap_count: Cell::new(0),
                color: Cell::new(GcColor::White),
            },
            data,
        });
        self.stats.allocated += std::mem::size_of::<GcBox<T>>();

        let ptr = NonNull::new(Box::into_raw(boxed)).expect("Box::into_raw returned null");
        self.head = Some(ptr.cast());

        Root { ptr }
    }

    pub fn collect(&mut self) {
        let mut collector = Collector::new();

        unsafe {
            collector.collect(self);
        }
    }
}

impl Drop for GcHeap {
    fn drop(&mut self) {
        self.collect(); // Ensure all allocated memory is freed when the heap is dropped
    }
}

// ========================================================================== //
//                                                                            //
//  ██████  ██████  ██      ██      ███████  ██████ ████████                  //
// ██      ██    ██ ██      ██      ██      ██         ██                     //
// ██      ██    ██ ██      ██      █████   ██         ██                     //
// ██      ██    ██ ██      ██      ██      ██         ██                     //
//  ██████  ██████  ███████ ███████ ███████  ██████    ██                     //
//                                                                            //
// ========================================================================== //

/// Mark-and-sweep garbage collector.
struct Collector {
    tracer: Tracer,
}

impl Collector {
    fn new() -> Self {
        Collector {
            tracer: Tracer::new(),
        }
    }

    /// The `collect` method performs the mark-and-sweep garbage collection algorithm.
    unsafe fn collect<'a>(&mut self, gc: &mut GcHeap) {
        // Mark phase
        if let Some(start) = gc.head {
            self.mark(start);
        }

        // Sweep phase
        self.sweep(gc);
    }

    fn mark(&mut self, start: NonNull<ErasedBox>) {
        // Trace the heap and mark all reachable objects.
        self.mark_in_heap(start);

        self.mark_roots(start);

        while let Some(ptr) = self.tracer.try_dequeue() {
            unsafe {
                if ptr.as_ref().header().color.get() == GcColor::Gray {
                    continue; // Already marked
                }

                if ptr.as_ref().header().is_rooted() {
                    ptr.as_ref().header().color.set(GcColor::Gray);
                }

                // Trace the node's outgoing references
                let vtable = ptr.as_ref().header().vtable;
                (vtable.trace_fn)(ptr, &mut self.tracer);
            }
        }

        self.tracer.clear();
    }

    /// Mark objects internally reachable.
    fn mark_in_heap(&mut self, start: NonNull<ErasedBox>) {
        let mut current = Some(start);
        while let Some(node) = current {
            let node_ref = unsafe { node.as_ref() };

            if node_ref.header().is_marked() {
                continue;
            }

            node_ref.header().mark();

            let vtable = node_ref.header().vtable;
            unsafe {
                (vtable.trace_in_heap_fn)(node);
            }

            current = node_ref.header().next.get();
        }
    }

    /// Traverse the object graph starting from the roots and mark all reachable objects.
    fn mark_roots(&mut self, start: NonNull<ErasedBox>) {
        self.tracer.clear();

        // Enqueue all roots according to referencing counting.
        let mut current = Some(start);
        while let Some(node) = current {
            let node_ref = unsafe { node.as_ref() };

            if node_ref.header().is_rooted() {
                self.tracer.enqueue(node);
            }

            current = node_ref.header().next.get();
        }
    }

    fn sweep(&mut self, gc: &mut GcHeap) {
        let mut current: Option<NonNull<ErasedBox>> = gc.head;
        let mut prev: Option<NonNull<ErasedBox>> = None;

        while let Some(ptr) = current {
            unsafe {
                let header = ptr.as_ref().header();
                if header.color.get() == GcColor::Gray {
                    // Node is reachable, reset color to white for next collection
                    header.color.set(GcColor::White);
                    prev = current;
                    current = header.next.get();
                } else {
                    // Node is unreachable, deallocate it
                    let next = header.next.get();
                    if let Some(mut prev_ptr) = prev {
                        prev_ptr.as_mut().header().set_next(next);
                    } else {
                        gc.head = next; // Update head if the first node is collected
                    }

                    let vtable = header.vtable;
                    (vtable.drop_fn)(ptr); // Call the drop function to deallocate the node

                    current = next;
                }
            }
        }
    }
}

// ========================================================================== //
//                                                                            //
// ██████   ██████  ██   ██                                                   //
// ██   ██ ██    ██  ██ ██                                                    //
// ██████  ██    ██   ███                                                     //
// ██   ██ ██    ██  ██ ██                                                    //
// ██████   ██████  ██   ██                                                   //
//                                                                            //
// ========================================================================== //

/// Represents the "colors" of the mark-and-sweep algorithm.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum GcColor {
    /// The object has not been visited yet.
    #[default]
    White,
    /// The object is being processed.
    Gray,
    /// The object and all its outgoing edges have been processed.
    Black,
}

const MARK_MASK: u32 = 1 << 31;
const COUNT_MASK: u32 = !MARK_MASK;
const MAX_COUNT: u32 = COUNT_MASK;

#[repr(C)]
pub(crate) struct GcHeader {
    next: Cell<Option<NonNull<ErasedBox>>>,
    vtable: &'static VTable,
    strong_count: Cell<u32>,
    in_heap_count: Cell<u32>,
    color: Cell<GcColor>,
}

impl GcHeader {
    fn set_next(&self, next: Option<NonNull<ErasedBox>>) {
        self.next.set(next);
    }

    pub(super) fn incr_strong(&self) {
        // Ensure the strong count does not exceed the maximum internal heap count.
        let count = self.strong_count.get();
        if count == MAX_COUNT {
            panic!("strong_count overflow");
        }
        self.strong_count.set(count + 1);
    }

    pub(super) fn decr_strong(&self) {
        self.strong_count.set(self.strong_count.get() - 1);
    }

    pub(super) fn incr_in_heap(&self) {
        let count = self.in_heap_count.get();
        if count == MAX_COUNT {
            panic!("in_heap_count overflow");
        }
        self.in_heap_count.set(count + 1);
    }

    pub(super) fn clear_in_heap(&self) {
        self.in_heap_count.set(0);
    }

    pub fn is_marked(&self) -> bool {
        (self.in_heap_count.get() & MARK_MASK) != 0
    }

    pub fn mark(&self) {
        self.in_heap_count.set(self.in_heap_count.get() | MARK_MASK);
    }

    pub fn unmark(&self) {
        self.in_heap_count
            .set(self.in_heap_count.get() & COUNT_MASK);
    }

    pub(super) fn mark_color(&self, color: GcColor) {
        self.color.set(color);
    }

    fn is_rooted(&self) -> bool {
        self.strong_count.get() > 0
    }
}

#[repr(C)]
pub(super) struct GcBox<T: Trace + 'static> {
    header: GcHeader,
    data: T,
}

impl<T: Trace + 'static> GcBox<T> {
    pub fn header(&self) -> &GcHeader {
        &self.header
    }

    #[inline]
    pub fn as_ref(&self) -> &T {
        &self.data
    }
}

/// A typed-erased value that ensures an upcasted value is not traced.
pub(crate) struct Erased(());

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

pub(crate) type ErasedBox = GcBox<Erased>;
