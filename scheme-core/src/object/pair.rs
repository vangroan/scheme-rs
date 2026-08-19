use scheme_gc::cell::GcRefCell;
use scheme_gc::{Gc, GcHeap, Trace};

use crate::value::Value;

pub struct PairObject(Gc<GcRefCell<PairInner>>);

impl PairObject {
    pub fn alloc(heap: &mut GcHeap, car: Value, cdr: Value) -> Self {
        let pair_inner = PairInner { car, cdr };
        let pair = heap.alloc(GcRefCell::new(pair_inner));
        PairObject(pair)
    }

    pub fn car(&self) -> Value {
        self.0.borrow().car.clone()
    }

    pub fn set_car(&self, value: Value) {
        self.0.borrow_mut().car = value;
    }

    pub fn cdr(&self) -> Value {
        self.0.borrow().cdr.clone()
    }

    pub fn set_cdr(&self, value: Value) {
        self.0.borrow_mut().cdr = value;
    }
}

impl Clone for PairObject {
    fn clone(&self) -> Self {
        PairObject(Gc::clone(&self.0))
    }
}

impl Into<Value> for PairObject {
    fn into(self) -> Value {
        Value::Pair(self)
    }
}

impl Trace for PairObject {
    fn trace(&self, tracer: &mut scheme_gc::Tracer) {
        self.0.trace(tracer);
    }

    fn trace_in_heap(&self) {
        self.0.trace_in_heap();
    }
}

struct PairInner {
    car: Value,
    cdr: Value,
}

impl Trace for PairInner {
    fn trace(&self, tracer: &mut scheme_gc::Tracer) {
        self.car.trace(tracer);
        self.cdr.trace(tracer);
    }

    fn trace_in_heap(&self) {
        self.car.trace_in_heap();
        self.cdr.trace_in_heap();
    }
}
