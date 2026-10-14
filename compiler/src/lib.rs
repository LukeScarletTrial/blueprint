pub mod ast;
pub mod codegen;
pub mod errors;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod semantics;

pub use crate::ast::Program;
pub use crate::codegen::CodeGen;
pub use crate::errors::CompileError;
pub use crate::interpreter::{Interpreter, Runtime, RuntimeError, Value};
pub use crate::lexer::{tokenize, LexError, SpannedToken, Token};
pub use crate::parser::{parse_program, ParseError};
pub use crate::semantics::{analyze, SemanticAnalyzer, SemanticError};

/// Compile pipeline: tokenizes, parses, and semantically analyzes the source.
pub fn compile(source: &str) -> Result<Program, CompileError> {
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let tokens = tokenize(source)?;
    let program = parse_program(source, tokens)?;
    analyze(source, &program)?;
    Ok(program)
}

/// Run pipeline: compiles the source and executes Main(). Returns captured runtime output.
pub fn run(source: &str) -> Result<(Value, Vec<String>), CompileError> {
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let program = compile(source)?;
    let mut interpreter = Interpreter::new();
    interpreter.load_program(source, &program)?;
    let val = interpreter.run_main()?;
    Ok((val, interpreter.runtime.output))
}

/// Build pipeline: compiles source and emits a native standalone executable.
pub fn build_executable(
    source: &str,
    output_path: &std::path::Path,
) -> Result<std::path::PathBuf, CompileError> {
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let program = compile(source)?;
    CodeGen::compile_to_executable(&program, source, output_path)
}

