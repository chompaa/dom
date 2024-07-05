use std::sync::{Arc, Mutex};

use miette::{Diagnostic, Result, SourceSpan};
use thiserror::Error;

use crate::{
    ast::{BinaryOp, Cond, Expr, ExprKind, Func, Ident, LogicOp, Loop, Stmt, UnaryOp, Use, Var},
    environment::{Env, Val, ValKind},
    lexer::RelOp,
};

#[derive(Error, Diagnostic, Debug)]
pub enum InterpreterError {
    #[error("assignment does not contain valid identifier")]
    #[diagnostic(code(interpreter::invalid_assignment_identifier))]
    InvalidAssignmentIdentifier {
        #[label("this identifier is invalid")]
        span: SourceSpan,
    },
    #[error("unary operator `{op:?}` unsupported for type `{kind}`")]
    #[diagnostic(code(interpreter::unary_expression_unsupported))]
    UnaryExpressionUnsupported {
        #[label("this operation is unsupported")]
        span: SourceSpan,
        kind: ExprKind,
        op: UnaryOp,
    },
    #[error("binary operation `{op:?}` unsupported for types `{left}` and `{right}`")]
    #[diagnostic(code(interpreter::binary_expression_unsupported))]
    BinaryExpressionUnsupported {
        #[label("this operation is unsupported")]
        span: SourceSpan,
        left: ExprKind,
        right: ExprKind,
        op: BinaryOp,
    },
    #[error("logical operation `{op:?}` unsupported for types `{left}` and `{right}`")]
    #[diagnostic(code(interpreter::logical_expression_unsupported))]
    LogicalExpressionUnsupported {
        #[label("this operation is unsupported")]
        span: SourceSpan,
        left: ExprKind,
        right: ExprKind,
        op: LogicOp,
    },
    #[error("relational operation `{op:?}` unsupported for types `{left}` and `{right}`")]
    #[diagnostic(code(interpreter::relational_expression_unsupported))]
    RelationalExpressionUnsupported {
        #[label("this operation is unsupported")]
        span: SourceSpan,
        left: ExprKind,
        right: ExprKind,
        op: RelOp,
    },
    #[error("caller is not a defined function")]
    #[diagnostic(code(interpreter::caller_not_defined))]
    CallerNotDefined {
        #[label("this is not a function")]
        span: SourceSpan,
    },
    #[error("right-hand-side of pipe expression is not a function call")]
    #[diagnostic(code(interpreter::invalid_pipe_caller))]
    InvalidPipeCaller {
        #[label("this is not a function call")]
        span: SourceSpan,
    },
    #[error("caller arguments do not match function arguments")]
    #[diagnostic(code(interpreter::mismatched_args))]
    MismatchedArgs {
        #[label("this call has incorrect argument count")]
        span: SourceSpan,
    },
    #[error("module not found")]
    #[diagnostic(code(interpreter::mismatched_args))]
    ModuleNotFound {
        #[label("this module could not be found")]
        span: SourceSpan,
    },
    #[error("expression is not a valid module")]
    #[diagnostic(code(interpreter::mismatched_args))]
    InvalidModule {
        #[label("this expression is not a module")]
        span: SourceSpan,
    },
}

#[derive(Error, Diagnostic, Debug)]
pub enum Exception {
    #[error("cannot break out of non-loop")]
    Break,
    #[error("cannot continue out of non-loop")]
    Continue,
    #[error("cannot return out of non-func")]
    Return(Option<Box<Expr>>),
}

pub trait UseHook {
    fn eval_use(
        &self,
        interpreter: &Interpreter,
        path: String,
        env: &Arc<Mutex<Env>>,
        span: SourceSpan,
    ) -> Result<()>;
}

pub trait ModuleHook {
    fn use_module(&self, path: String, env: &Arc<Mutex<Env>>) -> Result<Option<()>>;
}

pub struct Interpreter {
    use_hook: Box<dyn UseHook>,
    module_hook: Box<dyn ModuleHook>,
}

impl Interpreter {
    pub fn new<U: UseHook + Default + 'static, M: ModuleHook + Default + 'static>() -> Self {
        Self {
            use_hook: Box::new(U::default()),
            module_hook: Box::new(M::default()),
        }
    }

