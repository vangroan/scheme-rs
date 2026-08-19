//! Global storage.

use std::cell::RefCell;
use std::rc::Rc;

use scheme_gc::cell::GcRefCell;
use scheme_gc::prelude::*;

use crate::object::Env;
use crate::symbol::SymbolTable;

pub struct Store {
    /// Interned symbol table.
    pub(crate) symbols: Rc<RefCell<SymbolTable>>,

    /// Managed garbage-collected heap.
    pub(crate) heap: Rc<RefCell<GcHeap>>,

    /// Global environment.
    pub(crate) env: Gc<GcRefCell<Env>>,
}

impl Store {
    pub fn new() -> Self {
        let symbols = Rc::new(RefCell::new(SymbolTable::new()));
        let mut heap = GcHeap::new();
        let env = heap.alloc(GcRefCell::new(Env::new()));

        Self {
            symbols,
            heap: Rc::new(RefCell::new(heap)),
            env,
        }
    }
}
