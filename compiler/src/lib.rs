pub mod ast;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod errors;

pub use crate::parser::parse_program;
pub use crate::interpreter::{Interpreter, Runtime};
pub use crate::errors::CompileError;
