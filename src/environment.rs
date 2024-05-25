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

pub trait CloneableFn: Fn(Vec<Val>, &mut Env) -> Option<Val> {
    fn clone_box<'a>(&self) -> Box<dyn 'a + CloneableFn>
    where
        Self: 'a;
}

impl<F> CloneableFn for F
where
    F: Fn(Vec<Val>, &mut Env) -> Option<Val> + Clone,
{
    fn clone_box<'a>(&self) -> Box<dyn 'a + CloneableFn>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}

impl<'a> Clone for Box<dyn 'a + CloneableFn> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl<'a> std::fmt::Debug for Box<dyn 'a + CloneableFn> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NativeFunc")
    }
}

#[derive(Debug, Clone)]
pub enum Val {
    None,
    Bool(bool),
    Int(i32),
    Str(String),
    Func {
        ident: Ident,
        params: Vec<Ident>,
        body: Vec<Stmt>,
        env: Env,
    },
    NativeFunc(Box<dyn CloneableFn>),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Val::None => write!(f, ""),
            Val::Bool(bool) => writeln!(f, "{bool}"),
            Val::Int(int) => writeln!(f, "{int}"),
            Val::Str(value) => writeln!(f, "{value}"),
            Val::Func { ident, params, .. } => writeln!(f, "{ident}({})", params.join(", ")),
            Val::NativeFunc(func) => writeln!(f, "{func:?}"),
        }
    }
}

/// An environment for storing and looking up variables.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// The parent environment, if any.
    parent: Option<Box<Env>>,
    /// The values stored in this environment.
    values: HashMap<String, Val>,
}

impl Env {
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
