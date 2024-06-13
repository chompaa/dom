use miette::SourceSpan;

use crate::lexer::CmpOp;

/// An identifier (e.g. a variable name).
pub type Ident = String;

/// The kind of a statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// A program consisting of a sequence of statements.
    Program { body: Vec<Stmt> },
    /// A conditional statement.
    Cond(Cond),
    /// A callable function.
    Func(Func),
    /// A loop statement.
    Loop(Loop),
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

/// A conditional declaration.
#[derive(Debug, Clone)]
pub struct Cond {
    /// The condition to be checked.
    pub condition: Expr,
    /// The body of the conditional to be executed if the condition succeeds.
    pub body: Vec<Stmt>,
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

/// A loop statement.
#[derive(Debug, Clone)]
pub struct Loop {
    /// The value returned.
    pub body: Vec<Stmt>,
}

/// A variable declaration.
#[derive(Debug, Clone)]
pub struct Var {
    /// The identifier of the variable.
    pub ident: Ident,
    /// The value of the variable.
    pub value: Box<Stmt>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

/// An expression in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
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
    /// A unary operation expression.
    UnaryOp {
        /// The expression of the unary operation.
        expr: Box<Expr>,
        /// The unary operation itself.
        op: UnaryOp,
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
    /// A return expression for functions.
    Return {
        /// The value returned.
        value: Option<Box<Expr>>,
    },
    /// A continue expression for loops.
    Continue,
    /// A break expression for loops.
    Break,
}
