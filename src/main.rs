#[macro_use]
extern crate nom;

extern crate ansi_term;
extern crate getopts;
use getopts::Options;

use std::io::prelude::*;
use std::fs::File;
use std::env;

mod ast;
mod parser;
mod typechecker;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn initialize_args() -> Options {
    let mut opts = Options::new();

    opts.optopt("O", "", "set optimization level", "[0-3]");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "ast", "print the ast");

    return opts;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let opts = initialize_args();

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let print_ast = matches.opt_present("ast");

    let filename = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    let mut input = File::open(filename).unwrap();
    let mut code = String::new();
    input.read_to_string(&mut code).unwrap();

    let ast = match parser::parse(code) {
        Ok(t) => t,
        Err(s) => {
            println!("{}", s);
            return;
        }
    };

    if print_ast {
        println!("{}", ast);
    }

    match typechecker::check(ast) {
        Ok(_) => (),
        Err(s) => {
            println!("Typechecker error: {}", s);
            return;
        }
    }
}
