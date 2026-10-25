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
    imported_modules: std::collections::HashSet<String>,
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

        functions.insert(
            "input".to_string(),
            FunctionSignature {
                name: "input".to_string(),
                param_names: vec!["prompt".to_string()],
                param_types: vec![Some(Type::String)],
                return_type: Some(Type::String),
                span: Span::new(0, 0),
            },
        );

        functions.insert(
            "str_len".to_string(),
            FunctionSignature {
                name: "str_len".to_string(),
                param_names: vec!["s".to_string()],
                param_types: vec![Some(Type::String)],
                return_type: Some(Type::Int),
                span: Span::new(0, 0),
            },
        );

        functions.insert(
            "math_abs".to_string(),
            FunctionSignature {
                name: "math_abs".to_string(),
                param_names: vec!["n".to_string()],
                param_types: vec![Some(Type::Int)],
                return_type: Some(Type::Int),
                span: Span::new(0, 0),
            },
        );

        functions.insert(
            "math_pow".to_string(),
            FunctionSignature {
                name: "math_pow".to_string(),
                param_names: vec!["base".to_string(), "exp".to_string()],
                param_types: vec![Some(Type::Int), Some(Type::Int)],
                return_type: Some(Type::Int),
                span: Span::new(0, 0),
            },
        );

        functions.insert(
            "math_min".to_string(),
            FunctionSignature {
                name: "math_min".to_string(),
                param_names: vec!["a".to_string(), "b".to_string()],
                param_types: vec![Some(Type::Int), Some(Type::Int)],
                return_type: Some(Type::Int),
                span: Span::new(0, 0),
            },
        );

        functions.insert(
            "math_max".to_string(),
            FunctionSignature {
                name: "math_max".to_string(),
                param_names: vec!["a".to_string(), "b".to_string()],
                param_types: vec![Some(Type::Int), Some(Type::Int)],
                return_type: Some(Type::Int),
                span: Span::new(0, 0),
            },
        );

        // =====================================================================
        // Phase 4: Extended Standard Library
        // =====================================================================

        // math_sqrt / math_floor / math_ceil / math_round
        for name in &["math_sqrt", "math_floor", "math_ceil", "math_round"] {
            functions.insert(
                name.to_string(),
                FunctionSignature {
                    name: name.to_string(),
                    param_names: vec!["x".to_string()],
                    param_types: vec![None], // accepts numeric
                    return_type: Some(Type::F64),
                    span: Span::new(0, 0),
                },
            );
        }

        // file_read(path: string) -> string
        functions.insert("file_read".to_string(), FunctionSignature {
            name: "file_read".to_string(),
            param_names: vec!["path".to_string()],
            param_types: vec![Some(Type::String)],
            return_type: Some(Type::String),
            span: Span::new(0, 0),
        });

        // file_write(path: string, data: string) -> bool
        functions.insert("file_write".to_string(), FunctionSignature {
            name: "file_write".to_string(),
            param_names: vec!["path".to_string(), "data".to_string()],
            param_types: vec![Some(Type::String), Some(Type::String)],
            return_type: Some(Type::Bool),
            span: Span::new(0, 0),
        });

        // file_exists(path: string) -> bool
        functions.insert("file_exists".to_string(), FunctionSignature {
            name: "file_exists".to_string(),
            param_names: vec!["path".to_string()],
            param_types: vec![Some(Type::String)],
            return_type: Some(Type::Bool),
            span: Span::new(0, 0),
        });

        // path_join(a, b) -> string
        functions.insert("path_join".to_string(), FunctionSignature {
            name: "path_join".to_string(),
            param_names: vec!["a".to_string(), "b".to_string()],
            param_types: vec![Some(Type::String), Some(Type::String)],
            return_type: Some(Type::String),
            span: Span::new(0, 0),
        });

        // path_ext(p) -> string
        functions.insert("path_ext".to_string(), FunctionSignature {
            name: "path_ext".to_string(),
            param_names: vec!["p".to_string()],
            param_types: vec![Some(Type::String)],
            return_type: Some(Type::String),
            span: Span::new(0, 0),
        });

        // path_filename(p) -> string
        functions.insert("path_filename".to_string(), FunctionSignature {
            name: "path_filename".to_string(),
            param_names: vec!["p".to_string()],
            param_types: vec![Some(Type::String)],
            return_type: Some(Type::String),
            span: Span::new(0, 0),
        });

        // env_get(key) -> string
        functions.insert("env_get".to_string(), FunctionSignature {
            name: "env_get".to_string(),
            param_names: vec!["key".to_string()],
            param_types: vec![Some(Type::String)],
            return_type: Some(Type::String),
            span: Span::new(0, 0),
        });

        // env_set(key, val) -> bool
        functions.insert("env_set".to_string(), FunctionSignature {
            name: "env_set".to_string(),
            param_names: vec!["key".to_string(), "val".to_string()],
            param_types: vec![Some(Type::String), Some(Type::String)],
            return_type: Some(Type::Bool),
            span: Span::new(0, 0),
        });

        // process_exit(code)
        functions.insert("process_exit".to_string(), FunctionSignature {
            name: "process_exit".to_string(),
            param_names: vec!["code".to_string()],
            param_types: vec![Some(Type::Int)],
            return_type: Some(Type::Void),
            span: Span::new(0, 0),
        });

        // time_now() -> int
        functions.insert("time_now".to_string(), FunctionSignature {
            name: "time_now".to_string(),
            param_names: vec![],
            param_types: vec![],
            return_type: Some(Type::Int),
            span: Span::new(0, 0),
        });

        // time_sleep(ms)
        functions.insert("time_sleep".to_string(), FunctionSignature {
            name: "time_sleep".to_string(),
            param_names: vec!["ms".to_string()],
            param_types: vec![Some(Type::Int)],
            return_type: Some(Type::Void),
            span: Span::new(0, 0),
        });

        // str_to_int(s) -> int
        functions.insert("str_to_int".to_string(), FunctionSignature {
            name: "str_to_int".to_string(),
            param_names: vec!["s".to_string()],
            param_types: vec![Some(Type::String)],
            return_type: Some(Type::Int),
            span: Span::new(0, 0),
        });

        // str_to_float(s) -> f64
        functions.insert("str_to_float".to_string(), FunctionSignature {
            name: "str_to_float".to_string(),
            param_names: vec!["s".to_string()],
            param_types: vec![Some(Type::String)],
            return_type: Some(Type::F64),
            span: Span::new(0, 0),
        });

        // str_contains(haystack, needle) -> bool
        functions.insert("str_contains".to_string(), FunctionSignature {
            name: "str_contains".to_string(),
            param_names: vec!["haystack".to_string(), "needle".to_string()],
            param_types: vec![Some(Type::String), Some(Type::String)],
            return_type: Some(Type::Bool),
            span: Span::new(0, 0),
        });

        // str_starts_with(s, prefix) -> bool
        functions.insert("str_starts_with".to_string(), FunctionSignature {
            name: "str_starts_with".to_string(),
            param_names: vec!["s".to_string(), "prefix".to_string()],
            param_types: vec![Some(Type::String), Some(Type::String)],
            return_type: Some(Type::Bool),
            span: Span::new(0, 0),
        });

        // str_ends_with(s, suffix) -> bool
        functions.insert("str_ends_with".to_string(), FunctionSignature {
            name: "str_ends_with".to_string(),
            param_names: vec!["s".to_string(), "suffix".to_string()],
            param_types: vec![Some(Type::String), Some(Type::String)],
            return_type: Some(Type::Bool),
            span: Span::new(0, 0),
        });

        Self {
            source,
            structs: HashMap::new(),
            functions,
            scopes: Vec::new(),
            current_function: None,
            imported_modules: std::collections::HashSet::new(),
        }
    }

    pub fn register_app_toolkit(&mut self) {
        self.functions.insert(
            "window_open".to_string(),
            FunctionSignature {
                name: "window_open".to_string(),
                param_names: vec!["title".to_string(), "width".to_string(), "height".to_string()],
                param_types: vec![Some(Type::String), Some(Type::Int), Some(Type::Int)],
                return_type: Some(Type::Bool),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "window_close".to_string(),
            FunctionSignature {
                name: "window_close".to_string(),
                param_names: vec![],
                param_types: vec![],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "window_should_close".to_string(),
            FunctionSignature {
                name: "window_should_close".to_string(),
                param_names: vec![],
                param_types: vec![],
                return_type: Some(Type::Bool),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "window_poll".to_string(),
            FunctionSignature {
                name: "window_poll".to_string(),
                param_names: vec![],
                param_types: vec![],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "draw_clear".to_string(),
            FunctionSignature {
                name: "draw_clear".to_string(),
                param_names: vec!["r".to_string(), "g".to_string(), "b".to_string()],
                param_types: vec![Some(Type::Int), Some(Type::Int), Some(Type::Int)],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "draw_rect".to_string(),
            FunctionSignature {
                name: "draw_rect".to_string(),
                param_names: vec![
                    "x".to_string(),
                    "y".to_string(),
                    "w".to_string(),
                    "h".to_string(),
                    "r".to_string(),
                    "g".to_string(),
                    "b".to_string(),
                ],
                param_types: vec![
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                ],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "draw_circle".to_string(),
            FunctionSignature {
                name: "draw_circle".to_string(),
                param_names: vec![
                    "cx".to_string(),
                    "cy".to_string(),
                    "radius".to_string(),
                    "r".to_string(),
                    "g".to_string(),
                    "b".to_string(),
                ],
                param_types: vec![
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                ],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "draw_line".to_string(),
            FunctionSignature {
                name: "draw_line".to_string(),
                param_names: vec![
                    "x1".to_string(),
                    "y1".to_string(),
                    "x2".to_string(),
                    "y2".to_string(),
                    "r".to_string(),
                    "g".to_string(),
                    "b".to_string(),
                ],
                param_types: vec![
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                ],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "draw_text".to_string(),
            FunctionSignature {
                name: "draw_text".to_string(),
                param_names: vec![
                    "text".to_string(),
                    "x".to_string(),
                    "y".to_string(),
                    "size".to_string(),
                    "r".to_string(),
                    "g".to_string(),
                    "b".to_string(),
                ],
                param_types: vec![
                    Some(Type::String),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                    Some(Type::Int),
                ],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "window_present".to_string(),
            FunctionSignature {
                name: "window_present".to_string(),
                param_names: vec![],
                param_types: vec![],
                return_type: Some(Type::Void),
                span: Span::new(0, 0),
            },
        );

        self.functions.insert(
            "is_key_down".to_string(),
            FunctionSignature {
                name: "is_key_down".to_string(),
                param_names: vec!["key".to_string()],
                param_types: vec![Some(Type::Int)],
                return_type: Some(Type::Bool),
                span: Span::new(0, 0),
            },
        );
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
        // Pass 0: Check imports / annotations
        for ann in &program.annotations {
            if ann.name == "blueprint" {
                if ann.target == "AppToolkit" {
                    self.register_app_toolkit();
                } else if ann.target == "C" {
                    // Phase 3: C FFI module — allow C.Call() as dynamic
                    self.imported_modules.insert("C".to_string());
                } else if ann.target.starts_with("modules/") {
                    let mod_name = ann.target.trim_start_matches("modules/").to_string();
                    let simple_name = mod_name
                        .trim_end_matches(".blm")
                        .split('/')
                        .last()
                        .unwrap_or(&mod_name)
                        .to_string();
                    self.imported_modules.insert(mod_name);
                    self.imported_modules.insert(simple_name);
                }
            }
        }

        // Phase 3: Register C.Extern declarations as callable functions with Any return type
        for extern_decl in &program.c_externs {
            // Parse "return_type func_name(params...)" roughly
            let sig = extern_decl.signature.trim();
            // Find the first '(' to determine function name
            if let Some(paren_pos) = sig.find('(') {
                let before = sig[..paren_pos].trim();
                // Last word before '(' is the function name
                if let Some(name) = before.split_whitespace().last() {
                    let name = name.to_string();
                    self.functions.insert(name.clone(), FunctionSignature {
                        name,
                        param_names: vec![],
                        param_types: vec![],  // Accept any args
                        return_type: Some(Type::Custom("Any".into())),
                        span: extern_decl.span,
                    });
                }
            }
        }

        // Pass 0.5: Blueprint.Setup() validation
        // Mandatory for executable programs (programs with func Main())
        let has_main = program.items.iter().any(|item| matches!(item, Item::Func(f) if f.name == "Main"));
        let setup_count = program.items.iter().filter(|item| matches!(item, Item::Setup(_))).count();

        if has_main {
            if setup_count == 0 {
                return self.error(
                    "Missing mandatory 'Blueprint.Setup(\"AppName\", true)' declaration. Executable Blueprint programs with Main() require exactly one setup declaration.",
                    program.span,
                );
            } else if setup_count > 1 {
                // Find second setup span
                let mut seen = false;
                for item in &program.items {
                    if let Item::Setup(s) = item {
                        if seen {
                            return self.error(
                                "Duplicate 'Blueprint.Setup()' call. A Blueprint application can only have one setup declaration.",
                                s.span,
                            );
                        }
                        seen = true;
                    }
                }
            }
        }

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
            Expr::Float(_) => Ok(Type::F64),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::String(_) => Ok(Type::String),
            Expr::Ident(name) => {
                if let Some(ty) = self.lookup_variable(name) {
                    Ok(ty)
                } else if self.functions.contains_key(name) {
                    Ok(Type::Custom("Function".into()))
                } else if self.imported_modules.contains(name) {
                    Ok(Type::Custom("Module".into()))
                } else {
                    self.error(format!("Undefined variable '{}'", name), span)
                }
            }
            Expr::UnaryOp(UnOp::Neg, inner) => {
                let ty = self.analyze_expr(inner)?;
                if !ty.is_numeric() && ty != Type::Custom("Any".into()) {
                    return self.error(format!("Cannot negate non-numeric type {}", ty), span);
                }
                Ok(ty)
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
                        if lhs_ty.is_numeric() && rhs_ty.is_numeric() {
                            if lhs_ty.is_float() || rhs_ty.is_float() {
                                if lhs_ty == Type::F32 && rhs_ty == Type::F32 {
                                    Ok(Type::F32)
                                } else {
                                    Ok(Type::F64)
                                }
                            } else if lhs_ty == rhs_ty {
                                Ok(lhs_ty)
                            } else {
                                Ok(Type::Int)
                            }
                        } else if (lhs_ty == Type::Custom("Any".into()) && rhs_ty.is_numeric())
                            || (rhs_ty == Type::Custom("Any".into()) && lhs_ty.is_numeric())
                            || (lhs_ty == Type::Custom("Any".into()) && rhs_ty == Type::Custom("Any".into()))
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
                        if (lhs_ty.is_numeric() || lhs_ty == Type::Custom("Any".into()))
                            && (rhs_ty.is_numeric() || rhs_ty == Type::Custom("Any".into()))
                        {
                            Ok(Type::Bool)
                        } else {
                            self.error(
                                format!(
                                    "Comparison {:?} requires numeric operands, got {} and {}",
                                    op, lhs_ty, rhs_ty
                                ),
                                span,
                            )
                        }
                    }
                    BinOp::EqEq | BinOp::Neq => {
                        // Can check equality between matching types, compatible numeric types, or Any
                        if lhs_ty == rhs_ty
                            || (lhs_ty.is_numeric() && rhs_ty.is_numeric())
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

                if sig.name != "output" && !sig.param_names.is_empty() && sig.param_names.len() != args.len() {
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
            Expr::MethodCall(target, method, args) => {
                let target_ty = self.analyze_expr(target)?;
                for arg in args {
                    self.analyze_expr(arg)?;
                }

                // If calling a method on an imported module (e.g. library.example_func(...))
                if target_ty == Type::Custom("Module".into()) {
                    if let Some(sig) = self.functions.get(method) {
                        return Ok(sig.return_type.clone().unwrap_or(Type::Custom("Any".into())));
                    }
                    // If function was imported, return Any/Type
                    return Ok(Type::Custom("Any".into()));
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
