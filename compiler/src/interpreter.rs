use crate::ast::*;
use crate::lexer::{byte_to_location, SourceLocation, Span};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    Void,
}

impl Value {
    pub fn display_string(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Void => "()".to_string(),
            Value::Struct { name, fields } => {
                let mut entries: Vec<_> = fields.iter().collect();
                entries.sort_by_key(|(k, _)| (*k).clone());
                let field_strs: Vec<String> = entries
                    .into_iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display_string()))
                    .collect();
                format!("{}[{}]", name, field_strs.join(", "))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
    pub location: SourceLocation,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Runtime error at line {}, column {}: {}",
            self.location.line, self.location.column, self.message
        )
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Default)]
pub struct Runtime {
    pub output: Vec<String>,
}

impl Runtime {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }

    pub fn output(&mut self, s: &str) {
        self.output.push(s.to_string());
    }
}

pub struct Interpreter {
    source: String,
    structs: HashMap<String, StructDef>,
    funcs: HashMap<String, FuncDef>,
    globals: HashMap<String, Value>,
    pub runtime: Runtime,
    setup_info: Option<(String, bool)>,
}

enum ControlFlow {
    None,
    Return(Value),
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            structs: HashMap::new(),
            funcs: HashMap::new(),
            globals: HashMap::new(),
            runtime: Runtime::new(),
            setup_info: None,
        }
    }

    fn error<T>(&self, message: impl Into<String>, span: Span) -> Result<T, RuntimeError> {
        let location = byte_to_location(&self.source, span.start);
        Err(RuntimeError {
            message: message.into(),
            span,
            location,
        })
    }

    pub fn load_program(&mut self, source: &str, prog: &Program) -> Result<(), RuntimeError> {
        self.source = source.to_string();
        for item in &prog.items {
            match item {
                Item::Setup(setup) => {
                    self.setup_info = Some((setup.app_name.clone(), setup.use_deps));
                }
                Item::Struct(st) => {
                    self.structs.insert(st.name.clone(), st.clone());
                }
                Item::Func(func) => {
                    self.funcs.insert(func.name.clone(), func.clone());
                }
                Item::Var(var) => {
                    let mut env = vec![self.globals.clone()];
                    let val = self.eval_expr(&var.expr, &mut env)?;
                    self.globals.insert(var.name.clone(), val);
                }
                Item::Stmt(_) => {}
            }
        }
        Ok(())
    }

    pub fn run_main(&mut self) -> Result<Value, RuntimeError> {
        if let Some(main) = self.funcs.get("Main").cloned() {
            let mut env = vec![self.globals.clone()];
            let res = self.execute_block(&main.body, &mut env)?;
            match res {
                ControlFlow::Return(v) => Ok(v),
                ControlFlow::None => Ok(Value::Void),
            }
        } else {
            self.error("Main function not found", Span::new(0, 0))
        }
    }

    fn execute_block(
        &mut self,
        stmts: &[Stmt],
        env: &mut Vec<HashMap<String, Value>>,
    ) -> Result<ControlFlow, RuntimeError> {
        env.push(HashMap::new());

        for stmt in stmts {
            match stmt {
                Stmt::VarDef(var) => {
                    let val = self.eval_expr(&var.expr, env)?;
                    if let Some(scope) = env.last_mut() {
                        scope.insert(var.name.clone(), val);
                    }
                }
                Stmt::Expr(expr) => {
                    self.eval_expr(expr, env)?;
                }
                Stmt::Return(opt_expr, _) => {
                    let val = if let Some(e) = opt_expr {
                        self.eval_expr(e, env)?
                    } else {
                        Value::Void
                    };
                    env.pop();
                    return Ok(ControlFlow::Return(val));
                }
                Stmt::If(if_stmt) => {
                    let cond_val = self.eval_expr(&if_stmt.condition, env)?;
                    let is_true = match cond_val {
                        Value::Bool(b) => b,
                        _ => {
                            env.pop();
                            return self.error(
                                "Condition did not evaluate to a boolean",
                                if_stmt.condition.span,
                            );
                        }
                    };

                    if is_true {
                        let cf = self.execute_block(&if_stmt.then_branch, env)?;
                        if let ControlFlow::Return(v) = cf {
                            env.pop();
                            return Ok(ControlFlow::Return(v));
                        }
                    } else if let Some(else_branch) = &if_stmt.else_branch {
                        let cf = self.execute_block(else_branch, env)?;
                        if let ControlFlow::Return(v) = cf {
                            env.pop();
                            return Ok(ControlFlow::Return(v));
                        }
                    }
                }
            }
        }

        env.pop();
        Ok(ControlFlow::None)
    }

    fn lookup_var(env: &[HashMap<String, Value>], name: &str) -> Option<Value> {
        for scope in env.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn eval_expr(
        &mut self,
        expr: &SpannedExpr,
        env: &mut Vec<HashMap<String, Value>>,
    ) -> Result<Value, RuntimeError> {
        let span = expr.span;
        match &expr.expr {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Ident(name) => {
                if let Some(v) = Self::lookup_var(env, name) {
                    Ok(v)
                } else {
                    self.error(format!("Undefined variable '{}'", name), span)
                }
            }
            Expr::UnaryOp(UnOp::Neg, inner) => {
                let v = self.eval_expr(inner, env)?;
                match v {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    _ => self.error("Cannot negate non-integer", span),
                }
            }
            Expr::UnaryOp(UnOp::Not, inner) => {
                let v = self.eval_expr(inner, env)?;
                match v {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => self.error("Cannot invert non-boolean", span),
                }
            }
            Expr::BinaryOp(lhs, op, rhs) => {
                let l_val = self.eval_expr(lhs, env)?;
                let r_val = self.eval_expr(rhs, env)?;
                match op {
                    BinOp::Add => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                        (Value::String(a), Value::String(b)) => {
                            Ok(Value::String(format!("{}{}", a, b)))
                        }
                        (Value::String(a), b) => {
                            Ok(Value::String(format!("{}{}", a, b.display_string())))
                        }
                        (a, Value::String(b)) => {
                            Ok(Value::String(format!("{}{}", a.display_string(), b)))
                        }
                        _ => self.error("Invalid operand types for addition", span),
                    },
                    BinOp::Sub => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                        _ => self.error("Invalid operand types for subtraction", span),
                    },
                    BinOp::Mul => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                        _ => self.error("Invalid operand types for multiplication", span),
                    },
                    BinOp::Div => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                self.error("Division by zero", span)
                            } else {
                                Ok(Value::Int(a / b))
                            }
                        }
                        _ => self.error("Invalid operand types for division", span),
                    },
                    BinOp::Gt => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                        _ => self.error("Invalid operand types for '>'", span),
                    },
                    BinOp::Lt => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                        _ => self.error("Invalid operand types for '<'", span),
                    },
                    BinOp::Gte => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                        _ => self.error("Invalid operand types for '>='", span),
                    },
                    BinOp::Lte => match (l_val, r_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                        _ => self.error("Invalid operand types for '<='", span),
                    },
                    BinOp::EqEq => Ok(Value::Bool(l_val == r_val)),
                    BinOp::Neq => Ok(Value::Bool(l_val != r_val)),
                    BinOp::And => match (l_val, r_val) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                        _ => self.error("Invalid operand types for '&&'", span),
                    },
                    BinOp::Or => match (l_val, r_val) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                        _ => self.error("Invalid operand types for '||'", span),
                    },
                }
            }
            Expr::Call(name, args) => {
                if name == "output" {
                    let mut outputs = Vec::new();
                    for a in args {
                        let v = self.eval_expr(a, env)?;
                        outputs.push(v.display_string());
                    }
                    let line = outputs.join(" ");
                    self.runtime.output(&line);
                    return Ok(Value::Void);
                }
                if name == "str_len" {
                    if let Some(first) = args.first() {
                        let v = self.eval_expr(first, env)?;
                        let s = v.display_string();
                        return Ok(Value::Int(s.len() as i64));
                    }
                    return Ok(Value::Int(0));
                }
                if name == "math_abs" {
                    if let Some(first) = args.first() {
                        let v = self.eval_expr(first, env)?;
                        if let Value::Int(n) = v {
                            return Ok(Value::Int(n.abs()));
                        }
                    }
                    return Ok(Value::Int(0));
                }
                if name == "math_pow" {
                    if args.len() >= 2 {
                        let base_val = self.eval_expr(&args[0], env)?;
                        let exp_val = self.eval_expr(&args[1], env)?;
                        if let (Value::Int(b), Value::Int(e)) = (base_val, exp_val) {
                            return Ok(Value::Int(b.pow(e.max(0) as u32)));
                        }
                    }
                    return Ok(Value::Int(0));
                }
                if name == "math_min" {
                    if args.len() >= 2 {
                        let a_val = self.eval_expr(&args[0], env)?;
                        let b_val = self.eval_expr(&args[1], env)?;
                        if let (Value::Int(a), Value::Int(b)) = (a_val, b_val) {
                            return Ok(Value::Int(a.min(b)));
                        }
                    }
                    return Ok(Value::Int(0));
                }
                if name == "math_max" {
                    if args.len() >= 2 {
                        let a_val = self.eval_expr(&args[0], env)?;
                        let b_val = self.eval_expr(&args[1], env)?;
                        if let (Value::Int(a), Value::Int(b)) = (a_val, b_val) {
                            return Ok(Value::Int(a.max(b)));
                        }
                    }
                    return Ok(Value::Int(0));
                }
                if name == "input" {
                    // In non-interactive test/interpreter mode return a placeholder or read stdin
                    return Ok(Value::String("".to_string()));
                }
                if name == "window_open" {
                    return Ok(Value::Bool(true));
                }
                if name == "window_close" || name == "window_poll" || name == "window_present"
                    || name == "draw_clear" || name == "draw_rect" || name == "draw_circle"
                    || name == "draw_line" || name == "draw_text" {
                    return Ok(Value::Void);
                }
                if name == "window_should_close" {
                    return Ok(Value::Bool(false));
                }
                if name == "is_key_down" {
                    return Ok(Value::Bool(false));
                }

                let func = match self.funcs.get(name).cloned() {
                    Some(f) => f,
                    None => return self.error(format!("Function '{}' not found", name), span),
                };

                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.eval_expr(a, env)?);
                }

                let mut new_scope = HashMap::new();
                for (param, val) in func.params.iter().zip(arg_vals.into_iter()) {
                    new_scope.insert(param.name.clone(), val);
                }

                let mut func_env = vec![self.globals.clone(), new_scope];
                let res = self.execute_block(&func.body, &mut func_env)?;
                match res {
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::None => Ok(Value::Void),
                }
            }
            Expr::StructInit(name, named_fields) => {
                let st = match self.structs.get(name).cloned() {
                    Some(s) => s,
                    None => return self.error(format!("Struct '{}' not found", name), span),
                };

                let mut fields = HashMap::new();
                for (field_name, arg_expr) in named_fields.iter() {
                    let val = self.eval_expr(arg_expr, env)?;
                    // Positional syntax stores _0, _1, _2 — map back to real field name
                    let real_name = if field_name.starts_with('_') {
                        if let Ok(idx) = field_name[1..].parse::<usize>() {
                            st.fields.get(idx)
                                .map(|f| f.name.clone())
                                .unwrap_or_else(|| field_name.clone())
                        } else {
                            field_name.clone()
                        }
                    } else {
                        field_name.clone()
                    };
                    fields.insert(real_name, val);
                }

                Ok(Value::Struct {
                    name: name.clone(),
                    fields,
                })
            }
            Expr::FieldAccess(target, field_name) => {
                let target_val = self.eval_expr(target, env)?;
                match target_val {
                    Value::Struct { name: _, fields } => {
                        if let Some(v) = fields.get(field_name) {
                            Ok(v.clone())
                        } else {
                            self.error(format!("Field '{}' not found on struct", field_name), span)
                        }
                    }
                    _ => self.error("Field access on non-struct value", span),
                }
            }
            Expr::MethodCall(target, method, args) => {
                let _target_val = self.eval_expr(target, env)?;
                for a in args {
                    self.eval_expr(a, env)?;
                }
                self.error(format!("Method '{}' not implemented", method), span)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse_program;
    use crate::semantics::analyze;

    #[test]
    fn test_interpreter_example() {
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
        analyze(src, &prog).expect("semantics");

        let mut interp = Interpreter::new();
        interp.load_program(src, &prog).expect("load");
        interp.run_main().expect("run");

        assert_eq!(interp.runtime.output, vec!["User is a minor."]);
    }

    #[test]
    fn test_interpreter_adult_branch() {
        let src = r#"
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
    def userName = "AdultUser"
    def userAge = 25
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
        analyze(src, &prog).expect("semantics");

        let mut interp = Interpreter::new();
        interp.load_program(src, &prog).expect("load");
        interp.run_main().expect("run");

        assert_eq!(interp.runtime.output, vec!["User is an adult."]);
    }

    #[test]
    fn test_arithmetic_and_scoping() {
        let src = r#"
func Calc(x): [
    def y = x * 2 + 3
    return y
]

func Main(): [
    def result = Calc(5)
    output(result)
]
"#;
        let tokens = tokenize(src).expect("tokenize");
        let prog = parse_program(src, tokens).expect("parse");
        let mut interp = Interpreter::new();
        interp.load_program(src, &prog).expect("load");
        interp.run_main().expect("run");
        assert_eq!(interp.runtime.output, vec!["13"]);
    }
}
