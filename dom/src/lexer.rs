use miette::SourceSpan;
use thiserror::Error;

use crate::util::is_alpha;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("token `{0}` is invalid")]
    InvalidTokenKind(char),
    #[error("str was never terminated")]
    UnterminatedString,
    #[error("invalid escape sequence `{0}`")]
    InvalidEscapeSequence(char),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CmpOp {
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    // Literals
    Bool(String),
    Ident(String),
    Int(String),
    Str(String),

    // Keywords
    Let,
    Cond,
    Func,
    Return,
    Loop,
    Continue,
    Break,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    CmpOp(CmpOp),
    Assignment,
    Separator,

    // Grouping
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    // Misc
    EndOfLine,
    EndOfFile,
}

#[derive(Default)]
pub struct Lexer {
    buffer: Vec<char>,
    position: usize,
    cursor: usize,
    ch: char,
}

impl Lexer {
    /// Constructs a new [`Lexer`] instance from a source.
    pub fn new(source: impl Into<String>) -> Self {
        let buffer = source.into().chars().collect();
        let mut lexer = Self {
            buffer,
            ..Self::default()
        };
        lexer.read_char();
        lexer
    }

    /// Tokenizes the current buffer.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens: Vec<Token> = vec![];

        loop {
            let token = self.next()?;
            if token.kind == TokenKind::EndOfFile {
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn eof(&self) -> bool {
        self.cursor >= self.buffer.len()
    }

    /// Reads the character under the cursor without advancing the cursor and
    /// updating the current character.
    fn peek_char(&mut self) -> char {
        if self.eof() {
            '\0'
        } else {
            self.buffer[self.cursor]
        }
    }

    /// Reads the character under the cursor, advances the cursor, and
    /// updates the current character.
    fn read_char(&mut self) {
        if self.eof() {
            self.ch = '\0';
        } else {
            self.ch = self.buffer[self.cursor];
        }
        self.position = self.cursor;
        self.cursor += 1;
    }

    fn read_comment(&mut self) {
        loop {
            if self.ch == '\n' {
                break;
            }
            self.read_char();
        }
    }

    /// Reads an identifier, leaving the cursor at the last character of the identifier.
    fn read_ident(&mut self) -> String {
        let start = self.position;

        loop {
            if is_alpha(self.peek_char()) {
                self.read_char();
            } else {
                break;
            }
        }

        self.buffer[start..self.cursor].iter().collect::<String>()
    }

    /// Reads a number, leaving the cursor at the last character of the number.
    fn read_number(&mut self) -> String {
        let start = self.position;

        loop {
            if self.peek_char().is_ascii_digit() {
                self.read_char();
            } else {
                break;
            }
        }

        self.buffer[start..self.cursor].iter().collect::<String>()
    }

    fn read_str(&mut self) -> Result<String, LexerError> {
        let mut result = String::new();
        // Consume opening quote.
        self.read_char();

        while self.ch != '"' {
            match self.ch {
                '\0' => return Err(LexerError::UnterminatedString),
                '\\' => {
                    // Read escape char.
                    self.read_char();
                    match self.ch {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        _ => return Err(LexerError::InvalidEscapeSequence(self.ch)),
                    }
                }
                _ => result.push(self.ch),
            }
            self.read_char();
        }

        Ok(result)
    }

    /// Consumes all whitespace characters until a non-whitespace character is read.
    fn consume_whitespace(&mut self) {
        while self.ch == ' ' {
            self.read_char();
        }
    }

    /// Tokenizes the current character(s) and advances the cursor.
    fn next(&mut self) -> Result<Token, LexerError> {
        self.consume_whitespace();

        // Record the start position.
        let start = self.cursor;

        let kind = match self.ch {
            '\0' => TokenKind::EndOfFile,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => match self.peek_char() {
                '/' => {
                    self.read_comment();
                    return self.next();
                }
                _ => TokenKind::Slash,
            },
            '=' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    TokenKind::CmpOp(CmpOp::Eq)
                }
                _ => TokenKind::Assignment,
            },
            '!' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    TokenKind::CmpOp(CmpOp::NotEq)
                }
                _ => TokenKind::Bang,
            },
            '<' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    TokenKind::CmpOp(CmpOp::LessEq)
                }
                _ => TokenKind::CmpOp(CmpOp::Less),
            },
            '>' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    TokenKind::CmpOp(CmpOp::GreaterEq)
                }
                _ => TokenKind::CmpOp(CmpOp::Greater),
            },
            ',' => TokenKind::Separator,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '\n' => TokenKind::EndOfLine,
            '"' => TokenKind::Str(self.read_str()?),
            _ => {
                if is_alpha(self.ch) {
                    let ident = self.read_ident();

                    match ident.as_str() {
                        // Keywords
                        "let" => TokenKind::Let,
                        "if" => TokenKind::Cond,
                        "fn" => TokenKind::Func,
                        "return" => TokenKind::Return,
                        "loop" => TokenKind::Loop,
                        "continue" => TokenKind::Continue,
                        "break" => TokenKind::Break,
                        // Misc
                        "true" | "false" => TokenKind::Bool(ident),
                        _ => TokenKind::Ident(ident),
                    }
                } else if self.ch.is_ascii_digit() {
                    TokenKind::Int(self.read_number())
                } else {
                    return Err(LexerError::InvalidTokenKind(self.ch));
                }
            }
        };

        self.read_char();
        let span = SourceSpan::new((start - 1).into(), self.cursor - start);
        let token = Token { kind, span };
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![TokenKind::EndOfLine],
            r"'\n' should produce a new line token"
        )
    }

    #[test]
    fn alphabet() {
        let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut lexer = Lexer::new(alphabet);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![TokenKind::Ident(alphabet.to_string())],
            "All alphabetical characters should be detected"
        )
    }

    #[test]
    fn digits() {
        let digits = "1234567890";
        let mut lexer = Lexer::new(digits);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![TokenKind::Int(digits.to_string())],
            "All numerical characters should be detected"
        )
    }

    #[test]
    fn multiple_token_types() {
        let source = "(12 34) abc
cba (43 21)";
        let mut lexer = Lexer::new(source);
        let tokens = vec![
            TokenKind::LeftParen,
            TokenKind::Int("12".to_string()),
            TokenKind::Int("34".to_string()),
            TokenKind::RightParen,
            TokenKind::Ident("abc".to_string()),
            TokenKind::EndOfLine,
            TokenKind::Ident("cba".to_string()),
            TokenKind::LeftParen,
            TokenKind::Int("43".to_string()),
            TokenKind::Int("21".to_string()),
            TokenKind::RightParen,
        ];
        assert_eq!(lexer.tokenize().unwrap(), tokens);
    }
}
