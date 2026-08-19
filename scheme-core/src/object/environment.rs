//! Environment object.

use scheme_gc::prelude::*;

use crate::object::object::HeapObject;

pub struct Env {}

impl Env {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl HeapObject for Env {}

impl Trace for Env {
    fn trace(&self, tracer: &mut Tracer) {}

    fn trace_in_heap(&self) {}
}
