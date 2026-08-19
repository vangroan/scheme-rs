use crate::declare_id;

declare_id!(pub struct SymbolId(u16));

/// Table of interned symbol strings.
pub struct SymbolTable {
    /// List of symbols in this table.
    ///
    /// The index of a symbol in this list is its [`SymbolId`].
    symbols: Vec<Box<str>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Resolve a symbol name to its [`SymbolId`].
    pub fn resolve(&self, name_query: impl AsRef<str>) -> Option<SymbolId> {
        let name_query = name_query.as_ref();
        self.symbols
            .iter()
            .position(|name| name.as_ref() == name_query)
            .map(|index| SymbolId(index as u16))
    }

    /// Intern a symbol name and return its [`SymbolId`].
    ///
    /// If the symbol name already exists in the table, its existing [`SymbolId`] is returned.
    /// If the symbol name does not exist, it is added to the table and a new [`SymbolId`] is returned.
    pub fn intern(&mut self, name: impl ToString) -> SymbolId {
        let name: String = name.to_string();

        match self.resolve(name.as_str()) {
            Some(symbol) => symbol,
            None => {
                let next_index = self.symbols.len();
                self.symbols.push(name.into_boxed_str());
                SymbolId(next_index as u16)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Iterate over all symbols in the table, yielding their [`SymbolId`] and name.
    pub fn iter(&self) -> impl Iterator<Item = (SymbolId, &str)> {
        self.symbols
            .iter()
            .enumerate()
            .map(|(index, name)| (SymbolId(index as u16), name.as_ref()))
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intern_returns_new_ids_for_new_symbols() {
        let mut table = SymbolTable::new();

        let foo = table.intern("foo");
        let bar = table.intern("bar");

        assert_eq!(foo, SymbolId(0));
        assert_eq!(bar, SymbolId(1));
    }

    #[test]
    fn intern_returns_existing_id_for_duplicate_symbol() {
        let mut table = SymbolTable::new();

        let first = table.intern("alpha");
        let second = table.intern("alpha");

        assert_eq!(first, second);
        assert_eq!(first, SymbolId(0));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn is_empty_and_len_reflect_interned_symbols() {
        let mut table = SymbolTable::new();

        assert!(table.is_empty());
        assert_eq!(table.len(), 0);

        table.intern("x");
        assert!(!table.is_empty());
        assert_eq!(table.len(), 1);

        table.intern("x");
        assert_eq!(table.len(), 1);

        table.intern("y");
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn resolve_finds_existing_symbol_and_missing_returns_none() {
        let mut table = SymbolTable::new();
        let expected = table.intern("value");

        assert_eq!(table.resolve("value"), Some(expected));
        assert_eq!(table.resolve("missing"), None);
    }

    #[test]
    fn iter_returns_symbols_in_intern_order() {
        let mut table = SymbolTable::new();
        table.intern("first");
        table.intern("second");
        table.intern("third");

        let entries: Vec<(SymbolId, &str)> = table.iter().collect();
        assert_eq!(
            entries,
            vec![
                (SymbolId(0), "first"),
                (SymbolId(1), "second"),
                (SymbolId(2), "third"),
            ]
        );
    }
}
