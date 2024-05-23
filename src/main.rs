mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod util;

use crate::{environment::Env, interpreter::eval, parser::Parser};

use std::{
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

    let mut env = Env::new();

    let mut result = |contents: String| {
        let mut parser = Parser::new();
        let program = match parser.produce_ast(contents) {
            Ok(program) => program,
            Err(reason) => panic!("[L{}] {}", parser.line(), reason),
        };
        let result = eval(program, &mut env);
        match result {
            Ok(result) => match result {
                Val::Int(number) => format!("{}", number),
                Val::Func {
                    ident,
                    params,
                    body,
                    env,
                } => format!("{ident}, {:?}, {:?}, {:?}", params, body, env),
            },
            Err(reason) => format!("{}", reason),
        }
    };

    if let Some(path) = args.path {
        let contents = read_to_string(path).expect("Could not read file from specified path");
        println!("{}", result(contents))
    } else {
        loop {
            print!(">: ");

            io::stdout().flush().unwrap();

            let mut contents = String::new();
            io::stdin()
                .read_line(&mut contents)
                .expect("Failed to read input");

            println!("Input: {}", contents);

            println!("{}", result(contents))
        }
    }
}
