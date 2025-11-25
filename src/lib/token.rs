#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Dot,
    // keywords / identifiers
    Fun,
    Let,
    Class,
    This,
    If,
    Else,
    While,
    For,
    Return,
    And,
    Or,
    Identifier,
    Boolean,
    Number,
    String,
    Null,
    // operators
    Plus,
    Minus,
    Star,
    Slash,
    QuestionMark,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    EqualEqual,
    Modulo,
    Not,
    NotEqual,
    Eof,
}

use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(isize),
    Boolean(bool),
    Array(Vec<Value>),
    Function(std::rc::Rc<crate::interpreter::VeonFunction>),
    Class(std::rc::Rc<crate::interpreter::VeonClass>),
    Instance(std::rc::Rc<std::cell::RefCell<crate::interpreter::VeonInstance>>),
    Null,
    None,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Null, Value::Null) | (Value::None, Value::None) => true,
            (Value::Function(a), Value::Function(b)) => std::rc::Rc::ptr_eq(a, b),
            (Value::Class(a), Value::Class(b)) => std::rc::Rc::ptr_eq(a, b),
            (Value::Instance(a), Value::Instance(b)) => std::rc::Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Array(values) => {
                let parts: Vec<String> = values.iter().map(|v| format!("{v}")).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            Value::Function(_) => write!(f, "<fn>"),
            Value::Class(class) => write!(f, "<class {}>", class.name),
            Value::Instance(instance) => write!(f, "<{} instance>", instance.borrow().class.name),
            Value::Null | Value::None => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tty: TokenType,
    pub value: Value,
    pub line: usize,
}