    pub fn eval(&self, statement: impl Into<Stmt>, env: &Arc<Mutex<Env>>) -> Result<Val> {
        match statement.into() {
            Stmt::Program { body } => self.eval_body(body, env),
            Stmt::Cond(Cond {
                condition, body, ..
            }) => self.eval_cond(condition, body, env),
            Stmt::Func(Func {
                ident,
                params,
                body,
                span,
                ..
            }) => self.eval_func(&ident, params, body, env, span),
            Stmt::Loop(Loop { body, .. }) => self.eval_loop(&body, env),
            Stmt::Var(Var { ident, value, span }) => self.eval_var(&ident, *value, env, span),
            Stmt::Expr(expr) => {
                let Expr { kind, span } = expr;
                match kind {
                    ExprKind::Assignment { assignee, value } => {
                        self.eval_assign(*assignee, *value, env)
                    }
                    ExprKind::Pipe { left, right } => self.eval_pipe_expr(*left, *right, env),
                    ExprKind::Call { caller, args } => self.eval_call(*caller, args, env, span),
                    ExprKind::List { items } => self.eval_list_expr(items, env),
                    ExprKind::LogicOp { left, right, op } => {
                        self.eval_logic_expr(*left, *right, op, span, env)
                    }
                    ExprKind::RelOp { left, right, op } => {
                        self.eval_rel_expr(*left, *right, op, span, env)
                    }
                    ExprKind::UnaryOp { expr, op } => self.eval_unary_expr(*expr, op, span, env),
                    ExprKind::BinaryOp { left, right, op } => {
                        self.eval_binary_expr(*left, *right, op, span, env)
                    }
                    ExprKind::Ident(ident) => self.eval_ident(&ident, env, span),
                    ExprKind::Bool(value) => Ok(ValKind::Bool(value).into()),
                    ExprKind::Int(number) => Ok(ValKind::Int(number).into()),
                    ExprKind::Str(value) => Ok(ValKind::Str(value).into()),
                    ExprKind::Return { value } => Err(Exception::Return(value).into()),
                    ExprKind::Continue => Err(Exception::Continue.into()),
                    ExprKind::Break => Err(Exception::Break.into()),
                    ExprKind::Mod { module, item } => self.eval_mod_expr(*module, *item, env),
                }
            }
            Stmt::Use(Use { path, span }) => self.eval_use(&path, env, span),
        }
    }

    fn eval_body(&self, body: Vec<Stmt>, env: &Arc<Mutex<Env>>) -> Result<Val> {
        let mut last = Val::NONE;
        for stmt in body {
            last = self.eval(stmt, env)?;
        }
        Ok(last)
    }

    fn eval_cond(&self, condition: Expr, body: Vec<Stmt>, env: &Arc<Mutex<Env>>) -> Result<Val> {
        let ValKind::Bool(success) = self.eval(condition, env)?.kind else {
            unreachable!("`Val::Bool` should be returned from condition evaluation");
        };

        if success {
            let env = Env::with_parent(env);
            let result = self.eval_body(body, &env)?;
            return Ok(result);
        }

        Ok(Val::NONE)
    }

    fn eval_func(
        &self,
        ident: &Ident,
        params: Vec<Ident>,
        body: Vec<Stmt>,
        env: &Arc<Mutex<Env>>,
        span: SourceSpan,
    ) -> Result<Val> {
        let func = ValKind::Func {
            ident: ident.to_owned(),
            params,
            body,
            env: Env::with_parent(env),
        };

        env.lock().unwrap().declare(ident, func.into(), span)
    }

    fn eval_loop(&self, body: &Vec<Stmt>, env: &Arc<Mutex<Env>>) -> Result<Val> {
        let mut last = None;

        'outer: loop {
            let loop_env = Env::with_parent(env);

            for stmt in body {
                let result = self.eval(stmt.clone(), &loop_env);

                match result {
                    Ok(result) => last = Some(result),
                    Err(kind) => match kind.downcast_ref() {
                        Some(Exception::Continue) => continue 'outer,
                        Some(Exception::Break) => break 'outer,
                        _ => return Err(kind),
                    },
                }
            }
        }

