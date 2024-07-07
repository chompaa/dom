//! Parser used to produce an AST from a token stream
//!
//! Order of precedence (low to high):
//! - Assignments
//! - Logical OR
//! - Logical AND
//! - Operators
//! - Binary Addition
//! - Binary Multiplication
//! - Unary Operators
//! - Function Call
//! - Lists
//! - Primary Expressions

use std::collections::VecDeque;

use miette::{Diagnostic, Result, SourceSpan};
use thiserror::Error;

use crate::ast::{
    BinaryOp, Cond, Expr, ExprKind, Func, Ident, LogicOp, Loop, Stmt, UnaryOp, Use, Var,
};
use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Error, Diagnostic, Debug)]
pub enum ParserError {
    #[error("expected left brace `{{` following conditional statement")]
    #[diagnostic(code(parser::cond_block_begin))]
    CondBlockBegin {
        #[label("this conditional is missing a `{{` to start its body")]
        span: SourceSpan,
    },
    #[error("expected right brace `}}` to end conditional block")]
    #[diagnostic(code(parser::cond_block_end))]
    CondBlockEnd {
        #[label("this conditional is missing a `}}` to end its body")]
        span: SourceSpan,
    },
    #[error("invalid identifier following `fn` keyword")]
    #[diagnostic(code(parser::fn_identifier))]
    FnIdentifier {
        #[label("invalid identifier here")]
        span: SourceSpan,
    },
    #[error("expected left parenthesis `(` following identifier to begin function arguments")]
    #[diagnostic(code(parser::fn_args_begin))]
    FnArgsBegin {
        #[label("expected `(` following this identifier")]
        span: SourceSpan,
    },
    #[error("expected only identifiers in function arguments")]
    #[diagnostic(code(parser::fn_args))]
    FnArgs {
        #[label("this function has non-identifier arguments")]
        span: SourceSpan,
    },
    #[error("expected right parenthesis `)` to end argument list")]
    #[diagnostic(code(parser::fn_args_end))]
    FnArgsEnd {
        #[label("this function's arguments are never ended")]
        span: SourceSpan,
    },
    #[error("expected left brace `{{` following function arguments")]
    #[diagnostic(code(parser::fn_block_begin))]
    FnBlockBegin {
        #[label("this function is missing a `{{` to start its body")]
        span: SourceSpan,
    },
    #[error("expected right brace `}}` to end function block")]
    #[diagnostic(code(parser::fn_block_end))]
    FnBlockEnd {
        #[label("this function is missing a `}}` to end its body")]
        span: SourceSpan,
    },
    #[error("expected right bracket `]` to end list")]
    #[diagnostic(code(parser::fn_block_end))]
    ListItemsEnd {
        #[label("this list is missing a `]` to terminate")]
        span: SourceSpan,
    },
    #[error("invalid identifier following `let` keyword")]
    #[diagnostic(code(parser::var_identifier))]
    VarIdentifier {
        #[label("invalid identifier here")]
        span: SourceSpan,
    },
    #[error("expected assignment operator `=` following identifier in variable declaration")]
    #[diagnostic(code(parser::var_assignment))]
    VarAssignment {
        #[label("expected `=` following this identifier")]
        span: SourceSpan,
    },
    #[error("expected left brace `{{` following loop statement")]
    #[diagnostic(code(parser::loop_block_begin))]
    LoopBlockBegin {
        #[label("this loop is missing a `{{` to start its body")]
        span: SourceSpan,
    },
    #[error("expected right brace `}}` to end loop block")]
    #[diagnostic(code(parser::loop_block_end))]
    LoopBlockEnd {
        #[label("this loop is missing a `}}` to end its body")]
        span: SourceSpan,
    },
    #[error("unexpected token in import")]
    #[diagnostic(
        code(parser::use_non_ident),
        help("module names should only contain alphabetical chars and `_`")
    )]
    UseNonIdent {
        #[label("this token is not supported as a module name")]
        span: SourceSpan,
    },
    #[error("token `{kind:?}` is unsupported")]
    #[diagnostic(code(parser::unsupported_token))]
    Unsupported {
        kind: TokenKind,
        #[label("unsupported token")]
        span: SourceSpan,
    },
}

enum Process {
    Break,
    Push,
}

