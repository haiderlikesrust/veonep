use substring::Substring;

use crate::{
    error::VeonError,
    token::{Token, TokenType, Value},
};

#[derive(Debug, Clone)]
pub struct Scanner {
    pub source: String,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, VeonError> {
        let mut tokens: Vec<Token> = vec![];
        while !self.is_at_end() {
            self.start = self.current;
            let c = self.next();
            match c {
                '+' => tokens.push(Token {
                    tty: TokenType::Plus,
                    value: Value::None,
                    line: self.line,
                }),
                '-' => tokens.push(Token {
                    tty: TokenType::Minus,
                    value: Value::None,
                    line: self.line,
                }),
                '*' => tokens.push(Token {
                    tty: TokenType::Star,
                    value: Value::None,
                    line: self.line,
                }),
                '/' => tokens.push(Token {
                    tty: TokenType::Slash,
                    value: Value::None,
                    line: self.line,
                }),
                '?' => tokens.push(Token {
                    tty: TokenType::QuestionMark,
                    value: Value::None,
                    line: self.line,
                }),
                '>' => {
                    if self.next() == '=' {
                        tokens.push(Token {
                            tty: TokenType::GreaterEqual,
                            value: Value::None,
                            line: self.line,
                        })
                    } else {
                        tokens.push(Token {
                            tty: TokenType::Equal,
                            value: Value::None,
                            line: self.line,
                        })
                    }
                }
                '<' => {
                    if self.next() == '=' {
                        tokens.push(Token {
                            tty: TokenType::LessEqual,
                            value: Value::None,
                            line: self.line,
                        })
                    } else {
                        tokens.push(Token {
                            tty: TokenType::Less,
                            value: Value::None,
                            line: self.line,
                        })
                    }
                }
                '=' => {
                    if self.next() == '=' {
                        tokens.push(Token {
                            tty: TokenType::EqualEqual,
                            value: Value::None,
                            line: self.line,
                        })
                    } else {
                        tokens.push(Token {
                            tty: TokenType::Equal,
                            value: Value::None,
                            line: self.line,
                        })
                    }
                }
                '!' => {
                    if self.next() == '=' {
                        tokens.push(Token {
                            tty: TokenType::NotEqual,
                            value: Value::None,
                            line: self.line,
                        })
                    } else {
                        tokens.push(Token {
                            tty: TokenType::Not,
                            value: Value::None,
                            line: self.line,
                        })
                    }
                }
                '"' => tokens.push(self.tokenize_string()?),
                '\n' => self.line += 1,
                ' ' => (),
                _ => {}
            }
        }

        Ok(tokens)
    }

    pub fn tokenize_string(&mut self) -> Result<Token, VeonError> {
        while self.next() != '"' {}
        let text = self.source.substring(self.start + 1, self.current - 1);
        Ok(Token {
            tty: TokenType::String,
            value: Value::String(text.to_owned()),
            line: self.line,
        })
    }
    pub fn is_at_end(&self) -> bool {
        self.source.len() == self.current
    }

    pub fn next(&mut self) -> char {
        let c = self.source.chars().collect::<Vec<_>>()[self.current];
        self.current += 1;
        c
    }
}
