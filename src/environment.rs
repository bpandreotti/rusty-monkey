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

    // pub fn set_outer(&mut self, outer: &EnvHandle) {
    //     match &self.outer {
    //         Some(env) => env.borrow_mut().set_outer(outer),
    //         None => self.outer = Some(Rc::clone(outer)),
    //     }
    // }

    pub fn insert(&mut self, key: String, value: Object) {
        self.map.insert(key, value);
    }

    // @TODO: Consider returning objects by reference instead of cloning. I would need to use an
    // `Rc<_>`, and restructure most of `eval::eval_expression`
    pub fn get(&self, key: &str) -> Option<Object> {
        self.map.get(key).cloned().or(match &self.outer {
            Some(e) => e.borrow().get(key),
            None => None,
        })
    }
}
