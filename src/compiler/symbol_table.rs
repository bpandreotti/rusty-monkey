use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub enum SymbolScope {
    Builtin,
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

    pub fn define<S: Into<String>>(&mut self, name: S) -> &Symbol {
        self.define_with_scope(
            name.into(),
            if self.outer.is_none() {
                SymbolScope::Global
            } else {
                SymbolScope::Local
            },
        )
    }

    pub fn define_builtin<S: Into<String>>(&mut self, name: S) -> &Symbol {
        self.define_with_scope(name.into(), SymbolScope::Builtin)
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        self.store
            .get(name)
            .cloned()
            .or_else(|| self.outer.as_ref().and_then(|outer| outer.resolve(name)))
    }

    pub fn num_definitions(&self) -> usize {
        self.store.len()
    }

    fn define_with_scope(&mut self, name: String, scope: SymbolScope) -> &Symbol {
        let symbol = Symbol {
            scope,
            index: self.store.len(),
        };
        self.store.insert(name.clone(), symbol);
        self.store.get(&name).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_global() {
        let mut global = SymbolTable::new();
        let a = global.define("a");
        assert_eq!(
            Symbol {
                scope: SymbolScope::Global,
                index: 0
            },
            *a
        );
        let b = global.define("b");
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
        global.define("a");
        global.define("b");
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

    #[test]
    fn test_resolve_builtin() {
        let builtins = &[("a", 0), ("b", 1), ("c", 2), ("d", 3)];
        let mut table = SymbolTable::new();
        for &(b, _) in builtins {
            table.define_builtin(b);
        }

        for _ in 0..4 {
            table = SymbolTable::from_outer(Box::new(table));
            for &(name, index) in builtins {
                assert_eq!(
                    Some(Symbol {
                        scope: SymbolScope::Builtin,
                        index
                    }),
                    table.resolve(name),
                );
            }
        }
    }
}
