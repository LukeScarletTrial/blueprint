use crate::ast::*;
use std::collections::HashMap;
use crate::errors::CompileError;

#[derive(Debug)]
pub struct Runtime {
    pub output: Vec<String>,
}

impl Runtime {
    pub fn new() -> Self { Self { output: Vec::new() } }
    pub fn output(&mut self, s: &str) { self.output.push(s.to_string()); }
}

#[derive(Debug)]
pub struct Interpreter {
    pub structs: HashMap<String, StructDef>,
    pub funcs: HashMap<String, FuncDef>,
    pub globals: HashMap<String, Value>,
    pub runtime: Runtime,
}

#[derive(Debug, Clone)]n
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Struct(String, Vec<Value>),
    None,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            funcs: HashMap::new(),
            globals: HashMap::new(),
            runtime: Runtime::new(),
        }
    }

    pub fn load_program(&mut self, _prog: Program) -> Result<(), CompileError> {
        // Stub: real loader should populate structs and funcs
        Ok(())
    }

    pub fn run_main(&mut self) -> Result<(), CompileError> {
        // find Main and call it
        if let Some(main) = self.funcs.get("Main") {
            let mut env = HashMap::new();
            self.execute_block(&main.body, &mut env)
        } else {
            Err(CompileError::new("Main function not found"))
        }
    }

    fn execute_block(&mut self, _stmts: &[Stmt], _env: &mut HashMap<String, Value>) -> Result<(), CompileError> {
        Ok(())
    }
}
