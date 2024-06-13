//! Parser used to produce an AST from a token stream
//!
//! Order of precedence (low to high):
//! - Assignments
//! - Comparison Operators
//! - Binary Addition
//! - Binary Multiplication
//! - Unary Operators
//! - Function Call
//! - Primary Expressions

use std::collections::VecDeque;
use std::i32;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::ast::{BinaryOp, Cond, Expr, ExprKind, Func, Ident, Loop, Stmt, UnaryOp, Var};
use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Error, Diagnostic, Debug)]
pub enum ParserError {
    #[error("expected left brace `{{` following conditional statement")]
    #[diagnostic(code(parser::cond_block_begin))]
    CondBlockBegin {
        #[source_code]
        src: String,
        #[label("this conditional is missing a `{{` to start its body")]
        span: SourceSpan,
    },
    #[error("expected right brace `}}` to end conditional block")]
    #[diagnostic(code(parser::cond_block_end))]
    CondBlockEnd {
        #[source_code]
        src: String,
        #[label("this conditional is missing a `}}` to end its body")]
        span: SourceSpan,
    },
    #[error("invalid identifier following `fn` keyword")]
    #[diagnostic(code(parser::fn_identifier))]
    FnIdentifier {
        #[source_code]
        src: String,
        #[label("invalid identifier here")]
        span: SourceSpan,
    },
    #[error("expected left parenthesis `(` following identifier to begin function arguments")]
    #[diagnostic(code(parser::fn_args_begin))]
    FnArgsBegin {
        #[source_code]
        src: String,
        #[label("expected `(` following this identifier")]
        span: SourceSpan,
    },
    #[error("expected only identifiers in function arguments")]
    #[diagnostic(code(parser::fn_args))]
    FnArgs {
        #[source_code]
        src: String,
        #[label("this function has non-identifier arguments")]
        span: SourceSpan,
    },
    #[error("expected right parenthesis `)` to end argument list")]
    #[diagnostic(code(parser::fn_args_end))]
    FnArgsEnd {
        #[source_code]
        src: String,
        #[label("this function's arguments are never ended")]
        span: SourceSpan,
    },
    #[error("expected left brace `{{` following function arguments")]
    #[diagnostic(code(parser::fn_block_begin))]
    FnBlockBegin {
        #[source_code]
        src: String,
        #[label("this function is missing a `{{` to start its body")]
        span: SourceSpan,
    },
    #[error("expected right brace `}}` to end function block")]
    #[diagnostic(code(parser::fn_block_end))]
    FnBlockEnd {
        #[source_code]
        src: String,
        #[label("this function is missing a `}}` to end its body")]
        span: SourceSpan,
    },
    #[error("invalid identifier following `let` keyword")]
    #[diagnostic(code(parser::var_identifier))]
    VarIdentifier {
        #[source_code]
        src: String,
        #[label("invalid identifier here")]
        span: SourceSpan,
    },
    #[error("expected assignment operator `=` following identifier in variable declaration")]
    #[diagnostic(code(parser::var_assignment))]
    VarAssignment {
        #[source_code]
        src: String,
        #[label("expected `=` following this identifier")]
        span: SourceSpan,
    },
    #[error("expected left brace `{{` following loop statement")]
    #[diagnostic(code(parser::loop_block_begin))]
    LoopBlockBegin {
        #[source_code]
        src: String,
        #[label("this loop is missing a `{{` to start its body")]
        span: SourceSpan,
    },
    #[error("expected right brace `}}` to end loop block")]
    #[diagnostic(code(parser::loop_block_end))]
    LoopBlockEnd {
        #[source_code]
        src: String,
        #[label("this loop is missing a `}}` to end its body")]
        span: SourceSpan,
    },
    #[error("token `{kind:?}` is unsupported")]
    #[diagnostic(code(parser::unsupported_token))]
    Unsupported {
        kind: TokenKind,
        #[source_code]
        src: String,
        #[label("unsupported token")]
        span: SourceSpan,
    },
}

impl ParserError {
    pub fn to_report(self) -> String {
        let report: miette::ErrReport = self.into();
        format!("{report:?}")
    }
}

enum Process {
    Break,
    Consume,
    Push,
}

