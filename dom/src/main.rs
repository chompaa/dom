use dom::{Env, Interpreter, Parser, Val};

use std::{
    fmt::Write as _,
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

fn setup_env() -> Arc<Mutex<Env>> {
    let env = Arc::new(Mutex::new(Env::default()));

    env.lock().unwrap().declare_unchecked(
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

    env
}

fn result(source: &str, env: &Arc<Mutex<Env>>) -> Result<Val> {
    (|| -> Result<Val> {
        let program = Parser::new(source.to_string()).produce_ast()?;
        Interpreter::new().eval(program, env)
    })()
    .map_err(|error| error.with_source_code(source.to_string()))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let env = setup_env();

    match args.path {
        Some(path) => {
            let source = read_to_string(path).expect("should be able to read file from path");
            result(&source, &env).map(|_| ())
        }
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
