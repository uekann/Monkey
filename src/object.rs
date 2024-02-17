use crate::ast::*;
use crate::parser::Parser;
use anyhow::{bail, Result};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Null,
}

impl Object {
    pub fn cast_to_boolean(&self) -> Result<Object> {
        match self {
            Object::Integer(i) => Ok(Object::Boolean(*i != 0)),
            Object::Boolean(b) => Ok(Object::Boolean(*b)),
            Object::Null => Ok(Object::Boolean(false)),
            _ => bail!("cannot cast {:?} to boolean", self),
        }
    }
}
