use crate::lexer::Span;

/// A C FFI declaration: extern function signature
#[derive(Debug, Clone, PartialEq)]
pub struct CExtern {
    pub signature: String, // Raw C signature string, e.g. "double sqrt(double x)"
    pub span: Span,
}

/// A C FFI include directive: C.Include("header.h")
#[derive(Debug, Clone, PartialEq)]
pub struct CInclude {
    pub header: String,
    pub span: Span,
}

/// A C FFI link directive: C.Link("libname")
#[derive(Debug, Clone, PartialEq)]
pub struct CLink {
    pub lib: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub annotations: Vec<Annotation>,
    pub items: Vec<Item>,
    pub span: Span,
    /// C FFI includes collected at parse time
    pub c_includes: Vec<CInclude>,
    /// C FFI link libraries
    pub c_links: Vec<CLink>,
    /// C FFI extern declarations
    pub c_externs: Vec<CExtern>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: String,
    pub target: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Setup(SetupStmt),
    Struct(StructDef),
    Func(FuncDef),
    Var(VarDef),
    Stmt(Stmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetupStmt {
    pub app_name: String,
    pub use_deps: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDef {
    pub name: String,
    pub explicit_ty: Option<Type>,
    pub expr: SpannedExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDef(VarDef),
    Expr(SpannedExpr),
    Return(Option<SpannedExpr>, Span),
    If(IfStmt),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDef(v) => v.span,
            Stmt::Expr(e) => e.span,
            Stmt::Return(_, sp) => *sp,
            Stmt::If(i) => i.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: SpannedExpr,
    pub then_branch: Vec<Stmt>,
    pub else_branch: Option<Vec<Stmt>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedExpr {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Ident(String),
    BinaryOp(Box<SpannedExpr>, BinOp, Box<SpannedExpr>),
    UnaryOp(UnOp, Box<SpannedExpr>),
    Call(String, Vec<SpannedExpr>),
    MethodCall(Box<SpannedExpr>, String, Vec<SpannedExpr>),
    FieldAccess(Box<SpannedExpr>, String),
    StructInit(String, Vec<(String, SpannedExpr)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Lt,
    Gte,
    Lte,
    EqEq,
    Neq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,      // default/legacy 64-bit integer
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    String,
    Void,
    Custom(String),
}

impl Type {
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Int
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Custom(name) => write!(f, "{}", name),
        }
    }
}
