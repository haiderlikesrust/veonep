use crate::{
    error::{ParserError, ParserErrorType, VeonError},
    token::{Token, TokenType, Value},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Unary {
        operator: TokenType,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Array(Vec<Expr>),
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: TokenType,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: String,
    },
    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
    },
    This,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Class {
        name: String,
        methods: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, VeonError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, VeonError> {
        if self.matches(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.matches(&[TokenType::Fun]) {
            self.function()
        } else if self.matches(&[TokenType::Let]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt, VeonError> {
        self.consume(TokenType::Identifier, "Expected class name")?;
        let name = self.previous_identifier()?;

        self.consume(TokenType::LeftBrace, "Expect '{' before class body")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            self.consume(TokenType::Fun, "Expect 'fun' before method")?;
            methods.push(self.function()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body")?;
        Ok(Stmt::Class { name, methods })
    }

    fn function(&mut self) -> Result<Stmt, VeonError> {
        self.consume(TokenType::Identifier, "Expected function name")?;
        let name = self.previous_identifier()?;

        self.consume(TokenType::LeftParen, "Expect '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                self.consume(TokenType::Identifier, "Expect parameter name")?;
                params.push(self.previous_identifier()?);
                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters")?;

        self.consume(TokenType::LeftBrace, "Expect '{' before function body")?;
        let body = self.block()?;
        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> Result<Stmt, VeonError> {
        self.consume(TokenType::Identifier, "Expected variable name")?;
        let var_name = self.previous_identifier()?;
        let initializer = if self.matches(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration")?;
        Ok(Stmt::Var {
            name: var_name,
            initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, VeonError> {
        if self.matches(&[TokenType::If]) {
            self.if_statement()
        } else if self.matches(&[TokenType::While]) {
            self.while_statement()
        } else if self.matches(&[TokenType::For]) {
            self.for_statement()
        } else if self.matches(&[TokenType::Return]) {
            self.return_statement()
        } else if self.matches(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, VeonError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, VeonError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition'")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, VeonError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'")?;

        let initializer = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Let]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Value::Boolean(true))
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)]);
        }

        let body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        let result = if let Some(init) = initializer {
            Stmt::Block(vec![init, body])
        } else {
            body
        };

        Ok(result)
    }

    fn return_statement(&mut self) -> Result<Stmt, VeonError> {
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, VeonError> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, VeonError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, VeonError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, VeonError> {
        let expr = self.or()?;

        if self.matches(&[TokenType::Equal]) {
            let value = self.assignment()?;
            match expr {
                Expr::Variable(name) => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                }
                Expr::Get { object, name } => {
                    return Ok(Expr::Set {
                        object,
                        name,
                        value: Box::new(value),
                    })
                }
                _ => return Err(Self::error("Invalid assignment target")),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, VeonError> {
        let mut expr = self.and()?;

        while self.matches(&[TokenType::Or]) {
            let operator = self.previous().tty.clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, VeonError> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            let operator = self.previous().tty.clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, VeonError> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::EqualEqual, TokenType::NotEqual]) {
            let operator = self.previous().tty.clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, VeonError> {
        let mut expr = self.term()?;

        while self.matches(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous().tty.clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, VeonError> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().tty.clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, VeonError> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenType::Star, TokenType::Slash, TokenType::Modulo]) {
            let operator = self.previous().tty.clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, VeonError> {
        if self.matches(&[TokenType::Not, TokenType::Minus]) {
            let operator = self.previous().tty.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, VeonError> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenType::LeftParen]) {
                let mut arguments = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.matches(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    paren: TokenType::RightParen,
                    arguments,
                };
            } else if self.matches(&[TokenType::LeftBracket]) {
                let index = self.expression()?;
                self.consume(TokenType::RightBracket, "Expect ']' after index expression")?;
                expr = Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.matches(&[TokenType::Dot]) {
                self.consume(TokenType::Identifier, "Expect property name after '.'")?;
                let name = self.previous_identifier()?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, VeonError> {
        if self.matches(&[TokenType::Boolean]) {
            if let Value::Boolean(value) = self.previous().value.clone() {
                return Ok(Expr::Literal(Value::Boolean(value)));
            }
        }

        if self.matches(&[TokenType::Number]) {
            if let Value::Number(value) = self.previous().value.clone() {
                return Ok(Expr::Literal(Value::Number(value)));
            }
        }

        if self.matches(&[TokenType::String]) {
            if let Value::String(value) = self.previous().value.clone() {
                return Ok(Expr::Literal(Value::String(value)));
            }
        }

        if self.matches(&[TokenType::Null]) {
            return Ok(Expr::Literal(Value::Null));
        }

        if self.matches(&[TokenType::Identifier]) {
            if let Value::String(name) = self.previous().value.clone() {
                return Ok(Expr::Variable(name));
            }
        }

        if self.matches(&[TokenType::This]) {
            return Ok(Expr::This);
        }

        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.matches(&[TokenType::LeftBracket]) {
            let mut elements = Vec::new();
            if !self.check(TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.matches(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            self.consume(TokenType::RightBracket, "Expect ']' after array literal")?;
            return Ok(Expr::Array(elements));
        }

        Err(Self::error("Expected expression"))
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        for tty in types {
            if self.check(tty.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, tty: TokenType, message: &str) -> Result<(), VeonError> {
        if self.check(tty) {
            self.advance();
            return Ok(());
        }
        Err(Self::error(message))
    }

    fn check(&self, tty: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().tty == tty
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().tty == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn previous_identifier(&self) -> Result<String, VeonError> {
        if let Value::String(name) = self.previous().value.clone() {
            Ok(name)
        } else {
            Err(Self::error("Expected identifier"))
        }
    }

    fn error(message: &str) -> VeonError {
        VeonError::ParserError(ParserError {
            msg: message.to_string(),
            tty: ParserErrorType::InvalidExpression,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{scanner::Scanner, token::TokenType};

    use super::*;

    fn parse_source(source: &str) -> Vec<Stmt> {
        let mut scanner = Scanner::new(source.to_string());
        let tokens = scanner.tokenize().expect("tokenize");
        let mut parser = Parser::new(tokens);
        parser.parse().expect("parse")
    }

    #[test]
    fn parse_variable_declaration_and_expression() {
        let statements = parse_source("let x = 10; x + 2;");
        assert_eq!(statements.len(), 2);

        match &statements[0] {
            Stmt::Var { name, initializer } => {
                assert_eq!(name, "x");
                assert!(matches!(initializer, Some(_)));
            }
            _ => panic!("expected var statement"),
        }

        match &statements[1] {
            Stmt::Expression(Expr::Binary { operator, .. }) => {
                assert_eq!(operator, &TokenType::Plus);
            }
            _ => panic!("expected expression statement"),
        }
    }

    #[test]
    fn parse_array_and_index_expression() {
        let statements = parse_source("let items = [1, 2, 3]; items[1];");
        assert_eq!(statements.len(), 2);

        match &statements[0] {
            Stmt::Var { initializer, .. } => {
                if let Some(Expr::Array(elements)) = initializer {
                    assert_eq!(elements.len(), 3);
                } else {
                    panic!("expected array initializer");
                }
            }
            _ => panic!("expected var statement"),
        }

        match &statements[1] {
            Stmt::Expression(Expr::Index { .. }) => {}
            _ => panic!("expected index expression"),
        }
    }

    #[test]
    fn parse_function_and_class() {
        let statements = parse_source(
            "class Foo { fun greet(name) { return name; } } fun add(a, b) { return a + b; }",
        );
        assert_eq!(statements.len(), 2);
        matches!(statements[0], Stmt::Class { .. });
        matches!(statements[1], Stmt::Function { .. });
    }
}
