mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod util;

use crate::{environment::Env, interpreter::eval, parser::Parser};

use std::{
    fmt::Write as _,
    fs::read_to_string,
    io::{self, Write},
};

use clap::Parser as _;
use environment::Val;

#[derive(clap::Parser)]
struct Args {
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut env = Env::default();

    // TODO: Refactor `Env`
    let _ = env.declare(
        "print".to_owned(),
        Val::NativeFunc(Box::new(|args, _| {
            let joined = args.iter().fold(String::new(), |mut output, arg| {
                let _ = write!(output, "{arg} ");
                output
            });

            println!("{}", &joined.trim());

            None
        })),
    );

    let mut result = |contents: String| {
        let mut parser = Parser::new();
        let program = match parser.produce_ast(contents) {
            Ok(program) => program,
            Err(reason) => panic!("[L{}] {reason}", parser.line()),
        };
        eval(program, &mut env)
    };

    if let Some(path) = args.path {
        let contents = read_to_string(path).expect("Could not read file from specified path");
        if let Err(reason) = result(contents) {
            println!("{reason}");
        }
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
