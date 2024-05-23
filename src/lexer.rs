use thiserror::Error;

use crate::util::is_alpha;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("token `{0}` is invalid")]
    InvalidToken(char),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Token {
    // Literals
    Ident(String),
    Int(String),

    // Keywords
    Let,
    Func,

    // Grouping Operators
    BinaryOp(BinaryOp),
    Assignment,
    Separator,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
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
            if token == Token::EndOfFile {
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

    /// Consumes all whitespace characters until a non-whitespace character is read.
    fn consume_whitespace(&mut self) {
        while self.ch == ' ' {
            self.read_char();
        }
    }

    /// Tokenizes the current character(s) and advances the cursor.
    fn next(&mut self) -> Result<Token, LexerError> {
        self.consume_whitespace();

        let token = match self.ch {
            '\0' => Token::EndOfFile,
            '=' => Token::Assignment,
            ',' => Token::Separator,
            '+' => Token::BinaryOp(BinaryOp::Add),
            '-' => Token::BinaryOp(BinaryOp::Sub),
            '*' => Token::BinaryOp(BinaryOp::Mul),
            '/' => Token::BinaryOp(BinaryOp::Div),
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '\n' => Token::EndOfLine,
            _ => {
                if is_alpha(self.ch) {
                    let ident = self.read_ident();

                    match ident.as_str() {
                        "fn" => Token::Func,
                        "let" => Token::Let,
                        _ => Token::Ident(ident),
                    }
                } else if self.ch.is_ascii_digit() {
                    Token::Int(self.read_number())
                } else {
                    return Err(LexerError::InvalidToken(self.ch));
                }
            }
        };

        self.read_char();
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
            vec![Token::EndOfLine],
            r"'\n' should produce a new line token"
        )
    }

    #[test]
    fn alphabet() {
        let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut lexer = Lexer::new(alphabet);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![Token::Ident(alphabet.to_string())],
            "All alphabetical characters should be detected"
        )
    }

    #[test]
    fn digits() {
        let digits = "1234567890";
        let mut lexer = Lexer::new(digits);
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec![Token::Int(digits.to_string())],
            "All numerical characters should be detected"
        )
    }

    #[test]
    fn multiple_token_types() {
        let source = "(12 34) abc
cba (43 21)";
        let mut lexer = Lexer::new(source);
        let tokens = vec![
            Token::LeftParen,
            Token::Int("12".to_string()),
            Token::Int("34".to_string()),
            Token::RightParen,
            Token::Ident("abc".to_string()),
            Token::EndOfLine,
            Token::Ident("cba".to_string()),
            Token::LeftParen,
            Token::Int("43".to_string()),
            Token::Int("21".to_string()),
            Token::RightParen,
        ];
        assert_eq!(lexer.tokenize().unwrap(), tokens);
    }
}
