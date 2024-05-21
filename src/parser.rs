use std::collections::VecDeque;
use std::i32;

use crate::ast::{Expr, Stmt, StmtKind, Var};
use crate::lexer::{BinaryOp, Lexer, Token};

pub(crate) struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub(crate) fn new() -> Self {
        Self {
            tokens: vec![].into(),
        }
    }

    pub(crate) fn produce_ast(&mut self, source: String) -> Result<Stmt, ()> {
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
        let program = Stmt::new(StmtKind::Program { body });
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
            Token::Let => {
                // Consume the `let` keyword
                self.consume();
                StmtKind::Var(self.parse_var())
            }
            _ => StmtKind::Expr(self.parse_expr()),
        };

        Stmt::new(stmt)
    }

    fn parse_var(&mut self) -> Var {
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
        let mut left = self.parse_primary_expr();

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
