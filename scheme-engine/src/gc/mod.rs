mod heap;
mod pointer;
mod trace;

#[cfg(test)]
mod tests;

pub use heap::GcHeap;
pub use pointer::{Gc, Root};
