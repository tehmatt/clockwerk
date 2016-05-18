use nom::IResult;
use nom::{space, multispace, digit, alpha, alphanumeric};
use std::str;
use std::str::FromStr;
use ast::*;

named!(boolean_literals<bool>,
    alt!(
        chain!(tag!("true"), || true)
      | chain!(tag!("false"), || false)
    )
);

// TODO: Bounds checking
named!(integer_literals<i32>,
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
                None => Type::Int(0, i32::max_value())
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
      | map!(integer_literals, |x : i32| Expr::ConstInt(x))
      | map!(key_literals, |x : KeyType| Expr::ConstKey(x))
      | map!(color_literals, |x : ColorType| Expr::ConstColor(x))
      | map!(string_literals, |x : String| Expr::ConstString(x))
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
                        l: idents
                      ~ char!('=')
                      ~ r: exprs,
                      || Statement::Assign(l, r)
                    )
                ),
                preceded!(opt!(space), tag!(";"))
            )
        )
    )
);

named!(arguments<(Type, Ident)>,
    separated_pair!(types, space, idents)
);

named!(functions<Function>,
    chain!(
        ret: types?
      ~ space
      ~ name: idents
      ~ space?
      ~ char!('(')
      ~ args: delimited!(char!('('), separated_list!(char!(','), arguments), char!(')'))
      ~ body: statements,
      || Function { ret: ret, name: name, args: args, body: body}
    )
);

pub fn parse(source: &str) -> Result<Function, String> {
    match functions(source.as_bytes()) {
        IResult::Done(_, t) => Ok(t),
        IResult::Error(e) => Err(format!("Parse error: {:?}", e)),
        IResult::Incomplete(n) => Err(format!("Parse incomplete: needs {:?}", n))
    }
}
