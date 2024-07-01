//! Environment for storing and looking up variables.

use miette::{Diagnostic, Result, SourceSpan};
use thiserror::Error;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::ast::{Ident, Stmt};

#[derive(Error, Diagnostic, Debug)]
pub enum EnvError {
    #[error("identifier cannot be redeclared")]
    #[diagnostic(code(environment::identifier_already_exists))]
    IdentifierAlreadyExists {
        #[label("this identifier already exists")]
        span: SourceSpan,
    },
    #[error("identifier not found")]
    #[diagnostic(code(environment::identifier_not_found))]
    IdentifierNotFound {
        #[label("this identifier was never defined")]
        span: SourceSpan,
    },
}

pub trait CloneableFn: FnMut(&[Val], &Arc<Mutex<Env>>) -> Option<Val> {
    fn clone_box<'a>(&self) -> Box<dyn CloneableFn + Send + Sync + 'static>
    where
        Self: 'a;
}

impl<F> CloneableFn for F
where
    F: Fn(&[Val], &Arc<Mutex<Env>>) -> Option<Val> + Send + Sync + Clone + 'static,
{
    fn clone_box<'a>(&self) -> Box<dyn CloneableFn + Send + Sync + 'static>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CloneableFn + Send + Sync + 'static> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl std::fmt::Debug for Box<dyn CloneableFn + Send + Sync + 'static> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NativeFunc")
    }
}

/// Runtime values.
#[derive(Debug, Clone)]
pub struct Val {
    /// The identifier of the value (if stored in an environment)
    pub ident: Option<Ident>,
    pub kind: ValKind,
}

impl From<ValKind> for Val {
    fn from(value: ValKind) -> Self {
        Self {
            ident: None,
            kind: value,
        }
    }
}

impl Val {
    pub const NONE: Self = Val {
        ident: None,
        kind: ValKind::None,
    };

    pub fn from_kind(kind: ValKind) -> Self {
        Self { ident: None, kind }
    }

    pub fn with_ident(mut self, ident: Ident) -> Self {
        self.ident = Some(ident);
        self.to_owned()
    }
}

