pub mod ast;
pub mod codegen;
pub mod errors;
pub mod interpreter;
pub mod lexer;
pub mod modules;
pub mod parser;
pub mod semantics;

pub use crate::ast::Program;
pub use crate::codegen::CodeGen;
pub use crate::errors::CompileError;
pub use crate::interpreter::{Interpreter, Runtime, RuntimeError, Value};
pub use crate::lexer::{tokenize, LexError, SpannedToken, Token};
pub use crate::modules::resolve_modules;
pub use crate::parser::{parse_program, ParseError};
pub use crate::semantics::{analyze, SemanticAnalyzer, SemanticError};

/// Compile pipeline: tokenizes, parses, resolves module imports, and semantically analyzes the source.
pub fn compile_with_base_dir(source: &str, base_dir: &std::path::Path) -> Result<Program, CompileError> {
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let tokens = tokenize(source)?;
    let mut program = parse_program(source, tokens)?;
    resolve_modules(base_dir, &mut program)?;
    analyze(source, &program)?;
    Ok(program)
}

/// Compile pipeline defaulting to current directory for module resolution.
pub fn compile(source: &str) -> Result<Program, CompileError> {
    compile_with_base_dir(source, std::path::Path::new("."))
}

/// Run pipeline with a base directory: compiles the source and executes Main(). Returns captured runtime output.
pub fn run_with_base_dir(
    source: &str,
    base_dir: &std::path::Path,
) -> Result<(Value, Vec<String>), CompileError> {
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let program = compile_with_base_dir(source, base_dir)?;
    let mut interpreter = Interpreter::new();
    interpreter.load_program(source, &program)?;
    let val = interpreter.run_main()?;
    Ok((val, interpreter.runtime.output))
}

/// Run pipeline: compiles the source and executes Main(). Returns captured runtime output.
pub fn run(source: &str) -> Result<(Value, Vec<String>), CompileError> {
    run_with_base_dir(source, std::path::Path::new("."))
}

/// Build pipeline with a base directory: compiles source and emits a native standalone executable.
pub fn build_executable_with_base_dir(
    source: &str,
    output_path: &std::path::Path,
    base_dir: &std::path::Path,
) -> Result<std::path::PathBuf, CompileError> {
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let program = compile_with_base_dir(source, base_dir)?;
    CodeGen::compile_to_executable(&program, source, output_path)
}

/// Build pipeline: compiles source and emits a native standalone executable.
pub fn build_executable(
    source: &str,
    output_path: &std::path::Path,
) -> Result<std::path::PathBuf, CompileError> {
    build_executable_with_base_dir(source, output_path, std::path::Path::new("."))
}

