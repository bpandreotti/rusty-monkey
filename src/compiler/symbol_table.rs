use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub enum SymbolScope {
    Global,
    Local,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Symbol {
    pub scope: SymbolScope,
    pub index: usize,
}

#[derive(Clone)]
pub struct SymbolTable {
    pub outer: Option<Box<SymbolTable>>,
    store: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            outer: None,
            store: HashMap::new(),
        }
    }

    pub fn from_outer(outer: Box<SymbolTable>) -> SymbolTable {
        SymbolTable {
            outer: Some(outer),
            store: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String) -> &Symbol {
        let symbol = Symbol {
            scope: if self.outer.is_none() {
                SymbolScope::Global
            } else {
                SymbolScope::Local
            },
            index: self.store.len(),
        };
        self.store.insert(name.clone(), symbol);
        self.store.get(&name).unwrap()
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        self.store
            .get(name)
            .cloned()
            .or_else(|| self.outer.as_ref().and_then(|outer| outer.resolve(name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_global() {
        let mut global = SymbolTable::new();
        let a = global.define("a".into());
        assert_eq!(
            Symbol {
                scope: SymbolScope::Global,
                index: 0
            },
            *a
        );
        let b = global.define("b".into());
        assert_eq!(
            Symbol {
                scope: SymbolScope::Global,
                index: 1
            },
            *b
        );
    }

    #[test]
    fn test_resolve_global() {
        let mut global = SymbolTable::new();
        global.define("a".into());
        global.define("b".into());
        assert_eq!(
            Symbol {
                scope: SymbolScope::Global,
                index: 0
            },
            global.resolve("a").unwrap()
        );
        assert_eq!(
            Symbol {
                scope: SymbolScope::Global,
                index: 1
            },
            global.resolve("b").unwrap()
        );
    }
}
