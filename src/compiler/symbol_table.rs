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
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            outer: None,
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn from_outer(outer: Box<SymbolTable>) -> SymbolTable {
        SymbolTable {
            outer: Some(outer),
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: String) -> &Symbol {
        let scope = if self.outer.is_none() {
            SymbolScope::Global
        } else {
            SymbolScope::Local
        };
        let symbol = Symbol {
            scope,
            index: self.num_definitions,
        };
        self.num_definitions += 1;
        self.store.insert(name.clone(), symbol);
        self.store.get(&name).unwrap()
    }

    pub fn define_builtin(&mut self, name: String, index: usize) -> &Symbol {
        let symbol = Symbol {
            scope: SymbolScope::Builtin,
            index,
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

    #[test]
    fn test_resolve_builtin() {
        let builtins = &[("a", 0), ("b", 1), ("c", 2), ("d", 3)];
        let mut table = SymbolTable::new();
        for &(b, i) in builtins {
            table.define_builtin(b.into(), i);
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
