use crate::interpreter::RuntimeError;
use crate::lexer::{LexError, SourceLocation, Span};
use crate::parser::ParseError;
use crate::semantics::SemanticError;
use thiserror::Error;

/// Format diagnostic with rustc-style source snippet and `^^^` indicator
pub fn format_source_diagnostic(
    source: &str,
    location: SourceLocation,
    span: Span,
    category: &str,
    message: &str,
) -> String {
    let mut out = format!(
        "{} at line {}, column {}: {}\n",
        category, location.line, location.column, message
    );

    let lines: Vec<&str> = source.lines().collect();
    if location.line > 0 && location.line <= lines.len() {
        let line_idx = location.line - 1;
        let line_content = lines[line_idx];
        let line_num_str = format!("{}", location.line);
        let pad = " ".repeat(line_num_str.len());

        out.push_str(&format!(" {} |\n", pad));
        out.push_str(&format!(" {} | {}\n", line_num_str, line_content));

        let col = if location.column > 0 {
            location.column - 1
        } else {
            0
        };
        let underline_len = if span.end > span.start {
            (span.end - span.start).min(line_content.len().saturating_sub(col)).max(1)
        } else {
            1
        };

        let pointer_pad = " ".repeat(col);
        let pointer = "^".repeat(underline_len);
        out.push_str(&format!(" {} | {}{}\n", pad, pointer_pad, pointer));
    }

    out
}

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

    pub fn to_diagnostic_string(&self, source: &str) -> String {
        match self {
            CompileError::Lex(e) => format_source_diagnostic(
                source,
                e.location,
                e.span,
                "Lexer error",
                &e.message,
            ),
            CompileError::Parse(e) => format_source_diagnostic(
                source,
                e.location,
                e.span,
                "Parse error",
                &e.message,
            ),
            CompileError::Semantic(e) => format_source_diagnostic(
                source,
                e.location,
                e.span,
                "Semantic error",
                &e.message,
            ),
            CompileError::Runtime(e) => format_source_diagnostic(
                source,
                e.location,
                e.span,
                "Runtime error",
                &e.message,
            ),
            CompileError::General { message } => format!("Compile error: {}\n", message),
        }
    }
}
