use std::cell::Cell;
use std::ptr::NonNull;

use crate::gc::trace::{vtable_of, Trace, Tracer, VTable};
use crate::gc::Gc;

// ========================================================================== //
//                                                                            //
// ██   ██ ███████  █████  ██████                                             //
// ██   ██ ██      ██   ██ ██   ██                                            //
// ███████ █████   ███████ ██████                                             //
// ██   ██ ██      ██   ██ ██                                                 //
// ██   ██ ███████ ██   ██ ██                                                 //
//                                                                            //
// ========================================================================== //

const INITIAL_HEAP_SIZE: usize = 1024 * 1024 * 1024; // bytes

/// Statistics about the garbage collector's heap.
#[derive(Default, Debug, Clone)]
pub struct GcStats {
    /// The total number of bytes currently allocated in the heap.
    pub allocated_bytes: usize,

    /// The total number of objects currently allocated in the heap.
    pub allocated_objects: usize,

    /// The threshold in bytes at which the garbage collector will be triggered.
    pub threshhold_bytes: usize,
}

pub struct GcHeap {
    stats: GcStats,
    head: Option<NonNull<ErasedBox>>, // linked-list
}

impl GcHeap {
    pub fn new() -> Self {
        let stats = GcStats {
            threshhold_bytes: INITIAL_HEAP_SIZE,
            ..Default::default()
        };
        GcHeap { stats, head: None }
    }

    pub fn stats(&self) -> &GcStats {
        &self.stats
    }

    pub fn alloc<T>(&mut self, data: T) -> Gc<T>
    where
        T: Trace + 'static,
    {
        self.ensure_heap_size(self.stats.allocated_bytes + std::mem::size_of::<GcBox<T>>());

        let boxed = Box::new(GcBox {
            header: GcHeader {
                next: Cell::new(self.head),
                vtable: vtable_of::<T>(),
                strong_count: Cell::new(1),
                in_heap_count: Cell::new(0),
            },
            data,
        });
        self.stats.allocated_bytes += std::mem::size_of::<GcBox<T>>();
        self.stats.allocated_objects += 1;

        let ptr = NonNull::new(Box::into_raw(boxed)).expect("Box::into_raw returned null");
        self.head = Some(ptr.cast());

        Gc::new(ptr)
    }

    #[allow(dead_code, reason = "Used in unit tests")]
    pub(crate) fn set_collect_threshold(&mut self, threshold: usize) {
        self.stats.threshhold_bytes = threshold;
    }

    fn ensure_heap_size(&mut self, target_size: usize) {
        if target_size >= self.stats.threshhold_bytes
            || self.stats.allocated_bytes >= self.stats.threshhold_bytes
        {
            self.collect();
            self.stats.threshhold_bytes = self.stats.allocated_bytes * 2;
        }
    }

    pub fn collect(&mut self) {
        let mut collector = Collector::new();

        collector.collect(self);
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
    fn collect<'a>(&mut self, gc: &mut GcHeap) {
        // Mark phase
        if let Some(start) = gc.head {
            self.mark(start);
        }

        // Sweep phase
        self.sweep(gc);
    }

    fn mark(&mut self, start: NonNull<ErasedBox>) {
        // Trace the heap and mark internally reachable objects.
        self.mark_in_heap(start);

        // Trace the heap and mark externally reachable objects (roots).
        self.mark_roots(start);
    }

    /// Mark objects internally reachable.
    fn mark_in_heap(&mut self, start: NonNull<ErasedBox>) {
        let mut current = Some(start);
        while let Some(node) = current {
            let node_ref = unsafe { node.as_ref() };

            // SAFETY: The box must be created with a reference to the correct vtable.
            unsafe {
                (node_ref.vtable().trace_in_heap_fn)(node);
            }

            current = node_ref.next_node();
        }
    }

    /// Traverse the object graph starting from the roots and mark all reachable objects.
    fn mark_roots(&mut self, start: NonNull<ErasedBox>) {
        self.tracer.clear();

        // Enqueue all roots by comparing the external references (strong_count)
        // with the internal references (in_heap_count).
        let mut current = Some(start);
        while let Some(node) = current {
            let node_ref = unsafe { node.as_ref() };

            if node_ref.header().is_marked() {
                current = node_ref.next_node();
                continue;
            }

            if node_ref.header().is_rooted() {
                self.tracer.enqueue(node);
            }

            Self::trace_nodes_to_mark(&mut self.tracer);

            current = node_ref.next_node();
        }

        self.tracer.clear();
    }

    /// Process the queue of nodes to mark all reachable objects.
    ///
    /// Walking the object graph uses a queue instead of recusion to avoid
    /// overflowing the call stack for deeply nested object graphs.
    fn trace_nodes_to_mark(tracer: &mut Tracer) {
        while let Some(node) = tracer.try_dequeue() {
            let node_ref = unsafe { node.as_ref() };

            node_ref.header().mark();

            // SAFETY: The box must be created with a reference to the correct vtable.
            unsafe {
                (node_ref.vtable().trace_fn)(node, tracer);
            }
        }
    }

    fn sweep(&mut self, gc: &mut GcHeap) {
        let mut unreachable_nodes = Vec::new();

        let mut current: Option<NonNull<ErasedBox>> = gc.head;

        while let Some(node) = current {
            let node_ref = unsafe { node.as_ref() };

            if node_ref.header().is_rooted() {
                node_ref.header().unmark();
                node_ref.header().reset_in_heap();
            } else if !node_ref.header().is_marked() {
                // FIXME: Drop will recursively drop objects that are referenced by this vector.
                unreachable_nodes.push(node);
            }

            current = node_ref.next_node();
        }

        // Deallocate unreachable nodes after the sweep phase to avoid
        // invalidating the linked list during iteration.
        for node in unreachable_nodes {
            if gc.head == Some(node) {
                gc.head = unsafe { node.as_ref() }.next_node();
            }

            let node_ref = unsafe { node.as_ref() };

            // SAFETY: The box must be created with a reference to the correct vtable.
            unsafe {
                let node_size = (node_ref.vtable().size_fn)();
                gc.stats.allocated_bytes -= node_size;
                gc.stats.allocated_objects -= 1;

                (node_ref.vtable().drop_fn)(node);
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

const MARK_MASK: u32 = 1 << 31;
const COUNT_MASK: u32 = !MARK_MASK;
const MAX_COUNT: u32 = COUNT_MASK;

#[repr(C)]
pub(crate) struct GcHeader {
    next: Cell<Option<NonNull<ErasedBox>>>,
    vtable: &'static VTable,
    strong_count: Cell<u32>,
    in_heap_count: Cell<u32>,
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

    pub(super) fn reset_in_heap(&self) {
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

    fn is_rooted(&self) -> bool {
        // external-refs = total-refs - internal-refs
        self.in_heap_count.get() < self.strong_count.get()
    }
}

#[repr(C)]
pub(crate) struct GcBox<T: Trace + 'static> {
    header: GcHeader,
    data: T,
}

impl<T: Trace + 'static> GcBox<T> {
    pub fn header(&self) -> &GcHeader {
        &self.header
    }

    pub(crate) fn next_node(&self) -> Option<NonNull<ErasedBox>> {
        self.header.next.get()
    }

    pub(crate) fn vtable(&self) -> &'static VTable {
        self.header.vtable
    }

    #[inline]
    pub fn as_ref(&self) -> &T {
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
