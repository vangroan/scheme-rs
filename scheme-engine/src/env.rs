//! Execution environment.
use std::rc::Rc;

use crate::error::{Error, Result};
use crate::expr::{Expr, NativeFunc, Proc};
use crate::symbol::{SymbolId, SymbolTable};
use crate::{declare_id, Handle};

declare_id!(
    /// Constant value identifier.
    pub struct ConstantId(u16)
);

declare_id!(
    /// Local variable location identifier.
    ///
    /// This is the offset within the local scope, relative to the
    /// call frame's starting position in the dynamic evaluation stack.
    ///
    /// Importantly not the static lexical scoping stack.
    ///
    /// Thus the absolute position of the local cannot be known, because its
    /// location is determined during runtime.
    pub struct LocalId(u8)
);

declare_id!(
    /// Up-value variable location identifier.
    ///
    /// This is the index of the up-value in the heap buffer
    /// of the closure.
    pub struct UpValueId(u8)
);

declare_id!(
    /// Procedure prototype location identifier.
    pub struct ProcId(u16)
);

pub struct Env {
    /// Parent environment.
    parent: Option<Handle<Env>>,

    /// Table of values which do not change during runtime.
    ///
    /// Includes literals like booleans, numbers and strings, but
    /// also defined procedures.
    constants: (),

    /// Table of values which can be mutated during runtime.
    ///
    /// Does not include procedures, but can include closure instances.
    variables: SymbolTable,
    var_values: Vec<Expr>,

    /// Table of procedure prototypes that were declared in this environment.
    pub(crate) procedures: Vec<Rc<Proc>>,
}

impl Env {
    /// Create a new empty environment.
    pub fn new() -> Self {
        Env {
            parent: None,
            constants: (),

            variables: SymbolTable::new(),
            var_values: Vec::new(),

            procedures: Vec::new(),
        }
    }

    pub fn with_parent(parent: Handle<Env>) -> Self {
        Env {
            parent: Some(parent),
            constants: (),

            variables: SymbolTable::new(),
            var_values: Vec::new(),

            procedures: Vec::new(),
        }
    }

    pub fn lookup_var(&self, name: &str) -> Option<&Expr> {
        self.resolve_var(name)
            .and_then(|symbol| self.get_var(symbol))
    }

    pub fn resolve_var(&self, name: &str) -> Option<SymbolId> {
        self.variables.resolve(name)
    }

    pub fn get_var(&self, symbol: SymbolId) -> Option<&Expr> {
        self.var_values.get(symbol.as_usize())
    }

    pub fn set_var(&mut self, symbol: SymbolId, value: Expr) -> Result<()> {
        if symbol.as_usize() < self.var_values.len() {
            self.var_values[symbol.as_usize()] = value;
            Ok(())
        } else {
            Err(Error::Reason(format!(
                "variable is not declared: {symbol:?}"
            )))
        }
    }

    pub fn intern_var(&mut self, name: &str) -> SymbolId {
        let symbol = self.variables.intern_symbol(name);
        grow_table(&mut self.var_values, symbol.as_usize());
        symbol
    }

    pub(crate) fn add_procedure(&mut self, procedure: Proc) -> ProcId {
        let index = self.procedures.len();
        self.procedures.push(Rc::new(procedure));
        ProcId::new(index as u16)
    }

    /// TODO: Store argument arity information so it can be validated on compile or at runtime.
    pub fn bind_native_func(&mut self, name: &str, func: NativeFunc) -> Result<SymbolId> {
        match self.variables.insert_unique(name) {
            Some(symbol) => {
                grow_table(&mut self.var_values, symbol.as_usize());
                self.var_values[symbol.as_usize()] = Expr::NativeFunc(func);
                Ok(symbol)
            }
            None => Err(Error::Reason(format!("variable already bound {name:?}"))),
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

fn grow_table<T: Default>(table: &mut Vec<T>, index: usize) {
    if index >= table.len() {
        table.extend((table.len()..index + 1).map(|_| T::default()));
    }
}
