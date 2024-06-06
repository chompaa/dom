use std::fmt::Write as _;

use dom::{eval, Env, Parser, Val};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn interpret(contents: &str) -> String {
    let env = Env::new();

    env.borrow_mut()
        .declare(
            "print".to_owned(),
            Val::NativeFunc(Box::new(|args, _| {
                let joined = args.iter().fold(String::new(), |mut output, arg| {
                    let _ = write!(output, "{arg} ");
                    output
                });
                console::log_1(&joined.trim().into());
                None
            })),
        )
        .expect("should be able to declare `print` function");

    let mut parser = Parser::new();
    match parser.produce_ast(contents.to_string()) {
        Ok(program) => {
            let ast = format!("{:#?}", program);
            let _ = eval(program, &env);
            return ast;
        }
        Err(reason) => {
            console::log_1(&format!("[L{}] {reason}", parser.line()).into());
            return "AST could not be produced".to_string();
        }
    };
}
