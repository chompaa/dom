mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod util;

use environment::{Env, Val};
use interpreter::eval;
use parser::Parser;

use std::{
    cell::RefCell,
    fmt::Write as _,
    fs::read_to_string,
    io::{self, Write},
    rc::Rc,
};

use clap::Parser as _;

#[derive(clap::Parser)]
struct Args {
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    let env = Rc::new(RefCell::new(Env::default()));

    // TODO: Refactor `Env`
    let _ = env.borrow_mut().declare(
        "print".to_owned(),
        Val::NativeFunc(Box::new(|args, _| {
            let joined = args.iter().fold(String::new(), |mut output, arg| {
                let _ = write!(output, "{arg} ");
                output
            });

            println!("{}", &joined);

            None
        })),
    );

    let result = |contents: String| {
        let mut parser = Parser::new(contents);
        let program = match parser.produce_ast() {
            Ok(program) => program,
            Err(reason) => {
                panic!("{}", reason.to_report());
            }
        };
        eval(program, &env)
    };

    if let Some(path) = args.path {
        let contents = read_to_string(path).expect("Could not read file from specified path");
        let _ = result(contents);
    } else {
        loop {
            print!(">: ");

            io::stdout().flush().unwrap();

            let mut contents = String::new();
            io::stdin()
                .read_line(&mut contents)
                .expect("Failed to read input");

            match result(contents) {
                Ok(result) => print!("{result}"),
                Err(reason) => println!("{reason}"),
            }
        }
    }
}
