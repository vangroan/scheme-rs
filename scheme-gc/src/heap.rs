use std::alloc::Layout;
use std::ptr::NonNull;

use crate::gc_box::{ErasedBox, GcBox};
use crate::gc_header::GcHeader;
use crate::pointer::Gc;
use crate::trace::{Trace, Tracer};

// ========================================================================== //
//                                                                            //
// ██   ██ ███████  █████  ██████                                             //
// ██   ██ ██      ██   ██ ██   ██                                            //
// ███████ █████   ███████ ██████                                             //
// ██   ██ ██      ██   ██ ██                                                 //
// ██   ██ ███████ ██   ██ ██                                                 //
//                                                                            //
// ========================================================================== //

const MIN_HEAP_SIZE: usize = 1024 * 1024 * 1024; // bytes

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
            threshhold_bytes: MIN_HEAP_SIZE,
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

        let layout = Layout::new::<GcBox<T>>();
        assert!(layout.size() > 0, "Cannot allocate zero-sized type");

        // SAFTEY: Layout must not be zero-sized.
        let ptr = NonNull::new(unsafe { std::alloc::alloc(layout) })
            .expect("Allocation failed")
            .cast::<GcBox<T>>();

        let gc_box = GcBox::new(GcHeader::new::<T>(self.head), data);
        gc_box.header().incr_strong();

        // SAFETY: The pointer is valid and properly aligned for the type.
        // The old contents is uninitialized, so doesn't need drop.
        unsafe {
            ptr.as_ptr().write(gc_box);
        }

        self.stats.allocated_bytes += std::mem::size_of::<GcBox<T>>();
        self.stats.allocated_objects += 1;

        self.head = Some(ptr.cast());

        Gc::new(ptr)
    }

    #[allow(dead_code, reason = "Used in unit tests")]
    pub(crate) fn set_collect_threshold(&mut self, threshold: usize) {
        self.stats.threshhold_bytes = threshold;
    }

    fn ensure_heap_size(&mut self, target_size: usize) {
        if target_size >= self.stats.threshhold_bytes {
            self.collect();
            self.stats.threshhold_bytes = self.stats.allocated_bytes * 2;
        }
    }

    fn shrink_heap_size(&mut self) {
        if self.stats.allocated_bytes < self.stats.threshhold_bytes / 4 {
            self.stats.threshhold_bytes = std::cmp::max(
                MIN_HEAP_SIZE,
                std::cmp::max(
                    self.stats.allocated_bytes * 2,
                    self.stats.threshhold_bytes / 2,
                ),
            );
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
        let mut prev: Option<NonNull<ErasedBox>> = None;

        while let Some(node) = current {
            let node_ref = unsafe { node.as_ref() };
            let next = node_ref.next_node();

            if node_ref.header().is_marked() || node_ref.header().is_rooted() {
                node_ref.header().unmark();
                node_ref.header().reset_in_heap();
                prev = Some(node);
            } else {
                if let Some(prev_node) = prev {
                    unsafe { prev_node.as_ref() }.header().set_next(next);
                } else {
                    gc.head = next;
                }

                unreachable_nodes.push(node);
            }

            current = next;
        }

        // Drop inner data first while all headers remain valid. This avoids
        // use-after-free when dropping one object recursively drops Gc fields
        // that still point at other unreachable nodes.
        for &node in &unreachable_nodes {
            let node_ref = unsafe { node.as_ref() };

            // SAFETY: The box must be created with a reference to the correct vtable.
            unsafe {
                (node_ref.vtable().drop_data_fn)(node);
            }
        }

        // Then release backing allocations without running destructors again.
        for node in unreachable_nodes {
            let node_ref = unsafe { node.as_ref() };

            // SAFETY: The box must be created with a reference to the correct vtable.
            unsafe {
                let node_size = (node_ref.vtable().size_fn)();
                gc.stats.allocated_bytes -= node_size;
                gc.stats.allocated_objects -= 1;

                (node_ref.vtable().dealloc_fn)(node);
            }
        }

        gc.shrink_heap_size();
    }
}
