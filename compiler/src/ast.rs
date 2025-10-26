#[derive(Debug, Clone)]
pub enum ViraType {
    Int,
    Float,
    Bool,
    String,
    Array(Box<ViraType>),
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub typ: ViraType,
}

#[derive(Debug)]
pub enum AstNode {
    Literal(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    Binary(Box<AstNode>, BinOp, Box<AstNode>),
    Unary(UnaryOp, Box<AstNode>),
    VarDecl(String, ViraType, Box<AstNode>),
    VarRef(String),
    FuncDecl(String, Vec<(String, ViraType)>, ViraType, Box<AstNode>),
    Call(String, Vec<AstNode>),
    If(Box<AstNode>, Box<AstNode>, Option<Box<AstNode>>),
    While(Box<AstNode>, Box<AstNode>),
    For(String, Box<AstNode>, Box<AstNode>, Box<AstNode>, Box<AstNode>),
    Return(Option<Box<AstNode>>),
    Block(Vec<AstNode>),
    Write(Box<AstNode>),
    ArrayLiteral(Vec<AstNode>),
    Index(Box<AstNode>, Box<AstNode>),
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}
