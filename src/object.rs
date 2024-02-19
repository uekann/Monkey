use anyhow::{bail, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Null,
    ReturnValue(Box<Object>),
}

impl Object {
    pub fn cast_to_boolean(&self) -> Result<Object> {
        match self {
            Object::Integer(i) => Ok(Object::Boolean(*i != 0)),
            Object::Boolean(b) => Ok(Object::Boolean(*b)),
            Object::Null => Ok(Object::Boolean(false)),
            Object::ReturnValue(obj) => obj.cast_to_boolean(),
            _ => bail!("cannot cast {:?} to boolean", self),
        }
    }
}

pub struct Environment {
    store: std::collections::HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Object> {
        self.store.get(name)
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.store.insert(name, value);
    }
}