        match last {
            Some(val) => Ok(val),
            None => Ok(Val::NONE),
        }
    }

    fn eval_var(
        &self,
        ident: &str,
        value: Stmt,
        env: &Arc<Mutex<Env>>,
        span: SourceSpan,
    ) -> Result<Val> {
        let value = self.eval(value, env)?;
        let result = env.lock().unwrap().declare(ident, value, span)?;
        Ok(result)
    }

    fn eval_assign(&self, assignee: Expr, value: Expr, env: &Arc<Mutex<Env>>) -> Result<Val> {
        let span = assignee.span;

        let ExprKind::Ident(assignee) = assignee.kind else {
            return Err(InterpreterError::InvalidAssignmentIdentifier { span }.into());
        };

        let value = self.eval(value, env)?;
        let result = Env::assign(env, &assignee, value, span)?;
        Ok(result)
    }

    fn eval_pipe_expr(&self, left: Expr, right: Expr, env: &Arc<Mutex<Env>>) -> Result<Val> {
        match right.kind {
            ExprKind::Call { caller, mut args } => {
                args.insert(0, left);
                self.eval_call(*caller, args, env, right.span)
            }
            _ => Err(InterpreterError::InvalidPipeCaller { span: right.span }.into()),
        }
    }

    fn eval_call(
        &self,
        caller: Expr,
        args: Vec<Expr>,
        env: &Arc<Mutex<Env>>,
        span: SourceSpan,
    ) -> Result<Val> {
        // Since we'll be (potentially) switching environments if the caller is a module,
        // we should parse all of the arguments in the current environment first
        let args = args
            .into_iter()
            .map(|arg| self.eval(arg, env))
            .collect::<Result<Vec<Val>>>()?;

        // We introduce another parameter here, so we don't lose the original environment
        self._eval_call(caller, args, env, env, span)
    }

    fn _eval_call(
        &self,
        caller: Expr,
        args: Vec<Val>,
        env: &Arc<Mutex<Env>>,
        mod_env: &Arc<Mutex<Env>>,
        span: SourceSpan,
    ) -> Result<Val> {
        match caller.kind {
            ExprKind::Mod { module, item } => {
                // If the caller is a member of a module, find the module
                let mod_env = self.lookup_module(*module, env)?;
                // Now call in the module's environment instead
                return self._eval_call(*item, args, env, &mod_env, span);
            }
            ExprKind::Ident(ref ident) => {
                // Check if the caller is a built-in function
                if let Some(builtin) = Env::lookup_builtin(mod_env, ident) {
                    let result = builtin.run(&args, env);
                    return Ok(result.unwrap_or(Val::NONE));
                }
            }
            _ => (),
        }

        let caller_span = caller.span;
        // Caller must be a user-defined function at this point
        let ValKind::Func {
            params, body, env, ..
        } = self.eval(caller, env)?.kind
        else {
            return Err(InterpreterError::CallerNotDefined { span: caller_span }.into());
        };

        if args.len() != params.len() {
            return Err(InterpreterError::MismatchedArgs { span }.into());
        }

        for (param, arg) in params.into_iter().zip(args.into_iter()) {
            env.lock().unwrap().declare(&param, arg, span)?;
        }

        let mut last = None;

        for stmt in body {
            let result = self.eval(stmt, &env);

            match result {
                Ok(result) => last = Some(result),
                Err(kind) => match kind.downcast_ref() {
                    Some(Exception::Return(value)) => {
                        last = match value {
                            Some(value) => Some(self.eval(*value.clone(), &env)?),
                            None => None,
                        };
                        break;
                    }
                    _ => return Err(kind),
                },
            }
        }

        Ok(last.unwrap_or(Val::NONE))
    }

    fn eval_list_expr(&self, items: Vec<Expr>, env: &Arc<Mutex<Env>>) -> Result<Val> {
        let items = items
            .into_iter()
            .map(|item| self.eval(item, env))
            .collect::<Result<Vec<Val>>>()?;

        Ok(ValKind::List(items).into())
    }

    fn eval_logic_expr(
        &self,
        left: Expr,
        right: Expr,
        op: LogicOp,
        span: SourceSpan,
        env: &Arc<Mutex<Env>>,
    ) -> Result<Val> {
        let lhs = self.eval(left.clone(), env)?.kind;
        let rhs = self.eval(right.clone(), env)?.kind;

        let (ValKind::Bool(lhs), ValKind::Bool(rhs)) = (lhs, rhs) else {
            return Err(InterpreterError::LogicalExpressionUnsupported {
                span,
                left: left.kind,
                right: right.kind,
                op,
            }
            .into());
        };

        let result = match op {
            LogicOp::And => lhs && rhs,
            LogicOp::Or => lhs || rhs,
        };

        Ok(ValKind::Bool(result).into())
    }

    fn eval_rel_expr(
        &self,
        left: Expr,
        right: Expr,
        op: RelOp,
        span: SourceSpan,
        env: &Arc<Mutex<Env>>,
    ) -> Result<Val> {
        let lhs = self.eval(left.clone(), env)?.kind;
        let rhs = self.eval(right.clone(), env)?.kind;

        let err = InterpreterError::RelationalExpressionUnsupported {
            span,
            left: left.kind,
            right: right.kind,
            op,
        };

        let result = match (&lhs, &rhs) {
            (ValKind::Bool(lhs), ValKind::Bool(rhs)) => match op {
                RelOp::Eq => lhs == rhs,
                RelOp::NotEq => lhs != rhs,
                _ => return Err(err.into()),
            },
            (ValKind::Int(lhs), ValKind::Int(rhs)) => match op {
                RelOp::Eq => lhs == rhs,
                RelOp::NotEq => lhs != rhs,
                RelOp::Greater => lhs > rhs,
                RelOp::GreaterEq => lhs >= rhs,
                RelOp::Less => lhs < rhs,
                RelOp::LessEq => lhs <= rhs,
            },
            (ValKind::Str(lhs), ValKind::Str(rhs)) => match op {
                RelOp::Eq => lhs == rhs,
                RelOp::NotEq => lhs != rhs,
                _ => return Err(err.into()),
            },
            _ => return Err(err.into()),
        };

        Ok(ValKind::Bool(result).into())
    }

    fn eval_unary_expr(
        &self,
        expr: Expr,
        op: UnaryOp,
        span: SourceSpan,
        env: &Arc<Mutex<Env>>,
    ) -> Result<Val> {
        let result = self.eval(expr.clone(), env)?;

        let err = InterpreterError::UnaryExpressionUnsupported {
            span,
            kind: expr.kind,
            op,
        };

        match result.kind {
            ValKind::Int(value) => match op {
                UnaryOp::Pos => Ok(result),
                UnaryOp::Neg => Ok(ValKind::Int(-value).into()),
                _ => Err(err.into()),
            },
            ValKind::Bool(value) => match op {
                UnaryOp::Not => Ok(ValKind::Bool(!value).into()),
                _ => Err(err.into()),
            },
            _ => Err(err.into()),
        }
    }

    fn eval_binary_expr(
        &self,
        left: Expr,
        right: Expr,
        op: BinaryOp,
        span: SourceSpan,
        env: &Arc<Mutex<Env>>,
    ) -> Result<Val> {
        let lhs = self.eval(left.clone(), env)?.kind;
        let rhs = self.eval(right.clone(), env)?.kind;

        let err = InterpreterError::BinaryExpressionUnsupported {
            span,
            left: left.kind,
            right: right.kind,
            op,
        };

        let result: ValKind = match (lhs, rhs) {
            // Integer operations
            (ValKind::Int(lhs), ValKind::Int(rhs)) => {
                let value = match op {
                    BinaryOp::Add => lhs + rhs,
                    BinaryOp::Sub => lhs - rhs,
                    BinaryOp::Mul => lhs * rhs,
                    BinaryOp::Div => lhs / rhs,
                };
                ValKind::Int(value)
            }
            // String addition.
            //
            // Example: "foo" + "bar" -> "foobar"
            (ValKind::Str(lhs), ValKind::Str(rhs)) => {
                if op == BinaryOp::Add {
                    ValKind::Str(format!("{lhs}{rhs}"))
                } else {
                    return Err(err.into());
                }
            }
            // String repeating. Integers less than one are not valid.
            //
            // Example: "foo" * 2 -> "foofoo".
            (ValKind::Str(lhs), ValKind::Int(rhs)) => {
                if op == BinaryOp::Mul && rhs >= 0 {
                    // Since `rhs` is positive, no need to worry about casting
                    ValKind::Str(lhs.repeat(rhs as usize))
                } else {
                    return Err(err.into());
                }
            }
            (ValKind::Int(lhs), ValKind::Str(rhs)) => {
                if op == BinaryOp::Mul && lhs >= 0 {
                    // Since `lhs` is positive, no need to worry about casting
                    ValKind::Str(rhs.repeat(lhs as usize))
                } else {
                    return Err(err.into());
                }
            }
            _ => return Err(err.into()),
        };

        Ok(result.into())
    }

    fn eval_ident(&self, ident: &Ident, env: &Arc<Mutex<Env>>, span: SourceSpan) -> Result<Val> {
        let val = Env::lookup(env, ident, span)?;
        Ok(val)
    }

    fn eval_mod_expr(&self, module: Expr, item: Expr, env: &Arc<Mutex<Env>>) -> Result<Val> {
        let mod_env = self.lookup_module(module, env)?;
        self.eval(item, &mod_env)
    }

    fn eval_use(&self, path: &str, env: &Arc<Mutex<Env>>, span: SourceSpan) -> Result<Val> {
        if self
            .module_hook
            .use_module(path.to_string(), env)?
            .is_some()
        {
            return Ok(Val::NONE);
        }

        self.use_hook.eval_use(self, path.to_string(), env, span)?;

        Ok(Val::NONE)
    }

    fn lookup_module(&self, module: Expr, env: &Arc<Mutex<Env>>) -> Result<Arc<Mutex<Env>>> {
        let span = module.span;

        let Val {
            kind: ValKind::Mod(mod_env),
            ..
        } = self.eval(module, env)?
        else {
            return Err(InterpreterError::InvalidModule { span }.into());
        };

        Ok(mod_env)
    }
}
