use scheme_gc::{cell::GcRefCell, Gc, Trace};

use crate::object::{Env, PairObject};
use crate::symbol::SymbolId;

#[derive(Clone, Default)]
pub enum Value {
    #[default]
    Nil,
    Num(Number),
    Sym(SymbolId),
    Env(Gc<GcRefCell<Env>>),
    Pair(PairObject),
}

impl Trace for Value {
    fn trace(&self, tracer: &mut scheme_gc::Tracer) {
        match self {
            Value::Nil => {}
            Value::Num(_) => {}
            Value::Sym(_) => {}
            Value::Env(env) => env.trace(tracer),
            Value::Pair(pair) => pair.trace(tracer),
        }
    }

    fn trace_in_heap(&self) {
        match self {
            Value::Nil => {}
            Value::Num(_) => {}
            Value::Sym(_) => {}
            Value::Env(env) => env.trace_in_heap(),
            Value::Pair(pair) => pair.trace_in_heap(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Number {
    pub fn from_i64(n: i64) -> Self {
        Number::Integer(n)
    }

    pub fn from_f64(n: f64) -> Self {
        Number::Float(n)
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Number::Integer(n)
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Number::Float(n)
    }
}