/// Value kinds.
#[derive(Debug, Clone)]
pub enum ValKind {
    /// Empty value.
    None,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i32),
    /// String value.
    Str(String),
    /// User-defined function.
    Func {
        ident: Ident,
        params: Vec<Ident>,
        body: Vec<Stmt>,
        env: Arc<Mutex<Env>>,
    },
    /// Built-in function.
    NativeFunc(Box<dyn CloneableFn + Send + Sync>),
    List(Vec<Val>),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.kind {
            ValKind::None => write!(f, ""),
            ValKind::Bool(bool) => write!(f, "{bool}"),
            ValKind::Int(int) => write!(f, "{int}"),
            ValKind::Str(value) => write!(f, "{value}"),
            ValKind::Func { ident, params, .. } => write!(f, "{ident}({})", params.join(", ")),
            ValKind::NativeFunc(func) => write!(f, "{func:?}"),
            ValKind::List(items) => {
                // We shouldn't use `join` here, since we'd need to map every item
                // using the `format` macro, and then collect
                write!(f, "[")?;
                for (idx, item) in items.iter().enumerate() {
                    write!(f, "{item}")?;
                    if idx < items.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}

/// An environment for storing and looking up variables.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// The parent environment, if any.
    parent: Option<Arc<Mutex<Env>>>,
    /// The values stored in this environment.
    values: HashMap<String, Val>,
}

impl Env {
    #[must_use]
    /// Creates a new environment.
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    /// Creates a new environment with the given parent environment.
    #[must_use]
    pub fn with_parent(parent: Arc<Mutex<Env>>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
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
    pub fn declare(&mut self, name: String, value: Val, span: SourceSpan) -> Result<Val> {
        // Check if a variable with the same name already exists in this environment.
        if self.values.contains_key(&name) {
            return Err(EnvError::IdentifierAlreadyExists { span }.into());
        }

        self.values.insert(name.clone(), value.clone());

        Ok(value.with_ident(name))
    }

    /// Declares a new variable with the given name and value, overwritting any variable that
    /// might exist.
    ///
    /// Does not return anything.
    pub fn declare_unchecked(&mut self, name: String, value: Val) {
        self.values.insert(name, value.clone());
    }

    /// Assigns a new value to the variable with the given name.
    ///
    /// Returns an error if no variable with the given name exists in this environment or its parents.
    pub fn assign(
        env: &Arc<Mutex<Self>>,
        name: String,
        value: Val,
        span: SourceSpan,
    ) -> Result<Val> {
        // Find the environment where the variable is declared.
        let env = Self::resolve(env, &name, span)?;

        env.lock()
            .unwrap()
            .values
            .insert(name.clone(), value.clone());

        Ok(value.with_ident(name))
    }

    /// Assigns a new value to the variable with the given name. Does not perform error
    /// checking.
    pub fn assign_unchecked(env: &Arc<Mutex<Self>>, name: String, value: Val) {
        // Find the environment where the variable is declared.
        let env = Self::resolve(env, &name, (0, 0).into()).unwrap();

        env.lock().unwrap().values.insert(name, value);
    }

    /// Looks up the value of the variable with the given name.
    ///
    /// Returns an error if no variable with the given name exists in this environment or its parents.
    pub fn lookup(env: &Arc<Mutex<Self>>, name: &str, span: SourceSpan) -> Result<Val> {
        // Find the environment where the variable is declared.
        let env = Self::resolve(env, name, span)?;
        let values = &env.lock().unwrap().values;
        let value = values
            .get(name)
            .expect("variable should have a value")
            .clone();

        Ok(value.with_ident(name.to_string()))
    }

    /// Resolves the environment that contains the variable with the given name.
    fn resolve(
        env: &Arc<Mutex<Self>>,
        name: &str,
        span: SourceSpan,
    ) -> Result<Arc<Mutex<Env>>, EnvError> {
        if env.lock().unwrap().values.contains_key(name) {
            return Ok(Arc::clone(env));
        }

        match &env.lock().unwrap().parent {
            Some(parent) => Self::resolve(parent, name, span),
            None => Err(EnvError::IdentifierNotFound { span }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for Val {
        fn eq(&self, other: &Self) -> bool {
            match (&self.kind, &other.kind) {
                (ValKind::Int(a), ValKind::Int(b)) => a == b,
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn declare_and_lookup() {
        let env = Env::new();

        let name = "foo";
        let value: Val = ValKind::Int(0).into();
        let span = (0, 3).into();

        // Declare a variable in the environment
        env.lock()
            .unwrap()
            .declare(name.to_string(), value.clone(), span)
            .expect("should be able to declare variable");

        // Lookup the variable
        let result = Env::lookup(&env, &name, span).expect("variable should exist");
        assert_eq!(result, value);
    }

    #[test]
    fn declare_error() {
        let env = Env::new();
        let mut env = env.lock().unwrap();

        let name = "foo";
        let value: Val = ValKind::Int(0).into();
        let span = (0, 3).into();

        // Declare a variable in the environment
        env.declare(name.to_string(), value.clone(), span)
            .expect("should be able to declare variable");

        // Attempt to redeclare the same variable
        let result = env
            .declare(name.to_string(), value.clone(), span)
            .expect_err("result should be an error");

        assert!(matches!(
            result.downcast_ref::<EnvError>(),
            Some(&EnvError::IdentifierAlreadyExists { span: _span })
        ));
    }

    #[test]
    fn lookup_error() {
        let env = Env::new();

        // Attempt to lookup a non-existent variable
        let name = "foo";
        let span = (0, 3).into();

        let result = Env::lookup(&env, &name, span).expect_err("result should be an error");

        assert!(matches!(
            result.downcast_ref::<EnvError>(),
            Some(EnvError::IdentifierNotFound { span: _span })
        ));
    }

    #[test]
    fn assign_and_lookup() {
        let env = Env::new();

        let name = "foo";
        let value: Val = ValKind::Int(0).into();
        let span = (0, 3).into();

        // Declare a variable in the environment
        env.lock()
            .unwrap()
            .declare(name.to_string(), value.clone(), span)
            .expect("should be able to declare variable");

        // Assign a new value to the variable
        let value: Val = ValKind::Int(1).into();
        Env::assign(&env, name.to_string(), value.clone(), span)
            .expect("should be able to assign value to variable");

        // Lookup the variable
        let result = Env::lookup(&env, &name, span).expect("should be able to lookup variable");
        assert_eq!(result, value);
    }

    #[test]
    fn nested_environments() {
        let parent_env = Env::new();

        let name = "foo";
        let value: Val = ValKind::Int(0).into();
        let span = (0, 3).into();

        // Declare a variable in the parent environment
        parent_env
            .lock()
            .unwrap()
            .declare(name.to_string(), value.clone(), span)
            .expect("should be able to declare variable");

        // Create a child environment with the parent environment
        let child_env = Env::with_parent(Arc::clone(&parent_env));

        // Lookup the variable from the child environment
        let result = Env::lookup(&child_env, &name, span);
        assert_eq!(result.unwrap(), value.clone());

        // Declare a new variable in the parent environment
        let name = "bar";
        let value: Val = ValKind::Int(0).into();
        parent_env
            .lock()
            .unwrap()
            .declare(name.to_string(), value.clone(), span)
            .expect("should be able to declare variable");

        // Lookup the new variable from the child environment
        let result =
            Env::lookup(&child_env, &name, span).expect("should be able to lookup variable");
        assert_eq!(result, value);
    }
}
