mod gc_box;
mod gc_header;
mod gc_refcell;
mod heap;
mod pointer;
mod trace;

pub use self::heap::GcHeap;
pub use self::pointer::Gc;
pub use self::trace::{Trace, Tracer};

pub mod prelude {
    pub use crate::Gc;
    pub use crate::GcHeap;
    pub use crate::Trace;
    pub use crate::Tracer;
}

pub mod cell {
    pub use crate::gc_refcell::{GcBorrowError, GcBorrowMutError, GcMut, GcRef, GcRefCell};
}
