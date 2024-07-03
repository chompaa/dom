mod evaluators;

use dom_core::{declare_native_func, std, Env, Interpreter, Parser, Val, ValKind};

use ::std::{
    fs::read_to_string,
    io::{self, Write},
    sync::{Arc, Mutex},
};

use clap::Parser as _;
use miette::Result;

#[derive(clap::Parser)]
struct Args {
    path: Option<String>,
}

fn result(source: &str, env: &Arc<Mutex<Env>>) -> Result<Val> {
    (|| -> Result<Val> {
        let program = Parser::new(source.to_string()).produce_ast()?;
        let module_evaluator = Box::new(evaluators::CliUseEvaluator);
        Interpreter::new(module_evaluator).eval(program, env)
    })()
    .map_err(|error| error.with_source_code(source.to_string()))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let env = Arc::new(Mutex::new(Env::default()));

    // Set up native functions
    declare_native_func!(env, std::print);
    declare_native_func!(env, std::input);
    declare_native_func!(env, std::get);
    declare_native_func!(env, std::set);
    declare_native_func!(env, std::push);
    declare_native_func!(env, std::pop);
    declare_native_func!(env, std::len);

    match args.path {
        // File mode
        Some(path) => {
            let source = read_to_string(path).expect("should be able to read file from path");
            result(&source, &env).map(|_| ())
        }
        // Interactive mode
        None => loop {
            print!(">: ");

            io::stdout().flush().unwrap();

            let mut source = String::new();
            io::stdin()
                .read_line(&mut source)
                .expect("should be able to read line");

            match result(&source, &env) {
                Ok(result) => print!("{result}"),
                Err(error) => return Err(error),
            }
        },
    }
}
