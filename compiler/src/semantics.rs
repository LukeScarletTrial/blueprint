use crate::ast::*;
use crate::lexer::{byte_to_location, SourceLocation, Span};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
    pub span: Span,
    pub location: SourceLocation,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Semantic error at line {}, column {}: {}",
            self.location.line, self.location.column, self.message
        )
    }
}

impl std::error::Error for SemanticError {}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub param_names: Vec<String>,
    pub param_types: Vec<Option<Type>>,
    pub return_type: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: HashMap<String, (Type, usize)>, // field_name -> (type, index)
    pub field_order: Vec<(String, Type)>,
    pub span: Span,
}

pub struct SemanticAnalyzer<'a> {
    source: &'a str,
    structs: HashMap<String, StructInfo>,
    functions: HashMap<String, FunctionSignature>,
    scopes: Vec<HashMap<String, Type>>,
    current_function: Option<String>,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut functions = HashMap::new();
        // Register built-in output function
        functions.insert(
            "output".to_string(),
            FunctionSignature {
                name: "output".to_string(),
                param_names: vec!["val".to_string()],
                param_types: vec![None], // accepts any printable type
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        Self {
            source,
            structs: HashMap::new(),
            functions,
            scopes: Vec::new(),
            current_function: None,
        }
    }

    fn error<T>(&self, message: impl Into<String>, span: Span) -> Result<T, SemanticError> {
        let location = byte_to_location(self.source, span.start);
        Err(SemanticError {
            message: message.into(),
            span,
            location,
        })
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_variable(&mut self, name: &str, ty: Type, span: Span) -> Result<(), SemanticError> {
        if let Some(current_scope) = self.scopes.last_mut() {
            if current_scope.contains_key(name) {
                return self.error(
                    format!("Variable '{}' already defined in this scope", name),
                    span,
                );
            }
            current_scope.insert(name.to_string(), ty);
            Ok(())
        } else {
            self.error("Internal error: no active scope", span)
        }
    }

    fn lookup_variable(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn analyze_program(&mut self, program: &Program) -> Result<(), SemanticError> {
        // Pass 1: Collect struct definitions
        for item in &program.items {
            if let Item::Struct(st) = item {
                if self.structs.contains_key(&st.name) {
                    return self.error(
                        format!("Duplicate struct definition '{}'", st.name),
                        st.span,
                    );
                }

                let mut field_map = HashMap::new();
                let mut field_order = Vec::new();

                for (idx, field) in st.fields.iter().enumerate() {
                    if field_map.contains_key(&field.name) {
                        return self.error(
                            format!("Duplicate field '{}' in struct '{}'", field.name, st.name),
                            field.span,
                        );
                    }
                    field_map.insert(field.name.clone(), (field.ty.clone(), idx));
                    field_order.push((field.name.clone(), field.ty.clone()));
                }

                self.structs.insert(
                    st.name.clone(),
                    StructInfo {
                        name: st.name.clone(),
                        fields: field_map,
                        field_order,
                        span: st.span,
                    },
                );
            }
        }

        // Pass 2: Collect function signatures
        for item in &program.items {
            if let Item::Func(func) = item {
                if self.functions.contains_key(&func.name) {
                    return self.error(
                        format!("Duplicate function definition '{}'", func.name),
                        func.span,
                    );
                }

                let param_names = func.params.iter().map(|p| p.name.clone()).collect();
                let param_types = func.params.iter().map(|p| p.ty.clone()).collect();

                self.functions.insert(
                    func.name.clone(),
                    FunctionSignature {
                        name: func.name.clone(),
                        param_names,
                        param_types,
                        return_type: func.return_ty.clone(),
                        span: func.span,
                    },
                );
            }
        }

        // Pass 3: Validate function bodies, statements, expressions
        for item in &program.items {
            match item {
                Item::Setup(_) => {}
                Item::Struct(_) => {}
                Item::Func(func) => {
                    self.analyze_func(func)?;
                }
                Item::Var(var) => {
                    let ty = self.analyze_expr(&var.expr)?;
                    if let Some(ref explicit) = var.explicit_ty {
                        if explicit != &ty {
                            return self.error(
                                format!(
                                    "Type mismatch for '{}': declared as {}, but initialized with {}",
                                    var.name, explicit, ty
                                ),
                                var.span,
                            );
                        }
                    }
                }
                Item::Stmt(stmt) => {
                    self.enter_scope();
                    self.analyze_stmt(stmt)?;
                    self.exit_scope();
                }
            }
        }

        Ok(())
    }

    fn analyze_func(&mut self, func: &FuncDef) -> Result<(), SemanticError> {
        self.current_function = Some(func.name.clone());
        self.enter_scope();

        for param in &func.params {
            let ty = param.ty.clone().unwrap_or(Type::Custom("Any".into()));
            self.define_variable(&param.name, ty, param.span)?;
        }

        for stmt in &func.body {
            self.analyze_stmt(stmt)?;
        }

        self.exit_scope();
        self.current_function = None;
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match stmt {
            Stmt::VarDef(var) => {
                let ty = self.analyze_expr(&var.expr)?;
                if let Some(ref explicit) = var.explicit_ty {
                    if explicit != &ty {
                        return self.error(
                            format!(
                                "Type mismatch for '{}': declared as {}, but got {}",
                                var.name, explicit, ty
                            ),
                            var.span,
                        );
                    }
                }
                self.define_variable(&var.name, ty, var.span)?;
            }
            Stmt::Expr(expr) => {
                self.analyze_expr(expr)?;
            }
            Stmt::Return(expr, _span) => {
                if let Some(ret_expr) = expr {
                    self.analyze_expr(ret_expr)?;
                }
            }
            Stmt::If(if_stmt) => {
                let cond_ty = self.analyze_expr(&if_stmt.condition)?;
                if cond_ty != Type::Bool && cond_ty != Type::Custom("Any".into()) {
                    return self.error(
                        format!("Condition must be a boolean, got {}", cond_ty),
                        if_stmt.condition.span,
                    );
                }

                self.enter_scope();
                for s in &if_stmt.then_branch {
                    self.analyze_stmt(s)?;
                }
                self.exit_scope();

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.enter_scope();
                    for s in else_branch {
                        self.analyze_stmt(s)?;
                    }
                    self.exit_scope();
                }
            }
        }
        Ok(())
    }

    pub fn analyze_expr(&mut self, spanned_expr: &SpannedExpr) -> Result<Type, SemanticError> {
        let span = spanned_expr.span;
        match &spanned_expr.expr {
            Expr::Int(_) => Ok(Type::Int),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::String(_) => Ok(Type::String),
            Expr::Ident(name) => {
                if let Some(ty) = self.lookup_variable(name) {
                    Ok(ty)
                } else if self.functions.contains_key(name) {
                    Ok(Type::Custom("Function".into()))
                } else {
                    self.error(format!("Undefined variable '{}'", name), span)
                }
            }
            Expr::UnaryOp(UnOp::Neg, inner) => {
                let ty = self.analyze_expr(inner)?;
                if ty != Type::Int && ty != Type::Custom("Any".into()) {
                    return self.error(format!("Cannot negate non-integer type {}", ty), span);
                }
                Ok(Type::Int)
            }
            Expr::UnaryOp(UnOp::Not, inner) => {
                let ty = self.analyze_expr(inner)?;
                if ty != Type::Bool && ty != Type::Custom("Any".into()) {
                    return self.error(
                        format!("Cannot apply 'not' to non-boolean type {}", ty),
                        span,
                    );
                }
                Ok(Type::Bool)
            }
            Expr::BinaryOp(lhs, op, rhs) => {
                let lhs_ty = self.analyze_expr(lhs)?;
                let rhs_ty = self.analyze_expr(rhs)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        if (lhs_ty == Type::Int || lhs_ty == Type::Custom("Any".into()))
                            && (rhs_ty == Type::Int || rhs_ty == Type::Custom("Any".into()))
                        {
                            Ok(Type::Int)
                        } else if *op == BinOp::Add
                            && (lhs_ty == Type::String || rhs_ty == Type::String)
                        {
                            // String concatenation
                            Ok(Type::String)
                        } else {
                            self.error(
                                format!(
                                    "Cannot apply operator {:?} between {} and {}",
                                    op, lhs_ty, rhs_ty
                                ),
                                span,
                            )
                        }
                    }
                    BinOp::Gt | BinOp::Lt | BinOp::Gte | BinOp::Lte => {
                        if (lhs_ty == Type::Int || lhs_ty == Type::Custom("Any".into()))
                            && (rhs_ty == Type::Int || rhs_ty == Type::Custom("Any".into()))
                        {
                            Ok(Type::Bool)
                        } else {
                            self.error(
                                format!(
                                    "Comparison {:?} requires integers, got {} and {}",
                                    op, lhs_ty, rhs_ty
                                ),
                                span,
                            )
                        }
                    }
                    BinOp::EqEq | BinOp::Neq => {
                        // Can check equality between matching types or Any
                        if lhs_ty == rhs_ty
                            || lhs_ty == Type::Custom("Any".into())
                            || rhs_ty == Type::Custom("Any".into())
                        {
                            Ok(Type::Bool)
                        } else {
                            self.error(
                                format!(
                                    "Cannot compare equality between different types {} and {}",
                                    lhs_ty, rhs_ty
                                ),
                                span,
                            )
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if (lhs_ty == Type::Bool || lhs_ty == Type::Custom("Any".into()))
                            && (rhs_ty == Type::Bool || rhs_ty == Type::Custom("Any".into()))
                        {
                            Ok(Type::Bool)
                        } else {
                            self.error(
                                format!(
                                    "Logical operators require booleans, got {} and {}",
                                    lhs_ty, rhs_ty
                                ),
                                span,
                            )
                        }
                    }
                }
            }
            Expr::Call(name, args) => {
                let sig = match self.functions.get(name) {
                    Some(s) => s.clone(),
                    None => {
                        return self.error(format!("Call to undefined function '{}'", name), span)
                    }
                };

                if sig.name != "output" && sig.param_names.len() != args.len() {
                    return self.error(
                        format!(
                            "Function '{}' expects {} arguments, got {}",
                            name,
                            sig.param_names.len(),
                            args.len()
                        ),
                        span,
                    );
                }

                for arg in args {
                    self.analyze_expr(arg)?;
                }

                Ok(sig.return_type.unwrap_or(Type::Custom("Any".into())))
            }
            Expr::StructInit(name, named_fields) => {
                let st_info = match self.structs.get(name) {
                    Some(s) => s.clone(),
                    None => return self.error(format!("Undefined struct '{}'", name), span),
                };

                for (field_name, arg) in named_fields.iter() {
                    // skip positional synthetic names like _0, _1
                    if field_name.starts_with('_') && field_name[1..].parse::<usize>().is_ok() {
                        self.analyze_expr(arg)?;
                        continue;
                    }
                    let arg_ty = self.analyze_expr(arg)?;
                    // look up the expected type by name
                    if let Some((_, expected_ty)) = st_info.field_order.iter().find(|(n, _)| n == field_name) {
                        if &arg_ty != expected_ty && arg_ty != Type::Custom("Any".into()) {
                            return self.error(
                                format!(
                                    "Field '{}' of struct '{}' expects {}, got {}",
                                    field_name, name, expected_ty, arg_ty
                                ),
                                arg.span,
                            );
                        }
                    } else {
                        return self.error(
                            format!("Struct '{}' has no field '{}'", name, field_name),
                            arg.span,
                        );
                    }
                }

                Ok(Type::Custom(name.clone()))
            }
            Expr::FieldAccess(target, field_name) => {
                let target_ty = self.analyze_expr(target)?;
                match target_ty {
                    Type::Custom(ref s_name) => {
                        if s_name == "Any" {
                            Ok(Type::Custom("Any".into()))
                        } else if let Some(st_info) = self.structs.get(s_name) {
                            if let Some((f_ty, _)) = st_info.fields.get(field_name) {
                                Ok(f_ty.clone())
                            } else {
                                self.error(
                                    format!(
                                        "Struct '{}' has no field named '{}'",
                                        s_name, field_name
                                    ),
                                    span,
                                )
                            }
                        } else {
                            self.error(format!("Unknown struct type '{}'", s_name), span)
                        }
                    }
                    _ => self.error(
                        format!(
                            "Cannot access field '{}' on non-struct type {}",
                            field_name, target_ty
                        ),
                        span,
                    ),
                }
            }
            Expr::MethodCall(target, _method, args) => {
                self.analyze_expr(target)?;
                for arg in args {
                    self.analyze_expr(arg)?;
                }
                // For MVP, allow method call or treat as dynamic
                Ok(Type::Custom("Any".into()))
            }
        }
    }
}

