use logos::Logos;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(r: std::ops::Range<usize>) -> Self {
        Self {
            start: r.start,
            end: r.end,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
    pub text: String,
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\r\n]*")]
pub enum Token {
    // Keywords
    #[token("def")]
    Def,
    #[token("func")]
    Func,
    #[token("defstruct")]
    DefStruct,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // Annotation
    #[token("@blueprint")]
    AtBlueprint,

    // Identifiers & literals
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
    String(String),

    // Symbols
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Neq,
    #[token(">=")]
    Gte,
    #[token("<=")]
    Lte,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token(".")]
    Dot,
}

fn parse_string(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    if slice.len() < 2 {
        return None;
    }
    let inner = &slice[1..slice.len() - 1];
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => return None,
            }
        } else {
            result.push(c);
        }
    }
    Some(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
    pub location: SourceLocation,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer error at line {}, column {}: {}",
            self.location.line, self.location.column, self.message
        )
    }
}

impl std::error::Error for LexError {}

pub fn byte_to_location(source: &str, byte_offset: usize) -> SourceLocation {
    let mut line = 1;
    let mut column = 1;
    for (i, c) in source.char_indices() {
        if i >= byte_offset {
            break;
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    SourceLocation { line, column }
}

pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    // Strip UTF-8 BOM if present (written by Windows PowerShell Out-File or Notepad)
    let clean_source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let mut lexer = Token::lexer(clean_source);
    let mut tokens = Vec::new();

    while let Some(token_res) = lexer.next() {
        let span = Span::from(lexer.span());
        let text = lexer.slice().to_string();

        match token_res {
            Ok(token) => {
                tokens.push(SpannedToken { token, span, text });
            }
            Err(_) => {
                // Determine whether it's an unclosed string or an invalid character
                let loc = byte_to_location(source, span.start);
                let message = if text.starts_with('"') {
                    "Unterminated string literal".to_string()
                } else {
                    format!("Unexpected character '{}'", text)
                };
                return Err(LexError {
                    message,
                    span,
                    location: loc,
                });
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords_and_identifiers() {
        let src = "def func defstruct return if else true false Main CreateProfile";
        let tokens = tokenize(src).expect("Failed to tokenize");
        assert_eq!(tokens[0].token, Token::Def);
        assert_eq!(tokens[1].token, Token::Func);
        assert_eq!(tokens[2].token, Token::DefStruct);
        assert_eq!(tokens[3].token, Token::Return);
        assert_eq!(tokens[4].token, Token::If);
        assert_eq!(tokens[5].token, Token::Else);
        assert_eq!(tokens[6].token, Token::True);
        assert_eq!(tokens[7].token, Token::False);
        assert_eq!(tokens[8].token, Token::Ident("Main".into()));
        assert_eq!(tokens[9].token, Token::Ident("CreateProfile".into()));
    }

    #[test]
    fn test_escaped_strings() {
        let src = r#""Hello\nWorld\"!""#;
        let tokens = tokenize(src).expect("Failed to tokenize");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::String("Hello\nWorld\"!".into()));
    }

    #[test]
    fn test_operators() {
        let src = "== != >= <= > < = + - * / . : , [ ] ( )";
        let tokens = tokenize(src).expect("Failed to tokenize");
        let kinds: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
        assert_eq!(
            kinds,
            vec![
                Token::EqEq,
                Token::Neq,
                Token::Gte,
                Token::Lte,
                Token::Gt,
                Token::Lt,
                Token::Eq,
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Dot,
                Token::Colon,
                Token::Comma,
                Token::LBracket,
                Token::RBracket,
                Token::LParen,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_source_locations() {
        let src = "def a = 10\ndef b = 20";
        let tokens = tokenize(src).expect("tokenize");
        let loc = byte_to_location(src, tokens[4].span.start); // def b
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 1);
    }

    #[test]
    fn test_invalid_character() {
        let src = "def @foo = 1";
        // '@' followed by 'foo' is not @blueprint
        let res = tokenize(src);
        assert!(res.is_err());
    }

    #[test]
    fn test_unterminated_string() {
        let src = "def s = \"unclosed string";
        let res = tokenize(src);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.message.contains("Unterminated string"));
    }

    #[test]
    fn test_comments() {
        let src = "// this is a comment\ndef x = 42 // trailing";
        let tokens = tokenize(src).expect("tokenize");
        assert_eq!(tokens.len(), 4); // def, x, =, 42
    }
}
