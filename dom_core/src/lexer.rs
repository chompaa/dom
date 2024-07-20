use std::{iter::Peekable, str::Chars};

use miette::{Diagnostic, Result, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum LexerError {
    #[error("token `{ch}` is invalid")]
    InvalidTokenKind {
        ch: char,
        #[label("this token is invalid")]
        span: SourceSpan,
    },
    #[error("str was never terminated")]
    UnterminatedString {
        #[label("string beginning here never terminated")]
        span: SourceSpan,
    },
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum RelOp {
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: SourceSpan,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenKind<'a> {
    // Literals
    Bool(&'a str),
    Ident(&'a str),
    Int(&'a str),
    Str(&'a str),

    // Keywords
    Let,
    Cond,
    Func,
    Return,
    Loop,
    Continue,
    Break,

    // Operators
    And,
    Or,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    RelOp(RelOp),
    Assignment,
    Separator,

    // Grouping
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    // Modules
    Use,
    Dot,

    // Misc
    Pipe,
    EndOfLine,
    EndOfFile,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    cursor: usize,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    /// Constructs a new [`Lexer`] instance from a source.
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();
        // The current character should be initialized to the first character.
        let current_char = chars.next();

        Self {
            source,
            chars,
            cursor: 0,
            current_char,
        }
    }

    /// Tokenizes the current source.
    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>> {
        let mut tokens = vec![];

        loop {
            let token = self.next()?;
            if token.kind == TokenKind::EndOfFile {
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Reads the character under the cursor without advancing the cursor and
    /// updating the current character.
    fn peek_char(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Reads the character under the cursor, advances the cursor, and
    /// updates the current character.
    fn read_char(&mut self) {
        self.current_char = self.chars.next();
        // We choose `1` as the default here in the case of the last token, so that the
        // cursor is properly positioned.
        self.cursor += self.current_char.map_or(1, char::len_utf8);
    }

    /// Reads a comment, leaving the cursor at the last character of the comment.
    fn read_comment(&mut self) {
        while let Some(ch) = self.peek_char() {
            if *ch == '\n' {
                break;
            }
            self.read_char();
        }
        self.read_char();
    }

    /// Reads an identifier, leaving the cursor at the last character of the identifier.
    fn read_ident(&mut self) -> &'a str {
        let start = self.cursor;

        while let Some(ch) = self.peek_char() {
            if ch.is_ident() {
                self.read_char();
            } else {
                break;
            }
        }

        &self.source[start..=self.cursor]
    }

    /// Reads a number, leaving the cursor at the last character of the number.
    fn read_number(&mut self) -> &'a str {
        let start = self.cursor;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.read_char();
            } else {
                break;
            }
        }

        &self.source[start..=self.cursor]
    }

    /// Reads a string, leaving the cursor at the last character of the string.
    fn read_str(&mut self) -> Result<&'a str> {
        let start = self.cursor;
        // Consume opening quote.
        self.read_char();

        loop {
            match self.current_char {
                Some(ch) if ch == '"' => {
                    break;
                }
                None => {
                    return Err(LexerError::UnterminatedString {
                        span: (start, 1).into(),
                    }
                    .into())
                }
                _ => self.read_char(),
            }
        }

        // Exclude the start and closing quotes in the slice.
        Ok(&self.source[start + 1..self.cursor])
    }

    /// Consumes all whitespace characters until a non-whitespace character is read.
    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if !ch.is_whitespace() {
                break;
            }
            self.read_char();
        }
    }

    /// Tokenizes the current character(s) and advances the cursor.
    fn next(&mut self) -> Result<Token<'a>> {
        self.consume_whitespace();

        // Record the start position.
        let start = self.cursor;

        let Some(ch) = self.current_char else {
            return Ok(Token {
                kind: TokenKind::EndOfFile,
                span: (0, 0).into(),
            });
        };

        let kind = match ch {
            '\0' => TokenKind::EndOfFile,
            '&' => match self.peek_char() {
                Some('&') => {
                    self.read_char();
                    TokenKind::And
                }
                _ => {
                    return Err(LexerError::InvalidTokenKind {
                        ch: '&',
                        span: (start + 1, 1).into(),
                    }
                    .into())
                }
            },
            '|' => match self.peek_char() {
                Some('|') => {
                    self.read_char();
                    TokenKind::Or
                }
                Some('>') => {
                    self.read_char();
                    TokenKind::Pipe
                }
                _ => {
                    return Err(LexerError::InvalidTokenKind {
                        ch: '|',
                        span: (start + 1, 1).into(),
                    }
                    .into())
                }
            },
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => match self.peek_char() {
                Some('/') => {
                    self.read_comment();
                    return self.next();
                }
                _ => TokenKind::Slash,
            },
            '=' => match self.peek_char() {
                Some('=') => {
                    self.read_char();
                    TokenKind::RelOp(RelOp::Eq)
                }
                _ => TokenKind::Assignment,
            },
            '!' => match self.peek_char() {
                Some('=') => {
                    self.read_char();
                    TokenKind::RelOp(RelOp::NotEq)
                }
                _ => TokenKind::Bang,
            },
            '<' => match self.peek_char() {
                Some('=') => {
                    self.read_char();
                    TokenKind::RelOp(RelOp::LessEq)
                }
                _ => TokenKind::RelOp(RelOp::Less),
            },
            '>' => match self.peek_char() {
                Some('=') => {
                    self.read_char();
                    TokenKind::RelOp(RelOp::GreaterEq)
                }
                _ => TokenKind::RelOp(RelOp::Greater),
            },
            ',' => TokenKind::Separator,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '.' => TokenKind::Dot,
            '\n' => TokenKind::EndOfLine,
            '"' => TokenKind::Str(self.read_str()?),
            '0'..='9' => TokenKind::Int(self.read_number()),
            ch if ch.is_ident() => {
                let ident = self.read_ident();

                match ident {
                    // Keywords
                    "let" => TokenKind::Let,
                    "if" => TokenKind::Cond,
                    "fn" => TokenKind::Func,
                    "return" => TokenKind::Return,
                    "loop" => TokenKind::Loop,
                    "continue" => TokenKind::Continue,
                    "break" => TokenKind::Break,
                    "use" => TokenKind::Use,
                    // Misc
                    "true" | "false" => TokenKind::Bool(ident),
                    ident => TokenKind::Ident(ident),
                }
            }
            ch => {
                return Err(LexerError::InvalidTokenKind {
                    ch,
                    span: (start, 1).into(),
                }
                .into());
            }
        };

        self.read_char();

        if kind == TokenKind::EndOfLine {
            return self.next();
        }

        let span = SourceSpan::new(start.into(), self.cursor - start);
        let token = Token { kind, span };
        Ok(token)
    }
}

trait CharExt {
    fn is_ident(&self) -> bool;
}

impl CharExt for char {
    fn is_ident(&self) -> bool {
        self.is_alphabetic() || *self == '_'
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty() {
        let mut lexer = Lexer::new("");
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![],
            "Empty source should return no tokens"
        );
    }

    #[test]
    fn end_of_line() {
        let mut lexer = Lexer::new("\n");
        assert_ne!(
            lexer.tokenize().unwrap(),
            vec![Token {
                kind: TokenKind::EndOfLine,
                span: (0, 1).into()
            }],
            r"'\n' should not produce a new line token"
        )
    }

    #[test]
    fn alphabet() {
        let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut lexer = Lexer::new(alphabet);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![Token {
                kind: TokenKind::Ident(alphabet),
                span: (0, 52).into()
            }],
            "All alphabetical characters should be detected"
        )
    }

    #[test]
    fn digits() {
        let digits = "1234567890";
        let mut lexer = Lexer::new(digits);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![Token {
                kind: TokenKind::Int(digits),
                span: (0, 10).into()
            }],
            "All numerical characters should be detected"
        )
    }

    #[test]
    fn string() {
        let source = r#"("foo")"#;
        let mut lexer = Lexer::new(source);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![
                Token {
                    kind: TokenKind::LeftParen,
                    span: (0, 1).into()
                },
                Token {
                    kind: TokenKind::Str("foo"),
                    span: (1, 5).into()
                },
                Token {
                    kind: TokenKind::RightParen,
                    span: (6, 1).into()
                },
            ],
            "Strings should read properly"
        )
    }

    #[test]
    fn comment() {
        let source = "// foo = bar.baz(-1, 0)\nfoo";
        let mut lexer = Lexer::new(source);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![Token {
                kind: TokenKind::Ident("foo"),
                span: (24, 3).into()
            }],
            "Comments should read properly"
        )
    }

    #[test]
    fn multiple_types() {
        let source = "if foo <= bar { !foo }";
        let mut lexer = Lexer::new(source);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![
                Token {
                    kind: TokenKind::Cond,
                    span: (0, 2).into()
                },
                Token {
                    kind: TokenKind::Ident("foo"),
                    span: (3, 3).into()
                },
                Token {
                    kind: TokenKind::RelOp(RelOp::LessEq),
                    span: (7, 2).into()
                },
                Token {
                    kind: TokenKind::Ident("bar"),
                    span: (10, 3).into()
                },
                Token {
                    kind: TokenKind::LeftBrace,
                    span: (14, 1).into()
                },
                Token {
                    kind: TokenKind::Bang,
                    span: (16, 1).into()
                },
                Token {
                    kind: TokenKind::Ident("foo"),
                    span: (17, 3).into()
                },
                Token {
                    kind: TokenKind::RightBrace,
                    span: (21, 1).into()
                },
            ],
            "All numerical characters should be detected"
        )
    }
}
