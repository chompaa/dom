use thiserror::Error;

use crate::{
    ast::{Expr, Func, Ident, Stmt, Var},
    environment::{Env, Val},
    lexer::{BinaryOp, CmpOp},
    util::Result,
};

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("missing identifier in assignment")]
    Assignment,
    #[error("binary expressions can only contain integers or references to integers")]
    Binary,
    #[error("comparison expressions can only contain integers or references to integers")]
    Cmp,
    #[error("could not interpret arguments")]
    Args,
    #[error("caller is not a defined function")]
    Caller,
}

pub fn eval(statement: impl Into<Stmt>, env: &mut Env) -> Result<Val> {
    match statement.into() {
        Stmt::Program { body } => eval_program(body, env),
        Stmt::Func(func) => eval_func(func, env),
        Stmt::Var(var) => eval_var(var, env),
        Stmt::Expr(expr) => match expr {
            Expr::Assignment { assignee, value } => eval_assign(*assignee, *value, env),
            Expr::Call { caller, args } => eval_call(*caller, args, env),
            Expr::CmpOp { left, right, op } => eval_cmp_expr(*left, *right, op, env),
            Expr::BinaryOp { left, right, op } => eval_binary_expr(*left, *right, op, env),
            Expr::Ident(ident) => eval_ident(ident, env),
            Expr::Bool(value) => Ok(Val::Bool(value)),
            Expr::Int(number) => Ok(Val::Int(number)),
            Expr::Str(value) => Ok(Val::Str(value)),
        },
    }
}

fn eval_program(body: Vec<Stmt>, env: &mut Env) -> Result<Val> {
    body.into_iter()
        .map(|stmt| eval(stmt, env))
        .last()
        .expect("Last value from program body should be obtainable")
}

fn eval_cmp_expr(left: Expr, right: Expr, op: CmpOp, env: &mut Env) -> Result<Val> {
    let lhs = eval(left, env)?;
    let rhs = eval(right, env)?;

    let (lhs, rhs) = match (lhs, rhs) {
        (Val::Int(lhs), Val::Int(rhs)) => (lhs, rhs),
        _ => return Err(Box::new(InterpreterError::Cmp)),
    };

    let result = match op {
        CmpOp::Eq => lhs == rhs,
        CmpOp::NotEq => lhs != rhs,
        CmpOp::Greater => lhs > rhs,
        CmpOp::GreaterEq => lhs >= rhs,
        CmpOp::Less => lhs < rhs,
        CmpOp::LessEq => lhs <= rhs,
    };

    Ok(Val::Bool(result))
}

fn eval_binary_expr(left: Expr, right: Expr, op: BinaryOp, env: &mut Env) -> Result<Val> {
    let lhs = eval(left, env)?;
    let rhs = eval(right, env)?;

    let (lhs, rhs) = match (lhs, rhs) {
        (Val::Int(lhs), Val::Int(rhs)) => (lhs, rhs),
        _ => return Err(Box::new(InterpreterError::Binary)),
    };

    let result = match op {
        BinaryOp::Add => lhs + rhs,
        BinaryOp::Sub => lhs - rhs,
        BinaryOp::Mul => lhs * rhs,
        BinaryOp::Div => lhs / rhs,
    };

    Ok(Val::Int(result))
}

fn eval_ident(ident: Ident, env: &mut Env) -> Result<Val> {
    let val = env.lookup(ident)?;
    Ok(val)
}

fn eval_func(func: Func, env: &mut Env) -> Result<Val> {
    let ident = &func.ident;

    let func = Val::Func {
        ident: ident.to_owned(),
        params: func.params,
        body: func.body,
        // TODO: Remove this clone
        env: Env::with_parent(env.clone()),
    };

    let result = env.declare(ident.to_owned(), func);

    Ok(result?)
}

fn eval_var(var: Var, env: &mut Env) -> Result<Val> {
    let value = eval(*var.value, env)?;
    let result = env.declare(var.ident, value)?;
    Ok(result)
}

fn eval_assign(assignee: Expr, value: Expr, env: &mut Env) -> Result<Val> {
    let Expr::Ident(assignee) = assignee else {
        return Err(Box::new(InterpreterError::Assignment));
    };

    let value = eval(value, env)?;
    let result = env.assign(assignee, value)?;
    Ok(result)
}

fn eval_call(caller: Expr, args: Vec<Expr>, env: &mut Env) -> Result<Val> {
    let Ok(args): Result<Vec<Val>> = args.into_iter().map(|arg| eval(arg, env)).collect() else {
        return Err(Box::new(InterpreterError::Args));
    };

    match eval(caller, env)? {
        Val::NativeFunc(mut native_func) => match native_func(args, env) {
            Some(result) => Ok(result),
            None => Ok(Val::None),
        },
        Val::Func {
            params, body, env, ..
        } => {
            let mut env = env;

            for (param, arg) in params.into_iter().zip(args.into_iter()) {
                env.declare(param, arg)?;
            }

            body.into_iter()
                .map(|stmt| eval(stmt, &mut env))
                .last()
                .expect("Last value from program body should be obtainable")
        }
        _ => Err(Box::new(InterpreterError::Caller)),
    }
}