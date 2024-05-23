//! Parser used to produce an AST from a token stream
//!
//! Order of precedence:
//! - Assignment
//! - Block
//! - Addition
//! - Multiplication
//! - Call
//! - Primary

use std::collections::VecDeque;
use std::i32;

use crate::ast::{Expr, Func, Ident, Stmt, Var};
use crate::lexer::{BinaryOp, Lexer, Token};

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: vec![].into(),
        }
    }

    pub fn produce_ast(&mut self, source: String) -> Result<Stmt, ()> {
        // Retrieve tokens from the lexer
        let mut lexer = Lexer::new(source);

        let Ok(tokens) = lexer.tokenize() else {
            return Err(());
        };
        self.tokens = tokens.into();

        // Build out the program body
        let mut body = vec![];
        while let Some(token) = self.tokens.front() {
            if *token == Token::EndOfLine {
                self.consume();
                continue;
            }
            body.push(self.parse_stmt());
        }

        // Return the program
        let program = Stmt::Program { body };
        Ok(program)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn consume(&mut self) -> Token {
        self.tokens.pop_front().expect("Tokens should not be empty")
    }

    fn parse_stmt(&mut self) -> Stmt {
        let Some(token) = self.peek() else {
            unreachable!();
        };

        let stmt = match token {
            Token::Func => Stmt::Func(self.parse_func()),
            Token::Let => Stmt::Var(self.parse_var()),
            _ => Stmt::Expr(self.parse_expr()),
        };

        stmt
    }

    fn parse_func(&mut self) -> Func {
        // TODO: Panics -> Results

        // Consume the `fn` keyword
        self.consume();

        let Token::Ident(ident) = self.consume() else {
            panic!("Expected identifier name following `fn` keyword");
        };

        if self.consume() != Token::LeftParen {
            panic!("Expected open parenthesis");
        }

        let params: Result<Vec<Ident>, ()> = self
            .parse_args()
            .into_iter()
            .map(|expr| match expr {
                Expr::Ident(ident) => Ok(ident),
                _ => Err(()),
            })
            .collect();

        let Ok(params) = params else {
            panic!("Expected identifiers in function arguments");
        };

        if Token::LeftBrace != self.consume() {
            panic!("Expected block following function declaration");
        }

        let mut body = vec![];
        while let Some(token) = self.tokens.front() {
            match *token {
                Token::RightBrace => break,
                Token::EndOfLine => {
                    self.consume();
                }
                _ => body.push(self.parse_stmt()),
            };
        }

        if Token::RightBrace != self.consume() {
            panic!("Expected closure at end of block");
        }

        Func {
            ident,
            params,
            body,
        }
    }

    fn parse_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();

        if self.peek() == Some(&Token::RightParen) {
            self.consume();
            return args;
        }

        // First argument won't be preceded by a separator
        args.push(self.parse_assignment_expr());

        // Get all separated arguments
        while self.peek() == Some(&Token::Separator) {
            self.consume();
            args.push(self.parse_assignment_expr())
        }

        if self.consume() != Token::RightParen {
            panic!("Expected close parenthesis");
        }

        args
    }

    fn parse_var(&mut self) -> Var {
        // Consume the `let` keyword
        self.consume();

        let Token::Ident(ident) = self.consume() else {
            panic!("Expected identifier name following `let` keyword");
        };

        if self.consume() != Token::Assignment {
            panic!("Expected assignment operator `=` following identifier in variable declaration");
        };

        let var = Var {
            ident,
            value: Box::new(self.parse_expr().into()),
        };

        if self.consume() != Token::EndOfLine {
            panic!(r"Expected newline `\n` at end of variable declaration")
        };

        var
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Expr {
        let left = self.parse_additive_expr();

        if self.peek() == Some(&Token::Assignment) {
            self.consume();

            let value = self.parse_assignment_expr();

            return Expr::Assignment {
                assignee: Box::new(left),
                value: Box::new(value),
            };
        }

        left
    }

    fn parse_additive_expr(&mut self) -> Expr {
        let mut left = self.parse_multiplicative_expr();

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
            let right = self.parse_multiplicative_expr();
            left = Expr::BinaryOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            }
        }

        left
    }

    fn parse_multiplicative_expr(&mut self) -> Expr {
        let mut left = self.parse_call_expr();

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
            let right = self.parse_multiplicative_expr();
            left = Expr::BinaryOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            }
        }

        left
    }

    fn parse_call_expr(&mut self) -> Expr {
        let left = self.parse_primary_expr();

        if self.peek() == Some(&Token::LeftParen) {
            self.consume();

            return Expr::Call {
                caller: Box::new(left),
                args: self.parse_args(),
            };
        }

        left
    }

    fn parse_primary_expr(&mut self) -> Expr {
        let token = self.consume();

        match token {
            Token::Ident(value) => Expr::Ident(value),
            Token::Int(value) => Expr::Int(
                value
                    .parse::<i32>()
                    .expect("`Int` token should be parsed as an `i32`"),
            ),
            Token::LeftParen => {
                let expr = self.parse_expr();
                // Consume closing parenthesis
                self.consume();
                expr
            }
            _ => Expr::Ident("NIL".into()),
        }
    }
}