pub struct Parser {
    tokens: VecDeque<Token>,
    src: String,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl Parser {
    #[must_use]
    pub fn new(src: String) -> Self {
        Self {
            tokens: vec![].into(),
            src,
        }
    }

    pub fn produce_ast(&mut self) -> Result<Stmt> {
        // Retrieve tokens from the lexer
        self.tokens = Lexer::new(self.src.clone()).tokenize()?.into();

        // Build out the program body
        let body = self.process(|_| Process::Push)?;

        // Return the program
        let program = Stmt::Program { body };
        Ok(program)
    }

    fn process<F>(&mut self, mut p: F) -> Result<Vec<Stmt>>
    where
        F: FnMut(&TokenKind) -> Process,
    {
        let mut body = vec![];

        while let Some(token) = &self.tokens.front() {
            match p(&token.kind) {
                Process::Break => break,
                Process::Push => {
                    body.push(self.parse_stmt()?);
                }
            }
        }

        Ok(body)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        match self.peek() {
            Some(token) => Some(&token.kind),
            None => None,
        }
    }

    fn consume(&mut self) -> Token {
        self.tokens.pop_front().expect("tokens should not be empty")
    }

    fn expect(&mut self, kind: &TokenKind, error: ParserError) -> Result<()> {
        if self.tokens.is_empty() {
            return Err(error.into());
        }

        if &self.consume().kind != kind {
            return Err(error.into());
        }

        Ok(())
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        let Some(token) = self.peek() else {
            unreachable!();
        };

        let stmt = match token.kind {
            TokenKind::Let => Stmt::Var(self.parse_var()?),
            TokenKind::Cond => Stmt::Cond(self.parse_cond()?),
            TokenKind::Func => Stmt::Func(self.parse_func()?),
            TokenKind::Loop => Stmt::Loop(self.parse_loop()?),
            TokenKind::Use => Stmt::Use(self.parse_use()?),
            _ => Stmt::Expr(self.parse_expr()?),
        };

        Ok(stmt)
    }

    fn parse_use(&mut self) -> Result<Use> {
        // Consume the `use` keyword
        self.consume();

        let mut path = String::new();
        let mut span = self.peek().unwrap().span;

        loop {
            let token = self.consume();
            let token_span = token.span;

            // First import won't be preceded by a separator
            let Token {
                kind: TokenKind::Ident(ident),
                ..
            } = token
            else {
                return Err(ParserError::UseNonIdent { span: token_span }.into());
            };

            path.push_str(&format!("{ident}/"));
            span = span.extend(token_span);

            // Subsequent arguments will be
            if self.peek_kind() == Some(&TokenKind::Slash) {
                self.consume();
            } else {
                break;
            }
        }

        // Remove trailing `/`
        path.pop();

        Ok(Use { path, span })
    }

    fn parse_loop(&mut self) -> Result<Loop> {
        // Consume the `loop` keyword
        let span = self.consume().span;

        self.expect(&TokenKind::LeftBrace, ParserError::LoopBlockBegin { span })?;

        let body = self.process(|token| match token {
            TokenKind::RightBrace => Process::Break,
            _ => Process::Push,
        })?;

        self.expect(&TokenKind::RightBrace, ParserError::LoopBlockEnd { span })?;

        Ok(Loop { body, span })
    }

    fn parse_func(&mut self) -> Result<Func> {
        // Consume the `fn` keyword
        self.consume();

        let ident_token = self.consume();
        let span = ident_token.span;

        let TokenKind::Ident(ident) = ident_token.kind else {
            return Err(ParserError::FnIdentifier { span }.into());
        };

        self.expect(&TokenKind::LeftParen, ParserError::FnArgsBegin { span })?;

        let (args, last) = self.parse_args(&TokenKind::RightParen)?;
        let last = last.unwrap_or(span.offset());

        let params: Result<Vec<Ident>, ()> = args
            .into_iter()
            .map(|expr| match expr.kind {
                ExprKind::Ident(ident) => Ok(ident),
                _ => Err(()),
            })
            .collect();

        let Ok(params) = params else {
            let span = (span.offset(), last - span.offset() + 1).into();
            return Err(ParserError::FnArgs { span }.into());
        };

        self.expect(&TokenKind::RightParen, ParserError::FnArgsEnd { span })?;

        self.expect(&TokenKind::LeftBrace, ParserError::FnBlockBegin { span })?;

        let body = self.process(|token| match token {
            TokenKind::RightBrace => Process::Break,
            _ => Process::Push,
        })?;

        self.expect(&TokenKind::RightBrace, ParserError::FnBlockEnd { span })?;

        let func = Func {
            ident,
            params,
            body,
            span,
        };

        Ok(func)
    }

    fn parse_args(&mut self, end: &TokenKind) -> Result<(Vec<Expr>, Option<usize>)> {
        let mut args = Vec::new();

        if self.peek_kind() == Some(end) {
            return Ok((args, None));
        }

        // To keep track of the last column of the last [`Expr`]
        let mut last;
        loop {
            // First argument won't be preceded by a separator
            let arg = self.parse_expr()?;
            last = Some(arg.span.offset() + arg.span.len());
            args.push(arg);

            // Subsequent arguments will be
            if self.peek_kind() == Some(&TokenKind::Separator) {
                self.consume();
            } else {
                break;
            }
        }

        Ok((args, last))
    }

    fn parse_cond(&mut self) -> Result<Cond> {
        // Consume the `if` keyword
        self.consume();

        let condition = self.parse_expr()?;
        let span = condition.span;

        self.expect(&TokenKind::LeftBrace, ParserError::CondBlockBegin { span })?;

        let body = self.process(|token| match token {
            TokenKind::RightBrace => Process::Break,
            _ => Process::Push,
        })?;

        self.expect(&TokenKind::RightBrace, ParserError::CondBlockEnd { span })?;

        let cond = Cond {
            condition,
            body,
            span,
        };

        Ok(cond)
    }

    fn parse_var(&mut self) -> Result<Var> {
        // Consume the `let` keyword
        self.consume();

        let ident_token = self.consume();

        let TokenKind::Ident(ident) = ident_token.kind else {
            return Err(ParserError::VarIdentifier {
                span: ident_token.span,
            }
            .into());
        };

        self.expect(
            &TokenKind::Assignment,
            ParserError::VarAssignment {
                span: ident_token.span,
            },
        )?;

        let var = Var {
            ident,
            value: Box::new(self.parse_expr()?.into()),
            span: ident_token.span,
        };

        Ok(var)
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_pipe_expr()?;

        if self.peek_kind() == Some(&TokenKind::Assignment) {
            self.consume();

            let right = self.parse_pipe_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::Assignment {
                    assignee: Box::new(left),
                    value: Box::new(right),
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_pipe_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_logical_or_expr()?;

        while let Some(&TokenKind::Pipe) = self.peek_kind() {
            // Consume the operator
            self.consume();

            let right = self.parse_logical_or_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::Pipe {
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_logical_or_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_logical_and_expr()?;

        while let Some(&TokenKind::Or) = self.peek_kind() {
            // Consume the operator
            self.consume();

            let right = self.parse_logical_and_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::LogicOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op: LogicOp::Or,
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_logical_and_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_rel_expr()?;

        while let Some(&TokenKind::And) = self.peek_kind() {
            // Consume the operator
            self.consume();

            let right = self.parse_rel_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::LogicOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op: LogicOp::And,
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_rel_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive_expr()?;

        if let Some(&TokenKind::RelOp(op)) = self.peek_kind() {
            // Consume the operator
            self.consume();

            let right = self.parse_additive_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::RelOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op,
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative_expr()?;

        while let Some(kind) = self.peek_kind() {
            let op = match kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };

            // Consume the operator
            self.consume();

            let right = self.parse_multiplicative_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op,
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary_expr()?;

        while let Some(kind) = self.peek_kind() {
            let op = match kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => break,
            };

            // Consume the operator
            self.consume();

            let right = self.parse_unary_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op,
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr> {
        match self.peek_kind() {
            Some(&TokenKind::Plus | &TokenKind::Minus | &TokenKind::Bang) => {
                let token = self.consume();

                let op = match token.kind {
                    TokenKind::Plus => UnaryOp::Pos,
                    TokenKind::Minus => UnaryOp::Neg,
                    TokenKind::Bang => UnaryOp::Not,
                    _ => unreachable!(),
                };

                // We should keep parsing as many unary operators as we can
                let right = self.parse_unary_expr()?;
                let span = token.span.extend(right.span);

                Ok(Expr {
                    kind: ExprKind::UnaryOp {
                        expr: Box::new(right),
                        op,
                    },
                    span,
                })
            }
            _ => self.parse_call_expr(),
        }
    }

    fn parse_call_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_mod_expr()?;

        if self.peek_kind() == Some(&TokenKind::LeftParen) {
            self.consume();

            let (args, last) = self.parse_args(&TokenKind::RightParen)?;
            let last = last.unwrap_or(left.span.offset());

            self.expect(
                &TokenKind::RightParen,
                ParserError::FnArgsEnd { span: left.span },
            )?;

            let span = (left.span.offset(), last - left.span.offset() + 1);

            left = Expr {
                kind: ExprKind::Call {
                    caller: Box::new(left),
                    args,
                },
                span: span.into(),
            }
        }

        Ok(left)
    }

    fn parse_mod_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_list_expr()?;

        while self.peek_kind() == Some(&TokenKind::Dot) {
            self.consume();

            let right = self.parse_list_expr()?;
            let span = left.span.extend(right.span);

            left = Expr {
                kind: ExprKind::Mod {
                    module: Box::new(left),
                    item: Box::new(right),
                },
                span,
            }
        }

        Ok(left)
    }

    fn parse_list_expr(&mut self) -> Result<Expr> {
        if self.peek_kind() != Some(&TokenKind::LeftBracket) {
            return self.parse_primary_expr();
        }

        let left = self.consume();

        let (items, last) = self.parse_args(&TokenKind::RightBracket)?;
        let last = last.unwrap_or(left.span.offset());

        let span = left.span.extend(last.into());

        self.expect(&TokenKind::RightBracket, ParserError::ListItemsEnd { span })?;

        Ok(Expr {
            kind: ExprKind::List { items },
            // `span` doesn't include the end `RightBracket`
            span: (span.offset(), span.len() + 1).into(),
        })
    }

    fn parse_primary_expr(&mut self) -> Result<Expr> {
        let token = self.consume();

        let expr = match token.kind {
            TokenKind::Ident(value) => Expr {
                kind: ExprKind::Ident(value),
                span: token.span,
            },
            TokenKind::Bool(value) => {
                let value = match value.as_ref() {
                    "true" => true,
                    "false" => false,
                    _ => unreachable!("`Bool` token should have value `true` or `false`"),
                };
                Expr {
                    kind: ExprKind::Bool(value),
                    span: token.span,
                }
            }
            TokenKind::Int(value) => Expr {
                kind: ExprKind::Int(
                    value
                        .parse::<i32>()
                        .expect("`Int` token should be parsed as an `i32`"),
                ),
                span: token.span,
            },
            TokenKind::Str(value) => Expr {
                kind: ExprKind::Str(value),
                span: token.span,
            },
            TokenKind::LeftParen => {
                let expr = self.parse_expr()?;
                // Consume closing parenthesis
                self.consume();
                expr
            }
            TokenKind::Return => {
                let (value, len) = if let Some(TokenKind::RightBrace) = self.peek_kind() {
                    (None, 0)
                } else {
                    let expr = self.parse_expr()?;
                    let len = expr.span.len();
                    (Some(Box::new(expr)), len)
                };
                let span = (token.span.offset(), len + token.span.len());
                Expr {
                    kind: ExprKind::Return { value },
                    span: span.into(),
                }
            }
            TokenKind::Continue => Expr {
                kind: ExprKind::Continue,
                span: token.span,
            },
            TokenKind::Break => Expr {
                kind: ExprKind::Break,
                span: token.span,
            },
            _ => {
                return Err(ParserError::Unsupported {
                    kind: token.kind,
                    span: token.span,
                }
                .into())
            }
        };

        Ok(expr)
    }
}

pub trait SourceSpanExt {
    fn extend(&self, span: SourceSpan) -> SourceSpan;
}

impl SourceSpanExt for SourceSpan {
    fn extend(&self, span: Self) -> Self {
        (self.offset(), span.offset() - self.offset() + span.len()).into()
    }
}
