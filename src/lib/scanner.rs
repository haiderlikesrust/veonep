use crate::{
    error::{ScannerError, ScannerErrorType, VeonError},
    token::{Token, TokenType, Value},
};

#[derive(Debug, Clone)]
pub struct Scanner {
    pub source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, VeonError> {
        let mut tokens: Vec<Token> = vec![];
        while !self.is_at_end() {
            self.start = self.current;
            let c = self.advance();
            match c {
                '(' => tokens.push(self.simple(TokenType::LeftParen)),
                ')' => tokens.push(self.simple(TokenType::RightParen)),
                '{' => tokens.push(self.simple(TokenType::LeftBrace)),
                '}' => tokens.push(self.simple(TokenType::RightBrace)),
                '[' => tokens.push(self.simple(TokenType::LeftBracket)),
                ']' => tokens.push(self.simple(TokenType::RightBracket)),
                ',' => tokens.push(self.simple(TokenType::Comma)),
                ';' => tokens.push(self.simple(TokenType::Semicolon)),
                '.' => tokens.push(self.simple(TokenType::Dot)),
                '+' => tokens.push(self.simple(TokenType::Plus)),
                '-' => tokens.push(self.simple(TokenType::Minus)),
                '*' => tokens.push(self.simple(TokenType::Star)),
                '/' => {
                    if self.match_char('/') {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        tokens.push(self.simple(TokenType::Slash))
                    }
                }
                '%' => tokens.push(self.simple(TokenType::Modulo)),
                '?' => tokens.push(self.simple(TokenType::QuestionMark)),
                '>' => {
                    if self.match_char('=') {
                        tokens.push(self.simple(TokenType::GreaterEqual))
                    } else {
                        tokens.push(self.simple(TokenType::Greater))
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        tokens.push(self.simple(TokenType::LessEqual))
                    } else {
                        tokens.push(self.simple(TokenType::Less))
                    }
                }
                '=' => {
                    if self.match_char('=') {
                        tokens.push(self.simple(TokenType::EqualEqual))
                    } else {
                        tokens.push(self.simple(TokenType::Equal))
                    }
                }
                '!' => {
                    if self.match_char('=') {
                        tokens.push(self.simple(TokenType::NotEqual))
                    } else {
                        tokens.push(self.simple(TokenType::Not))
                    }
                }
                '"' => tokens.push(self.tokenize_string()?),
                c if c.is_ascii_digit() => tokens.push(self.tokenize_number()?),
                c if Self::is_alpha(c) => tokens.push(self.tokenize_identifier()?),
                '\n' => self.line += 1,
                ' ' | '\r' | '\t' => (),
                _ => {
                    return Err(VeonError::ScannerError(ScannerError {
                        msg: format!("Invalid token: {c}"),
                        tty: ScannerErrorType::InvalidToken,
                    }))
                }
            }
        }

        tokens.push(Token {
            tty: TokenType::Eof,
            value: Value::None,
            line: self.line,
        });

        Ok(tokens)
    }

    pub fn tokenize_string(&mut self) -> Result<Token, VeonError> {
        while self.peek() != '"' {
            if self.is_at_end() {
                return Err(VeonError::ScannerError(ScannerError {
                    msg: "Unterminated string".to_string(),
                    tty: ScannerErrorType::InvalidToken,
                }));
            }

            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        // closing quote
        self.advance();

        let text = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect::<String>();
        Ok(Token {
            tty: TokenType::String,
            value: Value::String(text.to_owned()),
            line: self.line,
        })
    }

    pub fn tokenize_number(&mut self) -> Result<Token, VeonError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let text = self.lexeme();
        let value = text.parse::<isize>().map_err(|_| {
            VeonError::ScannerError(ScannerError {
                msg: format!("Invalid number literal: {text}"),
                tty: ScannerErrorType::InvalidToken,
            })
        })?;

        Ok(Token {
            tty: TokenType::Number,
            value: Value::Number(value),
            line: self.line,
        })
    }

    pub fn tokenize_identifier(&mut self) -> Result<Token, VeonError> {
        while Self::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.lexeme();
        let (tty, value) = match text.as_str() {
            "fun" => (TokenType::Fun, Value::None),
            "let" => (TokenType::Let, Value::None),
            "class" => (TokenType::Class, Value::None),
            "this" => (TokenType::This, Value::None),
            "if" => (TokenType::If, Value::None),
            "else" => (TokenType::Else, Value::None),
            "while" => (TokenType::While, Value::None),
            "for" => (TokenType::For, Value::None),
            "return" => (TokenType::Return, Value::None),
            "and" => (TokenType::And, Value::None),
            "or" => (TokenType::Or, Value::None),
            "true" => (TokenType::Boolean, Value::Boolean(true)),
            "false" => (TokenType::Boolean, Value::Boolean(false)),
            "null" => (TokenType::Null, Value::Null),
            _ => (TokenType::Identifier, Value::String(text)),
        };

        Ok(Token {
            tty,
            value,
            line: self.line,
        })
    }

    pub fn is_at_end(&self) -> bool {
        self.source.len() == self.current
    }

    pub fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn lexeme(&self) -> String {
        self.source[self.start..self.current]
            .iter()
            .collect::<String>()
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Self::is_alpha(c) || c.is_ascii_digit()
    }

    fn simple(&self, tty: TokenType) -> Token {
        Token {
            tty,
            value: Value::None,
            line: self.line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_literals_and_keywords() {
        let source = "let greeting = \"Hey\" + \"WOW\"\n42";
        let mut scanner = Scanner::new(source.to_string());
        let tokens = scanner.tokenize().expect("scan tokens");

        let types: Vec<TokenType> = tokens.iter().map(|token| token.tty.clone()).collect();
        assert_eq!(
            types,
            vec![
                TokenType::Let,
                TokenType::Identifier,
                TokenType::Equal,
                TokenType::String,
                TokenType::Plus,
                TokenType::String,
                TokenType::Number,
                TokenType::Eof
            ]
        );

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[3].value, Value::String("Hey".to_string()));
        assert_eq!(tokens[5].value, Value::String("WOW".to_string()));
        assert_eq!(tokens[6].value, Value::Number(42));
        assert_eq!(tokens[6].line, 2);
    }

    #[test]
    fn fail_on_unterminated_string() {
        let mut scanner = Scanner::new("\"oops".to_string());
        let err = scanner.tokenize().expect_err("should error");
        assert!(matches!(err, VeonError::ScannerError(_)));
    }

    #[test]
    fn skip_comments_and_handle_modulo() {
        let source = "10 % 3 // comment\n5";
        let mut scanner = Scanner::new(source.to_string());
        let tokens = scanner.tokenize().expect("scan tokens");

        let types: Vec<TokenType> = tokens.iter().map(|token| token.tty.clone()).collect();
        assert_eq!(
            types,
            vec![
                TokenType::Number,
                TokenType::Modulo,
                TokenType::Number,
                TokenType::Number,
                TokenType::Eof
            ]
        );

        assert_eq!(tokens[0].value, Value::Number(10));
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[3].line, 2);
    }
}
