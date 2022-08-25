use strum::Display;

#[derive(thiserror::Error, Debug)]
pub enum VeonError {
    #[error("Error while parsing: {0}")]
    ParserError(#[from] ParserError),
    #[error("Error while scanning: {0}")]
    ScannerError(ScannerError),
    #[error("Error while evaluating: {0}")]
    EvaluationError(EvaluationError),
}

#[derive(Debug, thiserror::Error)]
#[error("[{tty}:{msg}]")]
pub struct ParserError {
    pub msg: String,
    pub tty: ParserErrorType,
}
#[derive(Debug, thiserror::Error)]
#[error("[{tty}:{msg}]")]
pub struct ScannerError {
    pub msg: String,
    pub tty: ScannerErrorType,
}

#[derive(Debug, thiserror::Error)]
#[error("[{tty}:{msg}]")]
pub struct EvaluationError {
    pub msg: String,
    pub tty: EvaluationErrorType,
}

#[derive(Debug, Display)]
pub enum ParserErrorType {
    InvalidExpression,
}

#[derive(Debug, Display)]

pub enum ScannerErrorType {
    InvalidToken,
}

#[derive(Debug, Display)]
pub enum EvaluationErrorType {
    DivideByZero,
    InvalidOperation,
    InvalidTypeOperation,
}
