use std::fmt;
use ansi_term::Colour::*;

#[derive(Debug)]
pub enum ColorType {
    Red, White
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum KeyType {
    W, A, S, D
}

#[derive(Debug)]
pub enum OpType {
    Plus, Minus, UPlus, UMinus
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bool,
    Int(u32, u32),
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
    ConstInt(u32),
    ConstKey(KeyType),
    ConstColor(ColorType),
    ConstString(String),
    ConstList(Vec<Expr>),

    // Variable expressions
    Var(Ident),
    Binop(Box<Expr>, OpType, Box<Expr>),
    Unop(Ident, OpType),

    Call(Ident, Vec<Expr>),
    Elem(Box<Expr>, Box<Expr>)
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

pub struct AST(pub Vec<Function>);

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for func in self.0.iter() {
            try!(write!(f, "{}\n\n", func))
        }
        return Ok(());
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ret {
            Some(ref t) => try!(write!(f, "{} ", t)),
            None => ()
        }

        try!(write!(f, "{}(", Purple.paint(self.name.to_string())));

        let mut first = true;
        for &(ref typ, ref name) in self.args.iter() {
            if !first {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{} {}", typ, Cyan.paint(name.to_string())));
            first = false;
        }

        try!(write!(f, ") "));
        try!(write!(f, "{}", self.body));

        return Ok(());
    }
}


// Indentation for formatting
// XXX: There must be a better way
static mut indentation_level : i32 = 0;

// There must be a better way
fn write_indent(f : &mut fmt::Formatter) {
    unsafe {
        for _ in 0..indentation_level {
            write!(f, "    ").unwrap();
        }
    }
}

fn indent() {
    unsafe {
        indentation_level += 1;
    }
}

fn undent() {
    unsafe {
        indentation_level -= 1;
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Statement::Mutable(ref typ, ref name, ref expr) => {
                write_indent(f);
                writeln!(f, "mut {} {} = {};", typ, Cyan.paint(name.to_string()), expr)
            },
            &Statement::Const(ref typ, ref name, ref expr) => {
                write_indent(f);
                writeln!(f, "{} {} = {};", typ, Cyan.paint(name.to_string()), expr)
            },
            &Statement::Assign(ref name, ref expr) => {
                write_indent(f);
                writeln!(f, "{} = {};", Cyan.paint(name.to_string()), expr)
            },
            &Statement::Block(ref stmts) => {
                try!(writeln!(f, "{{"));
                indent();

                for stmt in stmts {
                    try!(write!(f, "{}", stmt));
                }

                undent();
                write_indent(f);
                writeln!(f, "}}")
            },
            &Statement::Loop(ref stmt) => {
                write_indent(f);
                writeln!(f, "{} {}", Red.paint("loop"), stmt)
            },
            &Statement::Input(ref branches) => {
                write_indent(f);
                try!(writeln!(f, "{} {{", Red.paint("input")));
                indent();

                // TODO: handle printing of single statements
                for &(ref key, ref arm) in branches {
                    write_indent(f);
                    try!(write!(f, "{:?} => {}", key, arm));
                }

                undent();
                write_indent(f);
                writeln!(f, "}}")
            },
            &Statement::Return(ref expr) => {
                write_indent(f);
                writeln!(f, "{} {};", Red.paint("return"), expr)
            },
            &Statement::Break => {
                write_indent(f);
                writeln!(f, "{};", Red.paint("break"))
            },
            &Statement::Expr(ref expr) => {
                write_indent(f);
                writeln!(f, "{};", expr)
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expr::ConstBool(ref b) => write!(f, "{}", Green.paint(b.to_string())),
            &Expr::ConstInt(ref i) => write!(f, "{}", Green.paint(i.to_string())),
            &Expr::ConstKey(ref k) => write!(f, "{}", Green.paint(format!("{:?}", k))),
            &Expr::ConstColor(ref c) => write!(f, "{}", Green.paint(format!("{:?}", c))),
            &Expr::ConstString(ref s) => write!(f, "\"{}\"", Green.paint(s.to_string())),
            &Expr::Var(ref name) => write!(f, "{}", Cyan.paint(name.to_string())),
            &Expr::Binop(ref l, ref o, ref r) => write!(f, "({} {} {})", l, o, r),
            &Expr::Unop(ref l, ref o) => write!(f, "{}{}", l, o),
            &Expr::Elem(ref list, ref elem) => write!(f, "{}[{}]", list, elem),
            &Expr::ConstList(ref elems) => {
                try!(write!(f, "["));

                let mut first = true;
                for elem in elems {
                    if !first {
                        try!(write!(f, ", "));
                    }

                    try!(write!(f, "{}", elem));
                    first = false;
                }

                write!(f, "]")
            },
            &Expr::Call(ref name, ref args) => {
                try!(write!(f, "{}(", Purple.paint(name.to_string())));

                let mut first = true;
                for arg in args {
                    if !first {
                        try!(write!(f, ", "));
                    }

                    try!(write!(f, "{}", arg));
                    first = false;
                }

                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Yellow.paint(match self {
            &Type::Bool => "bool".to_string(),
            &Type::Color => "color".to_string(),
            &Type::Key => "key".to_string(),
            &Type::Printable => "string".to_string(),
            &Type::Int(ref low, ref high) => format!("int<{}, {}>", low, high),
            &Type::List(ref t) => format!("list<{}>", t)
        }))
    }
}

impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &OpType::Plus => "+",
            &OpType::Minus => "+",
            &OpType::UPlus => "++",
            &OpType::UMinus => "--"
        })
    }
}
