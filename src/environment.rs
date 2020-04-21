use crate::builtins::*;
use crate::object::Object;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvHandle = Rc<RefCell<Environment>>;

#[derive(Clone, Debug)]
pub struct Environment {
    map: HashMap<String, Object>,
    outer: Option<EnvHandle>,
}

impl Environment {
    pub fn empty() -> Environment {
        Environment {
            map: HashMap::new(),
            outer: None,
        }
    }

    pub fn extend(outer: &EnvHandle) -> Environment {
        Environment {
            map: HashMap::new(),
            outer: Some(Rc::clone(outer)),
        }
    }

    pub fn insert(&mut self, key: String, value: Object) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Object> {
        self.map
            .get(key) // Try to find the identifier in the environment
            .cloned()
            // if that fails, try the outer environment (if it exists)
            .or_else(|| self.outer.as_ref().and_then(|e| e.borrow().get(key)))
            // and finally, try the built-in functions
            .or_else(|| get_builtin(key))
    }
}
