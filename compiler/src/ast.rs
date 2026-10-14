use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub annotations: Vec<Annotation>,
    pub items: Vec<Item>,
    pub span: Span,
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
    Int,
    Bool,
    String,
    Void,
    Custom(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Custom(name) => write!(f, "{}", name),
        }
    }
}
