use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp;

use ast::*;

type FunctionContext = HashMap<Ident, (Type, Vec<Type>)>;
type VariableContext = HashMap<Ident, (Type, bool)>;

fn subtype(t1 : &Type, t2 : &Type) -> bool {
    if t1 == t2 {
        return true;
    }

    match (t1, t2) {
        (&Type::Int(ref l1, ref h1), &Type::Int(ref l2, ref h2)) => (l1 <= l2) && (h1 >= h2),
        _ => false
    }
}

fn check_expr(e : &Expr, func_table : &FunctionContext, context : &VariableContext) -> Result<Type, String> {
    match e {
        &Expr::ConstBool(_) => Ok(Type::Bool),
        &Expr::ConstInt(x) => if x == u16::max_value() {
            Err(format!("Integer constants must be less than {}", u16::max_value()))
        } else {
            Ok(Type::Int(0, x + 1))
        },
        &Expr::ConstKey(_) => Ok(Type::Key),
        &Expr::ConstColor(_) => Ok(Type::Color),
        &Expr::ConstString(_) => Ok(Type::Printable),
        &Expr::ConstList(ref elems) => {
            if elems.len() > (u16::max_value() as usize) {
                Err(format!("Lists must contain less than {} elements", u16::max_value()))
            } else {
                Ok(Type::PrintableList(elems.len() as u16))
            }
        },
        &Expr::Var(ref name) => {
            match context.get(&*name) {
                Some(&(ref t, _)) => Ok(*t),
                None => Err(format!("Variable {} is referenced without being defined", name))
            }
        },
        &Expr::Binop(ref l, ref o, ref r) => {
            let t1 = try!(check_expr(l, func_table, context));
            let t2 = try!(check_expr(r, func_table, context));

            match (t1, *o, t2) {
                (_, OpType::UPlus, _) => Err(format!("Unary operator {} may not be used in a binary expression", o)),
                (_, OpType::UMinus, _) => Err(format!("Unary operator {} may not be used in a binary expression", o)),
                (Type::Int(l1,h1), _, Type::Int(l2, h2)) => Ok(Type::Int(cmp::min(l1, l2), cmp::max(h1, h2))),
                (Type::Printable, OpType::Plus, Type::Printable) => Ok(Type::Printable),
                (Type::Printable, OpType::Plus, Type::Int(_, _)) => Ok(Type::Printable),
                (Type::Printable, OpType::Times, Type::Int(_, _)) => Ok(Type::Printable),
                (Type::Printable, OpType::Plus, Type::Color) => Ok(Type::Printable),
                (Type::Color, OpType::Plus, Type::Printable) => Ok(Type::Printable),
                _ => Err(format!("Operator {} does not operate on ({} x {})", o, t1, t2))
            }
        },
        &Expr::Unop(ref name, ref o) => {
            match context.get(&*name) {
                Some(&(Type::Int(l, h), true)) => Ok(Type::Int(l, h)),
                Some(&(_, false)) => Err(format!("Cannot modify immutable variable {}", name)),
                _ => Err(format!("Unary operator {} can only be used on integers", o))
            }
        },
        &Expr::Call(ref func, ref args) => {
            match func_table.get(&*func) {
                None => Err(format!("Function {} used without declaration", func)),
                Some(&(ref ret_type, ref arg_format)) => {
                    if arg_format.len() != args.len() {
                        return Err(format!("Function {} expects {} arguments but received {}", func, arg_format.len(), args.len()))
                    }
                    let types : Vec<_> = args.iter().map(|e| check_expr(e, func_table, context)).collect();
                                                         //
                    // Make sure all arguments typecheck
                    match types.iter().find(|&ref x| x.is_err()) {
                        Some(e) => return e.clone(),
                        None => ()
                    }

                    if !types.iter().zip(arg_format.iter()).all(|(t1, t2)| subtype(t2, &t1.clone().unwrap())) {
                       return Err(format!("Function {} was passed arguments of incorrect types", func))
                    }

                    Ok(*ret_type)
                }
            }
        },
        &Expr::Elem(ref list, ref index) => {
            let t1 = try!(check_expr(list, func_table, context));
            let t2 = try!(check_expr(index, func_table, context));

            match (t1, t2) {
                (Type::PrintableList(ref i), Type::Int(_, ref h)) => {
                    if i < h {
                        Err(format!("Integer type {} is too large to index into {}", t2, t1))
                    } else {
                        Ok(Type::Printable)
                    }
                }
                (Type::PrintableList(_), _) => Err("Lists may only be indexed by integers".to_string()),
                _ => Err("Only lists can be indexed into".to_string())
            }
        }
    }
}

