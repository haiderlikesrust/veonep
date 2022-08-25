#[derive(Debug, Clone)]
pub enum TokenType {
    Fun,
    Let,
    Boolean,
    Number,
    String,
    // Array,
    // Object,
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
    Null,
}
#[derive(Debug, Clone)]

pub enum Value {
    String(String),
    Number(isize),
    Boolean(bool),
    None,
}
#[derive(Debug, Clone)]

pub struct Token {
    pub tty: TokenType,
    pub value: Value,
    pub line: usize,
}
// def, fn, fun, func, function, define,
// define, let, var

// 1 ?? "" = "1"
// "10 " ?? "10" ?? " HEY" "10 10 HEY"
