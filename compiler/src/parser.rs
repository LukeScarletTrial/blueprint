use crate::ast::*;
use crate::lexer::{byte_to_location, SourceLocation, Span, SpannedToken, Token};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub location: SourceLocation,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.location.line, self.location.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Vec<SpannedToken>) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
        }
    }

    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.cursor)
    }

    fn peek_next(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.cursor + 1)
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn current_span(&self) -> Span {
        if let Some(tok) = self.peek() {
            tok.span
        } else if let Some(last) = self.tokens.last() {
            Span::new(last.span.end, last.span.end)
        } else {
            Span::new(0, 0)
        }
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn error<T>(&self, msg: impl Into<String>, span: Span) -> Result<T, ParseError> {
        let loc = byte_to_location(self.source, span.start);
        Err(ParseError {
            message: msg.into(),
            span,
            location: loc,
        })
    }

    fn mk_error(&self, msg: impl Into<String>, span: Span) -> ParseError {
        let loc = byte_to_location(self.source, span.start);
        ParseError {
            message: msg.into(),
            span,
            location: loc,
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<SpannedToken, ParseError> {
        let span = self.current_span();
        match self.peek() {
            Some(tok) if std::mem::discriminant(&tok.token) == std::mem::discriminant(expected) => {
                Ok(self.advance().unwrap())
            }
            Some(tok) => self.error(
                format!("Expected {:?}, found {:?}", expected, tok.token),
                span,
            ),
            None => self.error(format!("Expected {:?}, found end of file", expected), span),
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), ParseError> {
        let span = self.current_span();
        match self.peek() {
            Some(tok) => match &tok.token {
                Token::Ident(name) => {
                    let n = name.clone();
                    self.advance();
                    Ok((n, span))
                }
                _ => self.error(format!("Expected identifier, found {:?}", tok.token), span),
            },
            None => self.error("Expected identifier, found end of file", span),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let start_span = self.current_span();
        let mut annotations = Vec::new();
        let mut items = Vec::new();

        while !self.is_eof() {
            match self.peek().map(|t| &t.token) {
                Some(Token::AtBlueprint) => {
                    let ann = self.parse_annotation()?;
                    annotations.push(ann);
                }
                Some(Token::DefStruct) => {
                    let st = self.parse_struct_def()?;
                    items.push(Item::Struct(st));
                }
                Some(Token::Func) => {
                    let func = self.parse_func_def()?;
                    items.push(Item::Func(func));
                }
                Some(Token::Def) => {
                    let var = self.parse_var_def()?;
                    items.push(Item::Var(var));
                }
                Some(Token::Ident(name)) if name == "Blueprint" => {
                    // Check if followed by .Setup(...)
                    if let Some(next) = self.peek_next() {
                        if next.token == Token::Dot {
                            let setup = self.parse_setup_stmt()?;
                            items.push(Item::Setup(setup));
                            continue;
                        }
                    }
                    let stmt = self.parse_stmt()?;
                    items.push(Item::Stmt(stmt));
                }
                _ => {
                    let stmt = self.parse_stmt()?;
                    items.push(Item::Stmt(stmt));
                }
            }
        }

        let end_span = self.tokens.last().map(|t| t.span.end).unwrap_or(0);
        let span = Span::new(start_span.start, end_span);

        Ok(Program {
            annotations,
            items,
            span,
        })
    }

    fn parse_annotation(&mut self) -> Result<Annotation, ParseError> {
        let at_tok = self.expect(&Token::AtBlueprint)?;
        // expect import <Target>
        let (import_kw, _) = self.expect_ident()?;
        if import_kw != "import" {
            return self.error("Expected 'import' after '@blueprint'", at_tok.span);
        }
        let (mut target, mut last_span) = self.expect_ident()?;
        while let Some(tok) = self.peek() {
            if tok.token == Token::Slash {
                self.advance();
                let (next_ident, next_span) = self.expect_ident()?;
                target.push('/');
                target.push_str(&next_ident);
                last_span = next_span;
            } else {
                break;
            }
        }
        Ok(Annotation {
            name: "blueprint".into(),
            target,
            span: Span::new(at_tok.span.start, last_span.end),
        })
    }

    fn parse_setup_stmt(&mut self) -> Result<SetupStmt, ParseError> {
        let (name, start_span) = self.expect_ident()?;
        if name != "Blueprint" {
            return self.error("Expected 'Blueprint'", start_span);
        }
        self.expect(&Token::Dot)?;
        let (method, _) = self.expect_ident()?;
        if method != "Setup" {
            return self.error("Expected 'Setup'", start_span);
        }
        self.expect(&Token::LParen)?;
        let app_name = match self.advance() {
            Some(tok) => match tok.token {
                Token::String(s) => s,
                _ => return self.error("Expected string for appName in Blueprint.Setup", tok.span),
            },
            None => return self.error("Expected appName string literal", start_span),
        };
        self.expect(&Token::Comma)?;
        let use_deps = match self.advance() {
            Some(tok) => match tok.token {
                Token::True => true,
                Token::False => false,
                _ => {
                    return self.error("Expected boolean for useDeps in Blueprint.Setup", tok.span)
                }
            },
            None => return self.error("Expected boolean literal in Blueprint.Setup", start_span),
        };
        let r_paren = self.expect(&Token::RParen)?;

        Ok(SetupStmt {
            app_name,
            use_deps,
            span: Span::new(start_span.start, r_paren.span.end),
        })
    }

    fn parse_struct_def(&mut self) -> Result<StructDef, ParseError> {
        let def_tok = self.expect(&Token::DefStruct)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::LBracket)?;

        let mut fields = Vec::new();
        while let Some(tok) = self.peek() {
            if tok.token == Token::RBracket {
                break;
            }
            let (f_name, f_span) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            let end_span = self.current_span();
            fields.push(StructField {
                name: f_name,
                ty,
                span: Span::new(f_span.start, end_span.start),
            });

            if let Some(tok) = self.peek() {
                if tok.token == Token::Comma {
                    self.advance();
                } else if tok.token != Token::RBracket {
                    return self.error("Expected ',' or ']' in struct definition", tok.span);
                }
            }
        }

        let r_bracket = self.expect(&Token::RBracket)?;
        Ok(StructDef {
            name,
            fields,
            span: Span::new(def_tok.span.start, r_bracket.span.end),
        })
    }

    fn parse_func_def(&mut self) -> Result<FuncDef, ParseError> {
        let func_tok = self.expect(&Token::Func)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::LParen)?;

        let mut params = Vec::new();
        while let Some(tok) = self.peek() {
            if tok.token == Token::RParen {
                break;
            }
            let (p_name, p_span) = self.expect_ident()?;
            let ty = if let Some(next) = self.peek() {
                if next.token == Token::Colon {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                }
            } else {
                None
            };
            let end = self.current_span().start;
            params.push(Param {
                name: p_name,
                ty,
                span: Span::new(p_span.start, end),
            });

            if let Some(next) = self.peek() {
                if next.token == Token::Comma {
                    self.advance();
                } else if next.token != Token::RParen {
                    return self.error("Expected ',' or ')' in parameter list", next.span);
                }
            }
        }
        self.expect(&Token::RParen)?;

        // Optional return type annotation: func Print(s): [ ... ]  OR  func IsAdult(p): bool [ ... ]
        let mut return_ty = None;
        if let Some(tok) = self.peek() {
            if tok.token == Token::Colon {
                self.advance(); // consume ':'
                // If next token is NOT '[', parse return type
                if let Some(next) = self.peek() {
                    if next.token != Token::LBracket {
                        return_ty = Some(self.parse_type()?);
                    }
                }
            }
        }

        let (body, body_span) = self.parse_block()?;
        Ok(FuncDef {
            name,
            params,
            return_ty,
            body,
            span: Span::new(func_tok.span.start, body_span.end),
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let (name, _span) = self.expect_ident()?;
        match name.as_str() {
            "int" => Ok(Type::Int),
            "bool" => Ok(Type::Bool),
            "string" | "str" => Ok(Type::String),
            "void" => Ok(Type::Void),
            _ => Ok(Type::Custom(name)),
        }
    }

    fn parse_block(&mut self) -> Result<(Vec<Stmt>, Span), ParseError> {
        let l_tok = self.expect(&Token::LBracket)?;
        let mut stmts = Vec::new();
        while let Some(tok) = self.peek() {
            if tok.token == Token::RBracket {
                break;
            }
            let s = self.parse_stmt()?;
            stmts.push(s);
        }
        let r_tok = self.expect(&Token::RBracket)?;
        Ok((stmts, Span::new(l_tok.span.start, r_tok.span.end)))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let tok = match self.peek() {
            Some(t) => t,
            None => return self.error("Expected statement, found EOF", self.current_span()),
        };

        match tok.token {
            Token::Def => {
                let var = self.parse_var_def()?;
                Ok(Stmt::VarDef(var))
            }
            Token::Return => {
                let ret_tok = self.advance().unwrap();
                // Check if return has an expression
                let expr = match self.peek() {
                    Some(t) if t.token != Token::RBracket => Some(self.parse_expr()?),
                    _ => None,
                };
                let end = expr
                    .as_ref()
                    .map(|e| e.span.end)
                    .unwrap_or(ret_tok.span.end);
                Ok(Stmt::Return(expr, Span::new(ret_tok.span.start, end)))
            }
            Token::If => {
                let if_stmt = self.parse_if_stmt()?;
                Ok(Stmt::If(if_stmt))
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_var_def(&mut self) -> Result<VarDef, ParseError> {
        let def_tok = self.expect(&Token::Def)?;
        let (name, _) = self.expect_ident()?;
        let explicit_ty = if let Some(tok) = self.peek() {
            if tok.token == Token::Colon {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            }
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let expr = self.parse_expr()?;
        let span = Span::new(def_tok.span.start, expr.span.end);
        Ok(VarDef {
            name,
            explicit_ty,
            expr,
            span,
        })
    }

    fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> {
        let if_tok = self.expect(&Token::If)?;
        let cond = self.parse_expr_without_block()?;
        let (then_branch, then_span) = self.parse_block()?;
        let mut else_branch = None;
        let mut end = then_span.end;

        if let Some(tok) = self.peek() {
            if tok.token == Token::Else {
                self.advance();
                // Can be `else [` or `else if`
                if let Some(tok2) = self.peek() {
                    if tok2.token == Token::If {
                        let inner_if = self.parse_if_stmt()?;
                        end = inner_if.span.end;
                        else_branch = Some(vec![Stmt::If(inner_if)]);
                    } else {
                        let (eb, e_span) = self.parse_block()?;
                        end = e_span.end;
                        else_branch = Some(eb);
                    }
                }
            }
        }

        Ok(IfStmt {
            condition: cond,
            then_branch,
            else_branch,
            span: Span::new(if_tok.span.start, end),
        })
    }

    // Expression parsing with operator precedence (Pratt parsing)
    pub fn parse_expr(&mut self) -> Result<SpannedExpr, ParseError> {
        self.parse_expr_bp(0, false)
    }

    fn parse_expr_without_block(&mut self) -> Result<SpannedExpr, ParseError> {
        self.parse_expr_bp(0, true)
    }

    fn parse_expr_bp(
        &mut self,
        min_bp: u8,
        disallow_trailing_block: bool,
    ) -> Result<SpannedExpr, ParseError> {
        let mut lhs = self.parse_primary(disallow_trailing_block)?;

        loop {
            // Check for postfix operators: .field, .method(args), [struct_init] or function call (if lhs was ident)
            if let Some(tok) = self.peek() {
                if tok.token == Token::Dot {
                    self.advance();
                    let (member, m_span) = self.expect_ident()?;
                    if let Some(next) = self.peek() {
                        if next.token == Token::LParen {
                            self.advance();
                            let mut args = Vec::new();
                            while let Some(a_tok) = self.peek() {
                                if a_tok.token == Token::RParen {
                                    break;
                                }
                                args.push(self.parse_expr()?);
                                if let Some(c_tok) = self.peek() {
                                    if c_tok.token == Token::Comma {
                                        self.advance();
                                    } else if c_tok.token != Token::RParen {
                                        return self.error("Expected ',' or ')'", c_tok.span);
                                    }
                                }
                            }
                            let r_tok = self.expect(&Token::RParen)?;
                            lhs = SpannedExpr {
                                span: Span::new(lhs.span.start, r_tok.span.end),
                                expr: Expr::MethodCall(Box::new(lhs), member, args),
                            };
                            continue;
                        }
                    }
                    lhs = SpannedExpr {
                        span: Span::new(lhs.span.start, m_span.end),
                        expr: Expr::FieldAccess(Box::new(lhs), member),
                    };
                    continue;
                }
            }

            // Binary operators
            let (op, l_bp, r_bp) = match self.peek().map(|t| &t.token) {
                Some(Token::Plus) => (BinOp::Add, 5, 6),
                Some(Token::Minus) => (BinOp::Sub, 5, 6),
                Some(Token::Star) => (BinOp::Mul, 7, 8),
                Some(Token::Slash) => (BinOp::Div, 7, 8),
                Some(Token::EqEq) => (BinOp::EqEq, 3, 4),
                Some(Token::Neq) => (BinOp::Neq, 3, 4),
                Some(Token::Gt) => (BinOp::Gt, 3, 4),
                Some(Token::Lt) => (BinOp::Lt, 3, 4),
                Some(Token::Gte) => (BinOp::Gte, 3, 4),
                Some(Token::Lte) => (BinOp::Lte, 3, 4),
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }

            self.advance(); // consume op
            let rhs = self.parse_expr_bp(r_bp, disallow_trailing_block)?;
            let span = Span::new(lhs.span.start, rhs.span.end);
            lhs = SpannedExpr {
                span,
                expr: Expr::BinaryOp(Box::new(lhs), op, Box::new(rhs)),
            };
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self, disallow_bracket: bool) -> Result<SpannedExpr, ParseError> {
        let tok = match self.peek() {
            Some(t) => t.clone(),
            None => return self.error("Expected expression, found EOF", self.current_span()),
        };

        match tok.token {
            Token::Int(n) => {
                self.advance();
                Ok(SpannedExpr {
                    expr: Expr::Int(n),
                    span: tok.span,
                })
            }
            Token::String(s) => {
                self.advance();
                Ok(SpannedExpr {
                    expr: Expr::String(s),
                    span: tok.span,
                })
            }
            Token::True => {
                self.advance();
                Ok(SpannedExpr {
                    expr: Expr::Bool(true),
                    span: tok.span,
                })
            }
            Token::False => {
                self.advance();
                Ok(SpannedExpr {
                    expr: Expr::Bool(false),
                    span: tok.span,
                })
            }
            Token::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                let r_paren = self.expect(&Token::RParen)?;
                Ok(SpannedExpr {
                    expr: inner.expr,
                    span: Span::new(tok.span.start, r_paren.span.end),
                })
            }
            Token::Minus => {
                self.advance();
                let inner = self.parse_expr_bp(9, disallow_bracket)?;
                let span = Span::new(tok.span.start, inner.span.end);
                Ok(SpannedExpr {
                    expr: Expr::UnaryOp(UnOp::Neg, Box::new(inner)),
                    span,
                })
            }
            Token::Ident(name) => {
                self.advance();
                // Check if followed by ( -> function call
                if let Some(next) = self.peek() {
                    if next.token == Token::LParen {
                        self.advance();
                        let mut args = Vec::new();
                        while let Some(a_tok) = self.peek() {
                            if a_tok.token == Token::RParen {
                                break;
                            }
                            args.push(self.parse_expr()?);
                            if let Some(c_tok) = self.peek() {
                                if c_tok.token == Token::Comma {
                                    self.advance();
                                } else if c_tok.token != Token::RParen {
                                    return self.error("Expected ',' or ')'", c_tok.span);
                                }
                            }
                        }
                        let r_tok = self.expect(&Token::RParen)?;
                        return Ok(SpannedExpr {
                            expr: Expr::Call(name, args),
                            span: Span::new(tok.span.start, r_tok.span.end),
                        });
                    }
                }

                // Check if followed by { -> Struct initialization Person { name: "Alex", age: 17 }
                if let Some(next) = self.peek() {
                    if next.token == Token::LBrace {
                        self.advance(); // consume {
                        let mut fields = Vec::new();
                        while let Some(f_tok) = self.peek() {
                            if f_tok.token == Token::RBrace {
                                break;
                            }
                            // expect: field_name: expr
                            let field_name_tok = self.advance().ok_or_else(|| {
                                self.mk_error("Expected field name in struct initializer", tok.span)
                            })?;
                            let field_name = match &field_name_tok.token {
                                Token::Ident(n) => n.clone(),
                                _ => return self.error(
                                    format!("Expected field name, got {:?}", field_name_tok.token),
                                    field_name_tok.span,
                                ),
                            };
                            self.expect(&Token::Colon)?;
                            let val = self.parse_expr()?;
                            fields.push((field_name, val));
                            if let Some(c_tok) = self.peek() {
                                if c_tok.token == Token::Comma {
                                    self.advance();
                                } else if c_tok.token != Token::RBrace {
                                    return self.error("Expected ',' or '}' in struct initializer", c_tok.span);
                                }
                            }
                        }
                        let r_tok = self.expect(&Token::RBrace)?;
                        return Ok(SpannedExpr {
                            expr: Expr::StructInit(name, fields),
                            span: Span::new(tok.span.start, r_tok.span.end),
                        });
                    }
                }

                // Check if followed by [ -> positional struct init Name[val, val]
                if !disallow_bracket {
                    if let Some(next) = self.peek() {
                        if next.token == Token::LBracket {
                            self.advance();
                            let mut fields = Vec::new();
                            let mut idx = 0usize;
                            while let Some(a_tok) = self.peek() {
                                if a_tok.token == Token::RBracket {
                                    break;
                                }
                                let val = self.parse_expr()?;
                                fields.push((format!("_{}", idx), val));
                                idx += 1;
                                if let Some(c_tok) = self.peek() {
                                    if c_tok.token == Token::Comma {
                                        self.advance();
                                    } else if c_tok.token != Token::RBracket {
                                        return self.error("Expected ',' or ']'", c_tok.span);
                                    }
                                }
                            }
                            let r_tok = self.expect(&Token::RBracket)?;
                            return Ok(SpannedExpr {
                                expr: Expr::StructInit(name, fields),
                                span: Span::new(tok.span.start, r_tok.span.end),
                            });
                        }
                    }
                }

                Ok(SpannedExpr {
                    expr: Expr::Ident(name),
                    span: tok.span,
                })
            }
            _ => self.error(format!("Unexpected token {:?}", tok.token), tok.span),
        }
    }
}

pub fn parse_program(source: &str, tokens: Vec<SpannedToken>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(source, tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_example_program() {
        let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Blueprint App", true)

defstruct Profile [
    name: string,
    age: int,
    is_adult: bool
]

func Print(statement): [
    output(statement)
]

func CreateProfile(name, age): [
    def isAdult = age >= 18
    def profile = Profile[name, age, isAdult]
    return profile
]

func Main(): [
    def userName = "Yasser"
    def userAge = 13
    def myProfile = CreateProfile(userName, userAge)

    if myProfile.is_adult [
        Print("User is an adult.")
    ] else [
        Print("User is a minor.")
    ]
]
"#;
        let tokens = tokenize(src).expect("tokenize");
        let prog = parse_program(src, tokens).expect("parse");
        assert_eq!(prog.annotations.len(), 1);
        assert_eq!(prog.annotations[0].target, "Blueprint");
        assert_eq!(prog.items.len(), 5); // Setup, Profile, Print, CreateProfile, Main
    }

    #[test]
    fn test_parse_precedence() {
        let src = "def a = 1 + 2 * 3";
        let tokens = tokenize(src).expect("tokenize");
        let prog = parse_program(src, tokens).expect("parse");
        match &prog.items[0] {
            Item::Var(v) => match &v.expr.expr {
                Expr::BinaryOp(l, BinOp::Add, r) => {
                    assert_eq!(l.expr, Expr::Int(1));
                    match &r.expr {
                        Expr::BinaryOp(rl, BinOp::Mul, rr) => {
                            assert_eq!(rl.expr, Expr::Int(2));
                            assert_eq!(rr.expr, Expr::Int(3));
                        }
                        _ => panic!("Expected multiplication on RHS"),
                    }
                }
                _ => panic!("Expected addition"),
            },
            _ => panic!("Expected var def"),
        }
    }

    #[test]
    fn test_invalid_syntax() {
        let src = "def = 10";
        let tokens = tokenize(src).expect("tokenize");
        let res = parse_program(src, tokens);
        assert!(res.is_err());
    }
}
