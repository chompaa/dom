//! Parser used to produce an AST from a token stream
//!
//! Order of precedence:
//! - Assignment
//! - Block
//! - Comparison
//! - Addition
//! - Multiplication
//! - Call
//! - Primary

use std::collections::VecDeque;
use std::i32;

use thiserror::Error;

use crate::ast::{Cond, Expr, Func, Ident, Return, Stmt, Var};
use crate::lexer::{BinaryOp, Lexer, Token};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("expected identifier following `fn` keyword")]
    FnIdentifier,
    #[error("expected left parenthesis '(' following function declaration")]
    FnArgsBegin,
    #[error("expected identifiers in function arguments")]
    FnArgs,
    #[error("expected right parenthesis ')' to end argument list")]
    FnArgsEnd,
    #[error("expected left brace '{{' following function arguments")]
    FnBlockBegin,
    #[error("expected right brace '}}' to end function block")]
    FnBlockEnd,
    #[error("expected identifier following `let` keyword")]
    VarIdentifier,
    #[error("expected assignment operator '=' following identifier in variable declaration")]
    VarAssign,
    #[error("expected newline '\\n' at end of variable declaration")]
    VarEndOfLine,
    #[error("token {0:?} is unsupported")]
    Unsupported(Token),
}

enum Process {
    Break,
    Consume,
    Push,
}