pub struct Parser {
    tokens: VecDeque<Token>,
    source: String,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl Parser {
    #[must_use]
    pub fn new(source: String) -> Self {
        Self {
            tokens: vec![].into(),
            source,
        }
    }

    pub fn produce_ast(&mut self) -> Result<Stmt, ParserError> {
        // Retrieve tokens from the lexer
        let mut lexer = Lexer::new(self.source.to_string());

        let Ok(tokens) = lexer.tokenize() else {
            // TODO: No panic
            panic!("lexer err");
        };
        self.tokens = tokens.into();

        // Build out the program body
        let body = self.process(|token| match token {
            TokenKind::EndOfLine => Process::Consume,
            _ => Process::Push,
        })?;

        // Return the program
        let program = Stmt::Program { body };
        Ok(program)
    }

    fn process<F>(&mut self, mut p: F) -> Result<Vec<Stmt>, ParserError>
    where
        F: FnMut(&TokenKind) -> Process,
    {
        let mut body = vec![];

        while let Some(token) = &self.tokens.front() {
            match p(&token.kind) {
                Process::Break => break,
                Process::Consume => {
                    self.consume();
                }
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

    fn expect(&mut self, kind: &TokenKind, error: ParserError) -> Result<(), ParserError> {
        if self.tokens.is_empty() {
            return Err(error);
        }

        if &self.consume().kind != kind {
            return Err(error);
        }

        Ok(())
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        let Some(token) = self.peek() else {
            unreachable!();
        };

        let stmt = match token.kind {
            TokenKind::Let => Stmt::Var(self.parse_var()?),
            TokenKind::Cond => Stmt::Cond(self.parse_cond()?),
            TokenKind::Func => Stmt::Func(self.parse_func()?),
            TokenKind::Loop => Stmt::Loop(self.parse_loop()?),
            _ => Stmt::Expr(self.parse_expr()?),
        };

        Ok(stmt)
    }

    fn parse_loop(&mut self) -> Result<Loop, ParserError> {
        // Consume the `loop` keyword
        let token = self.consume();

        self.expect(
            &TokenKind::LeftBrace,
            ParserError::LoopBlockBegin {
                src: self.source.clone(),
                span: token.span,
            },
        )?;

        let body = self.process(|token| match token {
            TokenKind::RightBrace => Process::Break,
            TokenKind::EndOfLine => Process::Consume,
            _ => Process::Push,
        })?;

        self.expect(
            &TokenKind::RightBrace,
            ParserError::LoopBlockEnd {
                src: self.source.clone(),
                span: token.span,
            },
        )?;

        Ok(Loop { body })
    }

    fn parse_func(&mut self) -> Result<Func, ParserError> {
        // Consume the `fn` keyword
        self.consume();

        let ident_token = self.consume();

        let TokenKind::Ident(ident) = ident_token.kind else {
            return Err(ParserError::FnIdentifier {
                src: self.source.clone(),
                span: ident_token.span,
            });
        };

        self.expect(
            &TokenKind::LeftParen,
            ParserError::FnArgsBegin {
                src: self.source.clone(),
                span: ident_token.span,
            },
        )?;

        let (args, len) = self.parse_args()?;

        let params: Result<Vec<Ident>, ()> = args
            .into_iter()
            .map(|expr| match expr.kind {
                ExprKind::Ident(ident) => Ok(ident),
                _ => Err(()),
            })
            .collect();

        let Ok(params) = params else {
            let span = (ident_token.span.offset(), ident_token.span.len() + len);
            return Err(ParserError::FnArgs {
                src: self.source.clone(),
                span: span.into(),
            });
        };

        self.expect(
            &TokenKind::RightParen,
            ParserError::FnArgsEnd {
                src: self.source.clone(),
                span: ident_token.span,
            },
        )?;

        self.expect(
            &TokenKind::LeftBrace,
            ParserError::FnBlockBegin {
                src: self.source.clone(),
                span: ident_token.span,
            },
        )?;

        let body = self.process(|token| match token {
            TokenKind::RightBrace => Process::Break,
            TokenKind::EndOfLine => Process::Consume,
            _ => Process::Push,
        })?;

        self.expect(
            &TokenKind::RightBrace,
            ParserError::FnBlockEnd {
                src: self.source.clone(),
                span: ident_token.span,
            },
        )?;

        let func = Func {
            ident,
            params,
            body,
        };

        Ok(func)
    }

    fn parse_args(&mut self) -> Result<(Vec<Expr>, usize), ParserError> {
        let mut args = Vec::new();
        let mut len = 0;

        if self.peek_kind() == Some(&TokenKind::RightParen) {
            self.consume();
            return Ok((args, 0));
        }

        // First argument won't be preceded by a separator
        let arg = self.parse_assignment_expr()?;
        len += arg.span.len();
        args.push(arg);

        // Get all separated arguments
        while self.peek_kind() == Some(&TokenKind::Separator) {
            self.consume();
            // TODO: Better error handling for no more tokens
            let arg = self.parse_assignment_expr()?;
            len += arg.span.len();
            args.push(arg);
        }

        Ok((args, len))
    }

    fn parse_cond(&mut self) -> Result<Cond, ParserError> {
        // Consume the `if` keyword
        self.consume();

        let condition = self.parse_expr()?;

        self.expect(
            &TokenKind::LeftBrace,
            ParserError::CondBlockBegin {
                src: self.source.clone(),
                span: condition.span,
            },
        )?;

        let body = self.process(|token| match token {
            TokenKind::RightBrace => Process::Break,
            TokenKind::EndOfLine => Process::Consume,
            _ => Process::Push,
        })?;

        self.expect(
            &TokenKind::RightBrace,
            ParserError::CondBlockEnd {
                src: self.source.clone(),
                span: condition.span,
            },
        )?;

        let cond = Cond { condition, body };

        Ok(cond)
    }

    fn parse_var(&mut self) -> Result<Var, ParserError> {
        // Consume the `let` keyword
        self.consume();

        let ident_token = self.consume();

        let TokenKind::Ident(ident) = ident_token.kind else {
            return Err(ParserError::VarIdentifier {
                src: self.source.clone(),
                span: ident_token.span,
            });
        };

        self.expect(
            &TokenKind::Assignment,
            ParserError::VarAssignment {
                src: self.source.clone(),
                span: ident_token.span,
            },
        )?;

        let var = Var {
            ident,
            value: Box::new(self.parse_expr()?.into()),
        };

        Ok(var)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_comparison_expr()?;

        if self.peek_kind() == Some(&TokenKind::Assignment) {
            self.consume();

            let right = self.parse_assignment_expr()?;
            let span = (
                left.span.offset(),
                (right.span.offset() - left.span.offset()) + right.span.len(),
            );

            left = Expr {
                kind: ExprKind::Assignment {
                    assignee: Box::new(left),
                    value: Box::new(right),
                },
                span: span.into(),
            }
        }

        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_additive_expr()?;

        if let Some(&TokenKind::CmpOp(op)) = self.peek_kind() {
            // Consume the operator
            self.consume();

            let right = self.parse_additive_expr()?;
            let span = (
                left.span.offset(),
                (right.span.offset() - left.span.offset()) + right.span.len(),
            );

            left = Expr {
                kind: ExprKind::CmpOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op,
                },
                span: span.into(),
            }
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr, ParserError> {
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
            let span = (
                left.span.offset(),
                (right.span.offset() - left.span.offset()) + right.span.len(),
            );

            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op,
                },
                span: span.into(),
            }
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_unary_expr()?;

        while let Some(kind) = self.peek_kind() {
            let op = match kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => break,
            };

            // Consume the operator
            self.consume();

            let right = self.parse_multiplicative_expr()?;
            let span = (
                left.span.offset(),
                (right.span.offset() - left.span.offset()) + right.span.len(),
            );

            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op,
                },
                span: span.into(),
            }
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParserError> {
        match self.peek_kind() {
            Some(&TokenKind::Plus | &TokenKind::Minus | &TokenKind::Bang) => {
                let token = self.consume();

                let op = match token.kind {
                    TokenKind::Plus => UnaryOp::Pos,
                    TokenKind::Minus => UnaryOp::Neg,
                    TokenKind::Bang => UnaryOp::Not,
                    _ => unreachable!(),
                };

                let right = self.parse_unary_expr()?;
                let span = (
                    token.span.offset(),
                    right.span.offset() - token.span.offset() + right.span.len(),
                );

                Ok(Expr {
                    kind: ExprKind::UnaryOp {
                        // We should keep parsing as many unary operators as we can
                        expr: Box::new(self.parse_unary_expr()?),
                        op,
                    },
                    span: span.into(),
                })
            }
            _ => self.parse_call_expr(),
        }
    }

    fn parse_call_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_primary_expr()?;

        if self.peek_kind() == Some(&TokenKind::LeftParen) {
            let token = self.consume();

            let (args, len) = self.parse_args()?;

            self.expect(
                &TokenKind::RightParen,
                ParserError::FnArgsEnd {
                    src: self.source.clone(),
                    span: left.span,
                },
            )?;

            let span = (left.span.offset(), left.span.len() + token.span.len() + len);

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

    fn parse_primary_expr(&mut self) -> Result<Expr, ParserError> {
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
                let (value, len) = if let Some(TokenKind::EndOfLine) = self.peek_kind() {
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
                    src: self.source.clone(),
                    span: token.span,
                })
            }
        };

        Ok(expr)
    }
}
