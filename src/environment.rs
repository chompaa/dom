//! Environment for storing and looking up variables.

use crate::interpreter::Val;
use std::collections::HashMap;

/// An environment for storing and looking up variables.
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
    pub fn from_parent(parent: Env) -> Self {
        Self {
            parent: Some(Box::new(parent)),
            values: HashMap::new(),
        }
    }

    /// Declares a new variable with the given name and value.
    ///
    /// Returns an error if a variable with the same name already exists in this environment.
    pub fn declare(&mut self, name: String, value: Val) -> Result<Val, ()> {
        // Check if a variable with the same name already exists in this environment.
        if self.values.contains_key(&name) {
            return Err(());
        }

        self.values.insert(name, value);

        Ok(value)
    }

    /// Assigns a new value to the variable with the given name.
    ///
    /// Returns an error if no variable with the given name exists in this environment or its parents.
    pub fn assign(&mut self, name: String, value: Val) -> Result<Val, ()> {
        // Find the environment where the variable is declared.
        let env = self.resolve_mut(&name)?;

        env.values.insert(name, value);

        Ok(value)
    }

    /// Looks up the value of the variable with the given name.
    ///
    /// Returns an error if no variable with the given name exists in this environment or its parents.
    pub fn lookup(&self, name: String) -> Result<Val, ()> {
        // Find the environment where the variable is declared.
        let env = self.resolve(&name)?;

        // Get the value of the variable from the environment.
        let Some(value) = env.values.get(&name) else {
            return Err(());
        };

        Ok(*value)
    }

    /// Resolves the environment that contains the variable with the given name.
    fn resolve(&self, name: &str) -> Result<&Env, ()> {
        if self.values.contains_key(name) {
            return Ok(self);
        }

        if let Some(parent) = &self.parent {
            return parent.resolve(name);
        }

        Err(())
    }

    /// Resolves the mutable environment that contains the variable with the given name.
    fn resolve_mut(&mut self, name: &str) -> Result<&mut Env, ()> {
        // If the variable is declared in this environment, return this environment.
        if self.values.contains_key(name) {
            return Ok(self);
        }

        // If there is a parent environment, recursively search for the variable there.
        if let Some(parent) = &mut self.parent {
            return parent.resolve_mut(name);
        }

        // If no environment contains the variable, return an error.
        Err(())
    }
}
