//! Heap allocated objects.

use scheme_gc::prelude::*;

#[repr(C)]
pub struct ConsObj(Gc<()>);
