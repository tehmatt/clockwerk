use std::collections::HashMap;

use ast::*;

type FunctionContext = HashMap<Ident, (Option<Type>, Vec<Type>)>;
type VariableContext = HashMap<Ident, (Type, bool)>;

fn check_expr(e : &Expr, func_table : &FunctionContext, context : &VariableContext) -> Result<Type, String> {
    return Ok(Type::Printable);
}

fn check_statement(s : &Statement, func_table : &FunctionContext, context : &mut VariableContext) -> Result<Option<Type>, String> {
    match s {
        &Statement::Mutable(ref t, ref var, ref val) => {
            if context.contains_key(&*var) {
                return Err(format!("Duplicated definition of {}", var));
            }

            let expr_type = try!(check_expr(val, func_table, &context));
            if expr_type != *t {
                return Err(format!("Assignment to {} must have type {}", var, t));
            }

            context.insert(var.clone(), (t.clone(), true));
            return Ok(None)
        },
        &Statement::Const(ref t, ref var, ref val) => {
            if context.contains_key(&*var) {
                return Err(format!("Duplicated definition of {}", var));
            }

            let expr_type = try!(check_expr(val, func_table, &context));
            if expr_type != *t {
                return Err(format!("Assignment to {} must have type {}", var, t));
            }

            context.insert(var.clone(), (t.clone(), false));
            return Ok(None)
        },
        &Statement::Assign(ref var, ref val) => {
            match context.get(&*var) {
                Some(&(_, false)) => return Err(format!("Attempted to modify immutable variable {}", var)),
                None => return Err(format!("Undeclared variable {}", var)),
                Some(&(ref t, _)) => {
                    let expr_type = try!(check_expr(val, func_table, &context));
                    if expr_type != *t {
                        return Err(format!("Assignment to {} must have type {}", var, t));
                    }
                }
            }

            return Ok(None)
        },
        &Statement::Block(ref stmts) => {
            // TODO: ensure all returns in a block have the same type
            for stmt in stmts.iter() {
                match check_statement(stmt, func_table, context) {
                    Ok(Some(x)) => return Ok(Some(x)),
                    Err(s) => return Err(s),
                    _ => ()
                }
            }
        },


        &Statement::Return(ref expr) => {
        }
        _ => ()
    }
    return Err("woops".to_string())
}

fn check_function(f : &Function, func_table : &FunctionContext) -> Result<(), String> {
    let mut context = VariableContext::new();

    // Add the local variables
    for &(ref t, ref var) in f.args.iter() {
        context.insert(var.clone(), (t.clone(), false));
    }

    // Check that statements are fine and the function always returns correctly
    match (check_statement(&f.body, func_table, &mut context), &f.ret) {
        (Err(s), _) => return Err(s),
        (Ok(Some(_)), &None) => return Err(format!("Function {} does not return", f.name)),
        (Ok(None), &Some(_)) => return Err(format!("Function {} returns unexpectedly", f.name)),
        (Ok(None), &None) => return Ok(()),
        (Ok(Some(ref t1)), &Some(ref t2)) => {
            if *t1 != *t2 {
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

    func_table.insert(f.name.clone(), (f.ret.clone(), f.args.iter().map(|&(ref x, _)| x.clone()).collect()));
    return Ok(());
}

pub fn check(t : AST) -> Result<(), String> {
    let mut func_table = FunctionContext::new();

    // Build the function table to allow forward references
    for func in &t.0 {
        try!(parse_function(func, &mut func_table));
    }

    for func in &t.0 {
        try!(check_function(func, &func_table));
    }

    return Ok(());
}
