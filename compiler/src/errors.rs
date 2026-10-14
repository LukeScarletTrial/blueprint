use crate::interpreter::RuntimeError;
use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::semantics::SemanticError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("{0}")]
    Lex(#[from] LexError),

    #[error("{0}")]
    Parse(#[from] ParseError),

    #[error("{0}")]
    Semantic(#[from] SemanticError),

    #[error("{0}")]
    Runtime(#[from] RuntimeError),

    #[error("Compile error: {message}")]
    General { message: String },
}

impl CompileError {
    pub fn new(msg: &str) -> Self {
        CompileError::General {
            message: msg.to_string(),
        }
    }
}
