use crate::{
    ast::{Expr, Ident, Stmt, StmtKind, Var},
    environment::Env,
    lexer::BinaryOp,
};

pub type Val = i32;

pub fn eval(statement: Stmt, env: &mut Env) -> Result<Val, ()> {
    match statement.kind {
        StmtKind::Program { body } => eval_program(body, env),
        StmtKind::Var(var) => eval_var(var, env),
        StmtKind::Expr(expr) => match expr {
            Expr::Assignment { assignee, value } => eval_assign(*assignee, *value, env),
            Expr::Int(number) => Ok(number),
            Expr::BinaryOp { left, right, op } => eval_binary_expr(*left, *right, op, env),
            Expr::Ident(ident) => eval_ident(ident, env),
        },
    }
}

fn eval_program(body: Vec<Stmt>, env: &mut Env) -> Result<Val, ()> {
    body.into_iter()
        .map(|stmt| eval(stmt, env))
        .last()
        .expect("Last value from program body should be obtainable")
}

fn eval_numeric_binary_expr(lhs: Val, rhs: Val, op: BinaryOp) -> Result<Val, ()> {
    let res = match op {
        BinaryOp::Add => lhs + rhs,
        BinaryOp::Sub => lhs - rhs,
        BinaryOp::Mul => lhs * rhs,
        BinaryOp::Div => lhs / rhs,
    };

    Ok(res)
}

fn eval_binary_expr(left: Expr, right: Expr, op: BinaryOp, env: &mut Env) -> Result<Val, ()> {
    let lhs = eval(left.into(), env)?;
    let rhs = eval(right.into(), env)?;

    eval_numeric_binary_expr(lhs, rhs, op)
}

fn eval_ident(ident: Ident, env: &mut Env) -> Result<Val, ()> {
    env.lookup(ident)
}

fn eval_var(var: Var, env: &mut Env) -> Result<Val, ()> {
    let value = eval(*var.value, env)?;
    env.declare(var.ident, value)
}

fn eval_assign(assignee: Expr, value: Expr, env: &mut Env) -> Result<Val, ()> {
    let Expr::Ident(assignee) = assignee else {
        panic!("Expected LHS of assignment to be an identifier");
    };

    let value = eval(value.into(), env)?;
    env.assign(assignee, value)
}