pub fn analyze(source: &str, program: &Program) -> Result<(), SemanticError> {
    let mut analyzer = SemanticAnalyzer::new(source);
    analyzer.analyze_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse_program;

    #[test]
    fn test_semantic_analysis_example() {
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
        let res = analyze(src, &prog);
        assert!(res.is_ok(), "Semantic error: {:?}", res.err());
    }

    #[test]
    fn test_undefined_variable() {
        let src = "func Foo(): [ def a = undefined_var ]";
        let tokens = tokenize(src).expect("tokenize");
        let prog = parse_program(src, tokens).expect("parse");
        let res = analyze(src, &prog);
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("Undefined variable"));
    }

    #[test]
    fn test_duplicate_struct() {
        let src = "defstruct Foo [ a: int ] defstruct Foo [ b: bool ]";
        let tokens = tokenize(src).expect("tokenize");
        let prog = parse_program(src, tokens).expect("parse");
        let res = analyze(src, &prog);
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("Duplicate struct"));
    }

    #[test]
    fn test_invalid_field_access() {
        let src = r#"
defstruct Foo [ a: int ]
func Bar(): [
    def f = Foo[10]
    def x = f.nonexistent
]
"#;
        let tokens = tokenize(src).expect("tokenize");
        let prog = parse_program(src, tokens).expect("parse");
        let res = analyze(src, &prog);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .message
            .contains("no field named 'nonexistent'"));
    }
}
