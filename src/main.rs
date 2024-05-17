mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod util;

use std::io::{self, Write};

use crate::{environment::Env, interpreter::eval, parser::Parser};

fn main() {
    let mut env = Env::new();
    let _ = env.declare("pi".to_string(), 3);

    loop {
        print!(">: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        println!("Input: {}", input);

        let Ok(program) = Parser::new().produce_ast(input.chars().collect::<String>()) else {
            panic!("Error parsing program");
        };

        let Ok(result) = eval(program, &mut env) else {
            panic!("Error evaluating program")
        };

        dbg!(result);
    }
}
