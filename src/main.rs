#[macro_use]
extern crate nom;

mod ast;
mod parser;

use parser::parse;

fn main() {
    let test1 =
"main() {
    string x = \"hello\";
    list<int<0, 6>> x = 10;
    loop {
        list<int> x = 6;
        string x = 5+( 6+7+8)+9;

        input {
            W => { }
            A => {
                loop {
                    list<list<list<string>>> whatabigtype = 10;
                    whatabigtype = 4;
                }
            }
        }
    }
}";
    match parse(test1) {
        Ok(t) => println!("{:?}", t),
        Err(s) => println!("{}", s)
    }
}