pub struct Parser {
    line: u32,
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            line: 1,
            tokens: vec![].into(),
        }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn produce_ast(&mut self, source: String) -> Result<Stmt, Box<dyn std::error::Error>> {
        // Retrieve tokens from the lexer
        let mut lexer = Lexer::new(source);

        let tokens = lexer.tokenize()?;
        self.tokens = tokens.into();

        // Build out the program body
        let body = self.process(|token| match token {
            Token::EndOfLine => Process::Consume,
            _ => Process::Push,
        })?;

        // Return the program
        let program = Stmt::Program { body };
        Ok(program)
    }

    fn process<F>(&mut self, mut _process: F) -> Result<Vec<Stmt>, ParserError>
    where
        F: FnMut(&Token) -> Process,
    {
        let mut body = vec![];

        while let Some(token) = &self.tokens.front() {
            match _process(token) {
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

    fn consume(&mut self) -> Token {
        let token = self.tokens.pop_front().expect("tokens should not be empty");

        // If the last token is an `EndOfLine` token, don't increase the line counter since if a
        // [`ParserError`] occurs there, lines will be miscounted
        if token == Token::EndOfLine && !self.tokens.is_empty() {
            self.line += 1;
        }

        token
    }

    fn expect(&mut self, token: Token, error: ParserError) -> Result<(), ParserError> {
        if self.tokens.is_empty() {
            return Err(error);
        }

        if self.consume() != token {
            return Err(error);
        }

        Ok(())
    }

    fn expect_not(&mut self, token: Token, error: ParserError) -> Result<(), ParserError> {
        if self.tokens.is_empty() {
            return Err(error);
        }

        if self.consume() == token {
            return Err(error);
        }

        Ok(())
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        let Some(token) = self.peek() else {
            unreachable!();
        };

        let stmt = match token {
            Token::Let => Stmt::Var(self.parse_var()?),
            Token::Cond => Stmt::Cond(self.parse_cond()?),
            Token::Func => Stmt::Func(self.parse_func()?),
            Token::Return => Stmt::Return(self.parse_return()?),
            _ => Stmt::Expr(self.parse_expr()?),
        };

        Ok(stmt)
    }

    fn parse_return(&mut self) -> Result<Return, ParserError> {
        // Consume the `return` keyword
        self.consume();

        if self.peek() == Some(&Token::EndOfLine) {
            let result = Return { value: None };
            return Ok(result);
        }

        let result = Return {
            value: Some(self.parse_expr()?),
        };

        Ok(result)
    }

    fn parse_func(&mut self) -> Result<Func, ParserError> {
        // Consume the `fn` keyword
        self.consume();

        let Token::Ident(ident) = self.consume() else {
            return Err(ParserError::FnIdentifier);
        };

        self.expect(Token::LeftParen, ParserError::FnArgsBegin)?;

        let params: Result<Vec<Ident>, ()> = self
            .parse_args()?
            .into_iter()
            .map(|expr| match expr {
                Expr::Ident(ident) => Ok(ident),
                _ => Err(()),
            })
            .collect();

        let Ok(params) = params else {
            return Err(ParserError::FnArgs);
        };

        self.expect(Token::LeftBrace, ParserError::FnBlockBegin)?;

        let body = self.process(|token| match token {
            Token::RightBrace => Process::Break,
            Token::EndOfLine => Process::Consume,
            _ => Process::Push,
        })?;

        self.expect(Token::RightBrace, ParserError::FnBlockEnd)?;

        let func = Func {
            ident,
            params,
            body,
        };

        Ok(func)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();

        if self.peek() == Some(&Token::RightParen) {
            self.consume();
            return Ok(args);
        }

        // First argument won't be preceded by a separator
        args.push(self.parse_assignment_expr()?);

        // Get all separated arguments
        while self.peek() == Some(&Token::Separator) {
            self.consume();
            // TODO: Better error handling for no more tokens
            args.push(self.parse_assignment_expr()?)
        }

        self.expect(Token::RightParen, ParserError::FnArgsEnd)?;

        Ok(args)
    }

    fn parse_cond(&mut self) -> Result<Cond, ParserError> {
        // Consume the `if` keyword
        self.consume();

        let condition = self.parse_expr()?;

        self.expect(Token::LeftBrace, ParserError::FnBlockBegin)?;

        let body = self.process(|token| match token {
            Token::RightBrace => Process::Break,
            Token::EndOfLine => Process::Consume,
            _ => Process::Push,
        })?;

        self.expect(Token::RightBrace, ParserError::FnBlockEnd)?;

        let cond = Cond { condition, body };

        Ok(cond)
    }

    fn parse_var(&mut self) -> Result<Var, ParserError> {
        // Consume the `let` keyword
        self.consume();

        let Token::Ident(ident) = self.consume() else {
            return Err(ParserError::VarIdentifier);
        };

        self.expect(Token::Assignment, ParserError::VarAssign)?;

        let var = Var {
            ident,
            value: Box::new(self.parse_expr()?.into()),
        };

        self.expect(Token::EndOfLine, ParserError::VarEndOfLine)?;

        Ok(var)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_comparison_expr()?;

        if self.peek() == Some(&Token::Assignment) {
            self.consume();

            let value = self.parse_assignment_expr()?;

            left = Expr::Assignment {
                assignee: Box::new(left),
                value: Box::new(value),
            };
        }

        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_additive_expr()?;

        let Some(&Token::CmpOp(op)) = self.peek() else {
            return Ok(left);
        };

        self.consume();
        let right = self.parse_additive_expr()?;
        left = Expr::CmpOp {
            left: Box::new(left),
            right: Box::new(right),
            op,
        };

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_multiplicative_expr()?;

        loop {
            let Some(token) = self.peek() else {
                break;
            };

            let op = match token {
                Token::BinaryOp(op) => {
                    if *op != BinaryOp::Add && *op != BinaryOp::Sub {
                        break;
                    }
                    *op
                }
                _ => break,
            };

            self.consume();
            let right = self.parse_multiplicative_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            }
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_call_expr()?;

        loop {
            let Some(token) = self.peek() else {
                break;
            };

            let op = match token {
                Token::BinaryOp(op) => {
                    if *op != BinaryOp::Mul && *op != BinaryOp::Div {
                        break;
                    }
                    *op
                }
                _ => break,
            };

            self.consume();
            let right = self.parse_multiplicative_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            }
        }

        Ok(left)
    }

    fn parse_call_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_primary_expr()?;

        if self.peek() == Some(&Token::LeftParen) {
            self.consume();

            left = Expr::Call {
                caller: Box::new(left),
                args: self.parse_args()?,
            };
        }

        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParserError> {
        let token = self.consume();

        let expr = match token {
            Token::Ident(value) => Expr::Ident(value),
            Token::Bool(value) => {
                let value = match value.as_ref() {
                    "true" => true,
                    "false" => false,
                    _ => unreachable!("`Bool` token should have value `true` or `false`"),
                };
                Expr::Bool(value)
            }
            Token::Int(value) => Expr::Int(
                value
                    .parse::<i32>()
                    .expect("`Int` token should be parsed as an `i32`"),
            ),
            Token::Str(value) => Expr::Str(value),
            Token::LeftParen => {
                let expr = self.parse_expr()?;
                // Consume closing parenthesis
                self.consume();
                expr
            }
            _ => return Err(ParserError::Unsupported(token)),
        };

        Ok(expr)
    }
}
