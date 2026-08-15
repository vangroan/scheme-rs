mod gc_box;
mod gc_header;
mod heap;
mod pointer;
mod trace;

#[cfg(test)]
mod tests;

pub use self::heap::GcHeap;
pub use self::pointer::Gc;
pub use self::trace::{Trace, Tracer};

pub mod prelude {
    pub use crate::Gc;
    pub use crate::Trace;
    pub use crate::Tracer;
}
