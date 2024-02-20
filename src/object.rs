use crate::ast::{Expression, Statement};
use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Null,
    ReturnValue(Box<Object>),
    FunctionObject {
        parameters: Vec<Expression>,
        body: Statement,
        env: Environment,
    },
    FunctionApplication {
        function: Box<Object>,
        arguments: Vec<Object>,
    },
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

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    store: std::collections::HashMap<String, Object>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn get(&self, name: &str) -> Option<&Object> {
        match self.store.get(name) {
            Some(obj) => Some(obj),
            None => match &self.outer {
                Some(outer) => outer.get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.store.insert(name, value);
    }
}