fn check_statement(s : &Statement, func_table : &FunctionContext, context : &mut VariableContext) -> Result<Option<Type>, String> {
    match s {
        &Statement::Mutable(ref t, ref var, ref val) => {
            if context.contains_key(&*var) {
                return Err(format!("Duplicated definition of {}", var));
            }

            let expr_type = try!(check_expr(val, func_table, &context));
            if !subtype(t, &expr_type) {
                return Err(format!("Assignment to {} must have type {}", var, t));
            }

            context.insert(var.clone(), (*t, true));
            return Ok(None)
        },
        &Statement::Const(ref t, ref var, ref val) => {
            if context.contains_key(&*var) {
                return Err(format!("Duplicated definition of {}", var));
            }

            let expr_type = try!(check_expr(val, func_table, &context));
            if !subtype(t, &expr_type) {
                return Err(format!("Assignment to {} must have type {}", var, t));
            }

            context.insert(var.clone(), (*t, false));
            return Ok(None)
        },
        &Statement::Assign(ref var, ref val) => {
            match context.get(&*var) {
                Some(&(_, false)) => return Err(format!("Attempted to modify immutable variable {}", var)),
                None => return Err(format!("Undeclared variable {}", var)),
                Some(&(ref t, _)) => {
                    let expr_type = try!(check_expr(val, func_table, &context));
                    if !subtype(t, &expr_type) {
                        return Err(format!("Assignment to {} must have type {}", var, t));
                    }
                }
            }

            return Ok(None)
        },
        &Statement::Block(ref stmts) => {
            // TODO: scoped contexts need to occur here

            // TODO: match return types in this function rather than check_function
            for stmt in stmts.iter() {
                match check_statement(stmt, func_table, context) {
                    Ok(Some(x)) => return Ok(Some(x)),
                    Err(s) => return Err(s),
                    _ => ()
                }
            }

            return Ok(None);
        },
        &Statement::Loop(ref stmt) => {
            return check_statement(&*stmt, func_table, context);
        },
        &Statement::Break => {
            return Ok(None);
        },
        &Statement::Input(ref branches) => {
            let mut keys = HashSet::new();

            for &(ref key, ref arm) in branches.iter() {
                if !keys.insert(key) {
                    return Err(format!("Duplicated branch {:?}", key));
                }

                // TODO: Check arm returns
                try!(check_statement(&*arm, func_table, context));
            }
            return Ok(None);
        },
        &Statement::Return(ref expr) => {
            return Ok(Some(try!(check_expr(&*expr, func_table, context))));
        },
        &Statement::Expr(ref expr) => {
            try!(check_expr(&*expr, func_table, context));
            return Ok(None);
        }
    }
    panic!("Internal Error")
}

fn check_function(f : &Function, func_table : &FunctionContext) -> Result<(), String> {
    let mut context = VariableContext::new();

    // Add the local variables
    for &(ref t, ref var) in f.args.iter() {
        context.insert(var.clone(), (*t, false));
    }

    // Check that statements are fine and the function always returns correctly
    match (check_statement(&f.body, func_table, &mut context), &f.ret) {
        (Err(s), _) => return Err(s),
        (Ok(Some(_)), &None) => return Err(format!("Function {} does not return", f.name)),
        (Ok(None), &Some(_)) => return Err(format!("Function {} returns unexpectedly", f.name)),
        (Ok(None), &None) => return Ok(()),
        (Ok(Some(ref t1)), &Some(ref t2)) => {
            if !subtype(t2, t1) {
                return Err(format!("Function {} returned {} when {} was expected", f.name, t1, t2));
            }
        }
    }

    return Ok(());
}

fn parse_function(f : &Function, func_table : &mut FunctionContext) -> Result<(), String> {
    if func_table.contains_key(&f.name) {
        return Err(format!("Function {} is already defined", f.name));
    }

    let ret_type = if let Some(t) = f.ret { t } else { Type::Bottom };
    func_table.insert(f.name.clone(), (ret_type, f.args.iter().map(|&(ref x, _)| *x).collect()));
    return Ok(());
}

pub fn check(t : AST) -> Result<(), String> {
    let mut func_table = FunctionContext::new();

    for builtin in vec!["say", "say_student"] {
        func_table.insert(builtin.to_string(), (Type::Bottom, vec![Type::Printable]));
    }

    // Build the function table to allow forward references
    for func in &t.0 {
        try!(parse_function(func, &mut func_table));
    }

    for func in &t.0 {
        try!(check_function(func, &func_table));
    }

    return Ok(());
}
