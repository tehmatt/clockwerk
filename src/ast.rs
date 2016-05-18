#[derive(Debug)]
pub enum ColorType {
    Red, White
}

#[derive(Debug)]
pub enum KeyType {
    W, A, S, D
}

#[derive(Debug)]
pub enum OpType {
    Plus, Minus, UPlus, UMinus
}

#[derive(Debug)]
pub enum Type {
    Bool,
    Int(i32, i32),
    Color,
    Key,
    Printable,
    List(Box<Type>)
}

pub type Ident = String;

#[derive(Debug)]
pub enum Expr {
    // Constants
    ConstBool(bool),
    ConstInt(i32),
    ConstKey(KeyType),
    ConstColor(ColorType),
    ConstString(String),

    // Variable expressions
    Var(Ident),
    Binop(Box<Expr>, OpType, Box<Expr>),
    Unop(Ident, OpType),

    Call(Ident, Vec<Expr>)
}

#[derive(Debug)]
pub enum Statement {
    // Declarations and modifications
    Mutable(Type, Ident, Expr),
    Const(Type, Ident, Expr),
    Assign(Ident, Expr),

    // Control flow
    Block(Vec<Statement>),
    Loop(Box<Statement>),
    Break,
    Input(Vec<(KeyType, Statement)>),
    Return(Expr),

    // Side-effects
    Expr(Expr)
}

#[derive(Debug)]
pub struct Function {
    pub ret: Option<Type>,
    pub name: Ident,
    pub args: Vec<(Type, Ident)>,
    pub body: Statement
}

#[derive(Debug)]
pub enum Tree {
    Functions(Vec<Function>)
}
