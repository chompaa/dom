//! Environment for storing and looking up variables.

use thiserror::Error;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::{Ident, Stmt};

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("identifier `{0}` cannot be redeclared")]
    Duplicate(String),
    #[error("identifier `{0}` used without declaration")]
    Declaration(String),
}

pub trait CloneableFn: FnMut(Vec<Val>, Rc<RefCell<Env>>) -> Option<Val> {
    fn clone_box<'a>(&self) -> Box<dyn 'a + CloneableFn>
    where
        Self: 'a;
}

impl<F> CloneableFn for F
where
    F: Fn(Vec<Val>, Rc<RefCell<Env>>) -> Option<Val> + Clone,
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
        env: Rc<RefCell<Env>>,
    },
    NativeFunc(Box<dyn CloneableFn>),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Val::None => write!(f, ""),
            Val::Bool(bool) => write!(f, "{bool}"),
            Val::Int(int) => write!(f, "{int}"),
            Val::Str(value) => write!(f, "{value}"),
            Val::Func { ident, params, .. } => write!(f, "{ident}({})", params.join(", ")),
            Val::NativeFunc(func) => write!(f, "{func:?}"),
        }
    }
}

/// An environment for storing and looking up variables.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// The parent environment, if any.
    parent: Option<Rc<RefCell<Env>>>,
    /// The values stored in this environment.
    values: HashMap<String, Val>,
}

impl Env {
    #[must_use]
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::default()))
    }

    /// Creates a new environment with the given parent environment.
    #[must_use]
    pub fn with_parent(parent: Rc<RefCell<Env>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: Some(parent),
            values: HashMap::new(),
        }))
    }

    /// Returns a reference to the values stored in this environment.
    #[must_use]
    pub fn values(&self) -> &HashMap<String, Val> {
        &self.values
    }

    /// Returns a mutable reference to the values stored in the environment.
    #[must_use]
    pub fn values_mut(&mut self) -> &mut HashMap<String, Val> {
        &mut self.values
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
    pub fn assign(env: &Rc<RefCell<Self>>, name: String, value: Val) -> Result<Val, EnvError> {
        // Find the environment where the variable is declared.
        let env = Self::resolve(env, &name)?;

        env.borrow_mut().values.insert(name, value.clone());

        Ok(value)
    }

    /// Looks up the value of the variable with the given name.
    ///
    /// Returns an error if no variable with the given name exists in this environment or its parents.
    pub fn lookup(env: &Rc<RefCell<Self>>, name: &str) -> Result<Val, EnvError> {
        // Find the environment where the variable is declared.
        let env = Self::resolve(env, name)?;
        let values = &env.borrow().values;
        let value = values
            .get(name)
            .expect("Environment should contain identifier");

        Ok(value.clone())
    }

    /// Resolves the environment that contains the variable with the given name.
    fn resolve(env: &Rc<RefCell<Self>>, name: &str) -> Result<Rc<RefCell<Env>>, EnvError> {
        if env.borrow().values.contains_key(name) {
            return Ok(Rc::clone(env));
        }

        match &env.borrow().parent {
            Some(parent) => Self::resolve(parent, name),
            None => Err(EnvError::Declaration(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for Val {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Val::Int(a), Val::Int(b)) => a == b,
                _ => false,
            }
        }
    }

    #[test]
    fn declare_and_lookup() {
        let env = Env::new();

        let name = "foo";
        let value = Val::Int(0);

        // Declare a variable in the environment
        env.borrow_mut()
            .declare(name.to_string(), value.clone())
            .expect("should be able to declare variable");

        // Lookup the variable
        let result = Env::lookup(&env, &name).expect("variable should exist");
        assert_eq!(result, value);
    }

    #[test]
    fn declare_error() {
        let env = Env::new();

        let name = "foo";
        let value = Val::Int(0);

        // Declare a variable in the environment
        env.borrow_mut()
            .declare(name.to_string(), value.clone())
            .expect("should be able to declare variable");

        // Attempt to redeclare the same variable
        let result = env.borrow_mut().declare(name.to_string(), value.clone());
        assert!(matches!(result, Err(EnvError::Duplicate(_))));
    }

    #[test]
    fn lookup_error() {
        let env = Env::new();

        // Attempt to lookup a non-existent variable
        let name = "foo";
        let result = Env::lookup(&env, &name);
        assert!(matches!(result, Err(EnvError::Declaration(_))));
    }

    #[test]
    fn assign_and_lookup() {
        let env = Env::new();

        let name = "foo";
        let value = Val::Int(0);

        // Declare a variable in the environment
        env.borrow_mut()
            .declare(name.to_string(), value.clone())
            .expect("should be able to declare variable");

        // Assign a new value to the variable
        let value = Val::Int(1);
        Env::assign(&env, name.to_string(), value.clone())
            .expect("should be able to assign value to variable");

        // Lookup the variable
        let result = Env::lookup(&env, &name).expect("should be able to lookup variable");
        assert_eq!(result, value);
    }

    #[test]
    fn nested_environments() {
        let parent_env = Env::new();

        let name = "foo";
        let value = Val::Int(0);

        // Declare a variable in the parent environment
        parent_env
            .borrow_mut()
            .declare(name.to_string(), value.clone())
            .expect("should be able to declare variable");

        // Create a child environment with the parent environment
        let child_env = Env::with_parent(Rc::clone(&parent_env));

        // Lookup the variable from the child environment
        let result = Env::lookup(&child_env, &name);
        assert_eq!(result.unwrap(), value.clone());

        // Declare a new variable in the parent environment
        let name = "bar";
        let value = Val::Int(0);
        parent_env
            .borrow_mut()
            .declare(name.to_string(), value.clone())
            .expect("should be able to declare variable");

        // Lookup the new variable from the child environment
        let result = Env::lookup(&child_env, &name).expect("should be able to lookup variable");
        assert_eq!(result, value);
    }
}
