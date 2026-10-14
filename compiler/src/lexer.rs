use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
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

    // Annotation
    #[token("@blueprint")]
    AtBlueprint,

    // Identifiers & literals
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Int(i64),

    #[regex("\"([^\\\"]|\\.)*\"", parse_string)]
    String(String),

    #[token("true")]
    True,
    #[token("false")]
    False,

    // Symbols
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("=")]
    Eq,
    #[token(">=")]
    Gte,
    #[token("<=")]
    Lte,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Neq,
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

    #[token(":" )]
    Annotation,

    // skip whitespace
    #[regex("[ \t\n\r]+", logos::skip)]
    #[error]
    Error,
}

fn parse_string(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    // strip quotes and unescape simple escapes
    let inner = &slice[1..slice.len()-1];
    let unescaped = inner.replace("\\\"", "\"").replace("\\n", "\n");
    Some(unescaped)
}
