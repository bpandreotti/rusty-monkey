use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub enum SymbolScope {
    Global,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Symbol {
    pub scope: SymbolScope,
    pub index: usize,
}

pub struct SymbolTable(HashMap<String, Symbol>);

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable(HashMap::new())
    }

    pub fn define(&mut self, name: String) -> &Symbol {
        let symbol = Symbol {
            scope: SymbolScope::Global,
            index: self.0.len(),
        };
        self.0.insert(name.clone(), symbol);
        self.0.get(&name).unwrap()
    }

    pub fn resolve(&self, name: &String) -> Option<Symbol> {
        self.0.get(name).cloned()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_global() {
        let mut global = SymbolTable::new();
        let a = global.define("a".into());
        assert_eq!(Symbol { scope: SymbolScope::Global, index: 0 }, *a);
        let b = global.define("b".into());
        assert_eq!(Symbol { scope: SymbolScope::Global, index: 1 }, *b);        
    }

    #[test]
    fn test_resolve_global() {
        let mut global = SymbolTable::new();
        global.define("a".into());
        global.define("b".into());
        assert_eq!(
            Symbol { scope: SymbolScope::Global, index: 0 },
            global.resolve(&"a".into()).unwrap()
        );
        assert_eq!(
            Symbol { scope: SymbolScope::Global, index: 1 },
            global.resolve(&"b".into()).unwrap()
        );
    }
}
