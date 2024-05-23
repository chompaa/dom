//! Environment for storing and looking up variables.

use thiserror::Error;

use std::collections::HashMap;

use crate::ast::{Ident, Stmt};

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("identifier `{0}` cannot be redeclared")]
    Duplicate(String),
    #[error("identifier `{0}` used without declaration")]
    Declaration(String),
}

#[derive(Debug, Clone)]
pub enum Val {
    Int(i32),
    Func {
        ident: Ident,
        params: Vec<Ident>,
        body: Vec<Stmt>,
        env: Env,
    },
}

/// An environment for storing and looking up variables.
#[derive(Debug, Clone)]
pub struct Env {
    /// The parent environment, if any.
    parent: Option<Box<Env>>,
    /// The values stored in this environment.
    values: HashMap<String, Val>,
}

impl Env {
    /// Creates a new, empty environment.
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
        }
    }

    /// Creates a new environment with the given parent environment.
    pub fn with_parent(parent: Env) -> Self {
        Self {
            parent: Some(Box::new(parent)),
            values: HashMap::new(),
        }
    }

    /// Declares a new variable with the given name and value.
    ///
    /// Returns an error if a variable with the same name already exists in this environment.
    pub fn declare(&mut self, name: String, value: Val) -> Result<Val, EnvError> {
        // Check if a variable with the same name already exists in this environment.
        if self.values.contains_key(&name) {
            return Err(EnvError::Duplicate(name));
        }

        self.values.insert(name, value.clone());

        Ok(value)
    }

    /// Assigns a new value to the variable with the given name.
    ///
    /// Returns an error if no variable with the given name exists in this environment or its parents.
    pub fn assign(&mut self, name: String, value: Val) -> Result<Val, EnvError> {
        // Find the environment where the variable is declared.
        let env = self.resolve_mut(&name)?;

        env.values.insert(name, value.clone());

        Ok(value)
    }

    /// Looks up the value of the variable with the given name.
    ///
    /// Returns an error if no variable with the given name exists in this environment or its parents.
    pub fn lookup(&self, name: String) -> Result<Val, EnvError> {
        // Find the environment where the variable is declared.
        let env = self.resolve(&name)?;

        let value = env
            .values
            .get(&name)
            .expect("Environment should contain identifier");

        Ok(value.clone())
    }

    /// Resolves the environment that contains the variable with the given name.
    fn resolve(&self, name: &str) -> Result<&Env, EnvError> {
        if self.values.contains_key(name) {
            return Ok(self);
        }

        if let Some(parent) = &self.parent {
            return parent.resolve(name);
        }

        Err(EnvError::Declaration(name.to_string()))
    }

    /// Resolves the mutable environment that contains the variable with the given name.
    fn resolve_mut(&mut self, name: &str) -> Result<&mut Env, EnvError> {
        // If the variable is declared in this environment, return this environment.
        if self.values.contains_key(name) {
            return Ok(self);
        }

        // If there is a parent environment, recursively search for the variable there.
        if let Some(parent) = &mut self.parent {
            return parent.resolve_mut(name);
        }

        // If no environment contains the variable, return an error.
        Err(EnvError::Declaration(name.to_string()))
    }
}
