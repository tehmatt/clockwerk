use nom::IResult;
use nom::Err::Position;
use nom::ErrorKind;
use nom::{space, multispace, digit, alpha, alphanumeric};
use std::str;
use std::str::FromStr;
use ast::*;

// TODO: parse comments
// This should include a general whitespace solution also

named!(boolean_literals<bool>,
    alt!(
        chain!(tag!("true"), || true)
      | chain!(tag!("false"), || false)
    )
);

// TODO: Negatives
named!(integer_literals<u32>,
   map_res!(
       map_res!(
           digit,
           str::from_utf8
       ),
       FromStr::from_str
   )
);

// TODO: String literals cannot currently contain quotes
named!(string_literals<String>,
    chain!(
        tag!("\"")
      ~ text: map_res!(
            take_until_and_consume!("\""),
            str::from_utf8
        ),
        || text.to_string()
    )
);

// TODO: macros
named!(key_literals<KeyType>,
    alt!(
        chain!(tag!("W"), || KeyType::W)
      | chain!(tag!("A"), || KeyType::A)
      | chain!(tag!("S"), || KeyType::S)
      | chain!(tag!("D"), || KeyType::D)
    )
);

// TODO: macros
named!(color_literals<ColorType>,
    alt!(
        chain!(tag!("red"), || ColorType::Red)
      | chain!(tag!("white"), || ColorType::White)
    )
);

named!(binops<OpType>,
    alt!(
        chain!(tag!("+"), || OpType::Plus)
      | chain!(tag!("-"), || OpType::Minus)
    )
);

named!(unops<OpType>,
    alt!(
        chain!(tag!("++"), || OpType::UPlus)
      | chain!(tag!("--"), || OpType::UMinus)
    )
);

named!(types<Type>,
    alt!(
        chain!(
            tag!("int")
          ~ bounds: chain!(
                char!('<')
              ~ low: integer_literals
              ~ char!(',')
              ~ space?
              ~ high: integer_literals
              ~ char!('>'),
              || (low, high)
            )?,
            || match bounds {
                Some((low, high)) => Type::Int(low, high),
                None => Type::Int(0, u32::max_value())
            }
        )
      | chain!(tag!("color"), || Type::Color)
      | chain!(tag!("key"), || Type::Key)
      | chain!(tag!("string"), || Type::Printable)
      | chain!(tag!("bool"), || Type::Bool)
      | map!(preceded!(tag!("list<"), terminated!(types, tag!(">"))), |x : Type| Type::List(Box::new(x)))
    )
);

named!(idents<String>,
    chain!(
        first: map_res!(
            alpha,
            str::from_utf8
        )
      ~ remainder: map_res!(
          alphanumeric,
          str::from_utf8
        )?,
      || match remainder {
          Some(r) => first.to_string() + r,
          None => first.to_string()
      }
    )
);

named!(calls<Expr>,
    chain!(
        func: idents
      ~ space?
      ~ char!('(')
      ~ multispace?
      ~ args: separated_list!(
            delimited!(opt!(multispace), char!(','), opt!(multispace)),
            exprs
        )
      ~ char!(')')
      ~ multispace?,
      || Expr::Call(func, args)
    )
);

named!(parens<Expr>,
    delimited!(
        chain!(char!('(') ~ multispace?, || ()),
        exprs,
        chain!(multispace? ~ char!(')'), || ())
    )
);

named!(terms<Expr>,
    alt!(
        map!(boolean_literals, |x : bool| Expr::ConstBool(x))
      | map!(integer_literals, |x : u32| Expr::ConstInt(x))
      | map!(key_literals, |x : KeyType| Expr::ConstKey(x))
      | map!(color_literals, |x : ColorType| Expr::ConstColor(x))
      | map!(string_literals, |x : String| Expr::ConstString(x))
      | calls
      | map!(idents, |x : Ident| Expr::Var(x))
      | chain!(
            l: idents
          ~ space?
          ~ o: unops,
          || Expr::Unop(l, o)
        )
      | parens
    )
);

named!(exprs<Expr>,
    delimited!(
        opt!(multispace),
        alt!(
            chain!(
                l: terms
              ~ multispace?
              ~ o: binops
              ~ multispace?
              ~ r: exprs,
                || Expr::Binop(Box::new(l), o, Box::new(r))
            )
          | terms
        ),
        opt!(multispace)
    )
);

named!(declarations<Statement>,
    chain!(
        mutable: opt!(tag!("mut"))
      ~ typ: types
      ~ space
      ~ name: idents
      ~ space?
      ~ char!('=')
      ~ space?
      ~ value: exprs,
      || match mutable {
          Some(_) => Statement::Mutable(typ, name, value),
          None => Statement::Const(typ, name, value)
      }
    )
);

named!(input_cases<(KeyType, Statement)>,
    chain!(
        multispace?
      ~ case: key_literals
      ~ space?
      ~ tag!("=>")
      ~ space?
      ~ control: statements
      ~ multispace?,
      || (case, control)
    )
);

named!(statements<Statement>,
    preceded!(
        opt!(multispace),
        alt!(
            chain!( // Blocks
                tag!("{")
              ~ multispace?
              ~ statements: many0!(statements)
              ~ multispace?
              ~ tag!("}"),
              || Statement::Block(statements)
            )
          | chain!( // Loops
                tag!("loop")
              ~ statements: statements,
              || Statement::Loop(Box::new(statements))
            )
          | chain!(
                tag!("input")
              ~ multispace?
              ~ tag!("{")
              ~ cases: many1!(input_cases)
              ~ tag!("}"),
              || Statement::Input(cases)
            )
          | terminated!(
                alt!(
                    chain!(tag!("break"), || Statement::Break)
                  | declarations
                  | chain!(
                        tag!("return")
                      ~ multispace?
                      ~ expr: exprs,
                      || Statement::Return(expr)
                    )
                  | chain!(
                        l: idents
                      ~ delimited!(opt!(space), char!('='), opt!(space))
                      ~ r: exprs,
                      || Statement::Assign(l, r)
                    )
                  | map!(exprs, |x : Expr| Statement::Expr(x))
                ),
                preceded!(opt!(space), tag!(";"))
            )
        )
    )
);

named!(arguments<(Type, Ident)>,
    delimited!(
        opt!(multispace),
        separated_pair!(types, space, idents),
        opt!(multispace)
    )
);

named!(functions<Function>,
    chain!(
        ret: terminated!(types, space)?
      ~ name: idents
      ~ space?
      ~ args: delimited!(char!('('), separated_list!(char!(','), arguments), char!(')'))
      ~ body: error!(ErrorKind::Custom(0), statements),
      || Function { ret: ret, name: name, args: args, body: body}
    )
);

// TODO: silently ignores failure when parsing functions - should look for
// non-spaces and fail if they've been seen
named!(files<AST>,
    map!(
        many1!(delimited!(opt!(multispace), functions, opt!(multispace))),
        |x : Vec<Function>| AST(x)
    )
);

pub fn parse(source: String) -> Result<AST, String> {
    match files(source.as_bytes()) {
        IResult::Done(_, t) => Ok(t),
        IResult::Error(e) =>
            match e {
                Position(code, bytes) => Err(format!("Parse error at {:?}: {:?}", code, str::from_utf8(bytes).unwrap())),
                _ => Err(format!("Unformatted error :("))
            },
        IResult::Incomplete(n) => Err(format!("Parse incomplete: needs {:?}", n))
    }
}
