use crate::object::Object;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub type EnvHandle = Rc<RefCell<Environment>>;

#[derive(Clone, Debug)]
pub struct Environment {
    pub is_fn_context: bool,
    map: HashMap<String, Object>,
    outer: Option<EnvHandle>,
}

impl Environment {
    pub fn empty() -> Environment {
        Environment {
            map: HashMap::new(),
            outer: None,
            is_fn_context: false,
        }
    }

    pub fn extend(outer: &EnvHandle) -> Environment {
        Environment {
            map: HashMap::new(),
            outer: Some(Rc::clone(outer)),
            is_fn_context: outer.borrow().is_fn_context
        }
    }

    pub fn insert(&mut self, key: String, value: Object) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Object> {
        self.map.get(key).cloned().or(match &self.outer {
            Some(e) => e.borrow().get(key),
            None => None,
        })
    }
}
