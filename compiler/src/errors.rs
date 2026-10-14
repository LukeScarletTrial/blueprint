use thiserror::Error;

#[derive(Debug, Error)]
#[error("Compile error: {message}")]
pub struct CompileError {
    pub message: String,
}

impl CompileError {
    pub fn new(msg: &str) -> Self { Self { message: msg.to_string() } }
}
