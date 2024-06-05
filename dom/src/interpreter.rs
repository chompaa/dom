use thiserror::Error;

use crate::{
    ast::{Cond, Expr, Func, Ident, Loop, Stmt, Var},
    environment::{Env, Val},
    lexer::{BinaryOp, CmpOp},
    util::Result,
};

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("missing identifier in assignment")]
    Assignment,
    #[error("binary expression unsupported")]
    Binary,
    #[error("comparison expressions can only contain integers or references to integers")]
    Cmp,
    #[error("could not interpret arguments")]
    Args,
    #[error("caller is not a defined function")]
    Caller,
}

#[derive(Error, Debug)]
pub enum Exception {
    #[error("cannot break out of non-loop")]
    Break,
    #[error("cannot continue out of non-loop")]
    Continue,
    #[error("cannot return out of non-func")]
    Return(Option<Box<Expr>>),
}

pub fn eval(statement: impl Into<Stmt>, env: &mut Env) -> Result<Val> {
    match statement.into() {
        Stmt::Program { body } => eval_body(body, env),
        Stmt::Cond(cond) => eval_cond(cond, env),
        Stmt::Func(func) => eval_func(func, env),
        Stmt::Loop(_loop) => eval_loop(_loop, env),
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
            Expr::Return { value } => Err(Box::new(Exception::Return(value))),
            Expr::Continue => Err(Box::new(Exception::Continue)),
            Expr::Break => Err(Box::new(Exception::Break)),
        },
    }
}

fn eval_body(body: Vec<Stmt>, env: &mut Env) -> Result<Val> {
    body.into_iter()
        .map(|stmt| eval(stmt, env))
        .last()
        .unwrap_or(Ok(Val::None))
}

fn eval_cond(cond: Cond, env: &mut Env) -> Result<Val> {
    let Cond { condition, body } = cond;

    let Val::Bool(success) = eval(condition, env)? else {
        return Err(Box::new(InterpreterError::Cmp));
    };

    if success {
        let result = eval_body(body, env)?;

        return Ok(result);
    }

    Ok(Val::None)
}

fn eval_func(func: Func, env: &mut Env) -> Result<Val> {
    let ident = &func.ident;

    let func = Val::Func {
        ident: ident.to_owned(),
        params: func.params,
        body: func.body,
        env: Env::with_parent(env.clone()),
    };

    let result = env.declare(ident.to_owned(), func);

    Ok(result?)
}

fn eval_loop(_loop: Loop, env: &mut Env) -> Result<Val> {
    let Loop { body } = _loop;

    let mut last = None;

    // Used so that we can keep track of what variables existed before the loop
    let stored_env = env.clone();
    let idents = stored_env.values().keys().collect::<Vec<_>>();

    'outer: loop {
        for stmt in &body {
            let result = eval(stmt.clone(), env);

            match result {
                Ok(result) => last = Some(result),
                Err(kind) => match kind.downcast_ref() {
                    Some(Exception::Continue) => continue 'outer,
                    Some(Exception::Break) => break 'outer,
                    _ => return Err(kind),
                },
            }
        }

        // Drop any values that were declared in an iteration
        env.values_mut().retain(|ident, _| idents.contains(&ident))
    }

    match last {
        Some(val) => Ok(val),
        None => Ok(Val::None),
    }
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

            let mut last = None;

            for stmt in body {
                let result = eval(stmt, &mut env);

                match result {
                    Ok(result) => last = Some(result),
                    Err(kind) => match kind.downcast_ref() {
                        Some(Exception::Return(value)) => {
                            last = match value {
                                Some(value) => Some(eval(*value.clone(), &mut env)?),
                                None => None,
                            };
                            break;
                        }
                        _ => return Err(kind),
                    },
                }
            }

            match last {
                Some(val) => Ok(val),
                None => Ok(Val::None),
            }
        }
        _ => Err(Box::new(InterpreterError::Caller)),
    }
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

    let result: Val = match (lhs, rhs) {
        // Integer operations
        (Val::Int(lhs), Val::Int(rhs)) => {
            let value = match op {
                BinaryOp::Add => lhs + rhs,
                BinaryOp::Sub => lhs - rhs,
                BinaryOp::Mul => lhs * rhs,
                BinaryOp::Div => lhs / rhs,
            };
            Val::Int(value)
        }
        // String addition.
        //
        // Example: "foo" + "bar" -> "foobar"
        (Val::Str(lhs), Val::Str(rhs)) => {
            if op == BinaryOp::Add {
                Val::Str(format!("{lhs}{rhs}"))
            } else {
                return Err(Box::new(InterpreterError::Binary));
            }
        }
        // String repeating. Integers less than one are not valid.
        //
        // Example: "foo" * 2 -> "foofoo".
        (Val::Str(lhs), Val::Int(rhs)) => {
            if op == BinaryOp::Mul && rhs > 1 {
                Val::Str(lhs.repeat(rhs as usize))
            } else {
                return Err(Box::new(InterpreterError::Binary));
            }
        }
        (Val::Int(lhs), Val::Str(rhs)) => {
            if op == BinaryOp::Mul && lhs > 1 {
                Val::Str(rhs.repeat(lhs as usize))
            } else {
                return Err(Box::new(InterpreterError::Binary));
            }
        }
        _ => return Err(Box::new(InterpreterError::Binary)),
    };

    Ok(result)
}

fn eval_ident(ident: Ident, env: &mut Env) -> Result<Val> {
    let val = env.lookup(ident)?;
    Ok(val)
}
