//! Heap allocated objects.

use crate::gc::Gc;

#[repr(C)]
pub struct ConsObj(Gc<()>);
