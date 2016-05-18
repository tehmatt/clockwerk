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
    ConstString(String),
    ConstInt(i32),
    ConstKey(KeyType),
    ConstColor(ColorType),
    Var(Ident),
    Binop(Box<Expr>, OpType, Box<Expr>),
    Unop(Ident, OpType)
}

#[derive(Debug)]
pub enum Statement {
    // Declarations
    Mutable(Type, Ident, Expr),
    Const(Type, Ident, Expr),

    // Control flow
    Block(Vec<Statement>),
    Loop(Box<Statement>),
    Break,
    Input(Vec<(KeyType, Statement)>),

    // Variable modifications
    Assign(Ident, Expr),
}
