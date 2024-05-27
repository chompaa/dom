use std::fmt::Write as _;

use dom::{eval, Env, Parser, Val};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn interpret(contents: &str) {
    let mut env = Env::default();

    let _ = env.declare(
        "print".to_owned(),
        Val::NativeFunc(Box::new(|args, _| {
            let joined = args.iter().fold(String::new(), |mut output, arg| {
                let _ = write!(output, "{arg} ");
                output
            });
            console::log_1(&joined.into());
            None
        })),
    );

    let mut parser = Parser::new();
    match parser.produce_ast(contents.to_string()) {
        Ok(program) => {
            let _ = eval(program, &mut env);
        }
        Err(reason) => {
            console::log_1(&format!("[L{}] {reason}", parser.line()).into());
        }
    };
}
