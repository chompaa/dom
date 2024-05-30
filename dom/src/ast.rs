use crate::lexer::{BinaryOp, CmpOp};

/// An identifier (e.g. a variable name).
pub type Ident = String;

/// The kind of a statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// A program consisting of a sequence of statements.
    Program { body: Vec<Stmt> },
    /// A callable function.
    Func(Func),
    /// A return statement.
    Return(Return),
    /// A variable declaration.
    Var(Var),
    /// An expression statement.
    Expr(Expr),
}

impl From<Expr> for Stmt {
    /// Converts an expression into a statement.
    fn from(expr: Expr) -> Self {
        Stmt::Expr(expr)
    }
}

/// A function declaration.
#[derive(Debug, Clone)]
pub struct Func {
    /// The identifier of the function.
    pub ident: Ident,
    /// The parameters of the function.
    pub params: Vec<Ident>,
    /// The body of the function.
    pub body: Vec<Stmt>,
}

/// A return statement.
#[derive(Debug, Clone)]
pub struct Return {
    /// The value returned.
    pub value: Option<Expr>,
}

/// A variable declaration.
#[derive(Debug, Clone)]
pub struct Var {
    /// The identifier of the variable.
    pub ident: Ident,
    /// The value of the variable.
    pub value: Box<Stmt>,
}

/// An expression in the abstract syntax tree.
#[derive(Debug, Clone)]
pub enum Expr {
    /// An assignment expression.
    Assignment {
        /// The assignee (left-hand side) of the assignment.
        assignee: Box<Expr>,
        /// The value (right-hand side) of the assignment.
        value: Box<Expr>,
    },
    Call {
        caller: Box<Expr>,
        args: Vec<Expr>,
    },
    /// A string expression.
    Str(String),
    /// An identifier expression.
    Ident(Ident),
    /// An boolean literal expression.
    Bool(bool),
    /// An integer literal expression.
    Int(i32),
    /// A comparison operation expression.
    CmpOp {
        /// The left operand of the comparison operation.
        left: Box<Expr>,
        /// The right operand of the comparison operation.
        right: Box<Expr>,
        /// The comparison operation itself.
        op: CmpOp,
    },
    /// A binary operation expression.
    BinaryOp {
        /// The left operand of the binary operation.
        left: Box<Expr>,
        /// The right operand of the binary operation.
        right: Box<Expr>,
        /// The binary operation itself.
        op: BinaryOp,
    },
}
