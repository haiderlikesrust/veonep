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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tty: TokenType,
    pub value: Value,
    pub line: usize,
}
