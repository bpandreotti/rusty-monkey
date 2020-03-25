use crate::object::Object;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Environment(HashMap<String, Object>);

impl Environment {
    pub fn empty() -> Environment {
        Environment(HashMap::new())
    }

    pub fn insert(&mut self, key: String, value: Object) {
        self.0.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Object> {
        self.0.get(key)
    }
}
