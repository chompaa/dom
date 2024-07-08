use std::fmt;

use miette::SourceSpan;

use crate::lexer::RelOp;

/// An identifier (e.g. a variable name).
pub type Ident = String;

/// The kind of a statement.
#[derive(Debug, Clone, PartialEq)]
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
    /// A use statement for modules.
    Use(Use),
}

impl From<Expr> for Stmt {
    /// Converts an expression into a statement.
    fn from(expr: Expr) -> Self {
        Stmt::Expr(expr)
    }
}

/// A conditional declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Cond {
    /// The condition to be checked.
    pub condition: Expr,
    /// The body of the conditional to be executed if the condition succeeds.
    pub body: Vec<Stmt>,
    /// The span of the condition.
    pub span: SourceSpan,
}

/// A function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    /// The identifier of the function.
    pub ident: Ident,
    /// The parameters of the function.
    pub params: Vec<Ident>,
    /// The body of the function.
    pub body: Vec<Stmt>,
    /// The span of the function identifier.
    pub span: SourceSpan,
}

/// A loop statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    /// The value returned.
    pub body: Vec<Stmt>,
    /// The span of the loop keyword.
    pub span: SourceSpan,
}

/// A variable declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    /// The identifier of the variable.
    pub ident: Ident,
    /// The value of the variable.
    pub value: Box<Stmt>,
    /// The span of the variable identifier.
    pub span: SourceSpan,
}

/// Logical operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicOp {
    And,
    Or,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

/// An expression in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

/// The kind of expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// An assignment expression.
    Assignment {
        /// The assignee (left-hand side) of the assignment.
        assignee: Box<Expr>,
        /// The value (right-hand side) of the assignment.
        value: Box<Expr>,
    },
    Pipe {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        caller: Box<Expr>,
        args: Vec<Expr>,
    },
    List {
        items: Vec<Expr>,
    },
    /// A string expression.
    Str(String),
    /// An identifier expression.
    Ident(Ident),
    /// An boolean literal expression.
    Bool(bool),
    /// An integer literal expression.
    Int(i32),
    /// A relational operation expression.
    RelOp {
        /// The left operand of the comparison operation.
        left: Box<Expr>,
        /// The right operand of the comparison operation.
        right: Box<Expr>,
        /// The comparison operation itself.
        op: RelOp,
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
    /// A logical operation expression.
    LogicOp {
        /// The left operand of the logic operation.
        left: Box<Expr>,
        /// The right operand of the logic operation.
        right: Box<Expr>,
        /// The logic operation itself.
        op: LogicOp,
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
    /// A module access expression.
    Mod {
        /// The module being accessed.
        module: Box<Expr>,
        /// The item in the module.
        item: Box<Expr>,
    },
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assignment { .. } => write!(f, "Assignment"),
            Self::Pipe { .. } => write!(f, "Pipe"),
            Self::Call { .. } => write!(f, "Call"),
            Self::List { .. } => write!(f, "List"),
            Self::Str { .. } => write!(f, "Str"),
            Self::Ident { .. } => write!(f, "Ident"),
            Self::Bool { .. } => write!(f, "Bool"),
            Self::Int { .. } => write!(f, "Int"),
            Self::LogicOp { .. } => write!(f, "LogicOp"),
            Self::RelOp { .. } => write!(f, "RelOp"),
            Self::UnaryOp { .. } => write!(f, "UnaryOp"),
            Self::BinaryOp { .. } => write!(f, "BinaryOp"),
            Self::Return { .. } => write!(f, "Return"),
            Self::Continue => write!(f, "Continue"),
            Self::Break => write!(f, "Break"),
            Self::Mod { .. } => write!(f, "Mod"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: String,
    pub span: SourceSpan,
}
