use std::fmt::Write as _;

use dom::{eval, Env, Parser, Val};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn set_hook() {
    // This is important since in wasm builds, unicode will be disabled by default.
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(false)
                .unicode(true)
                .color(true)
                .build(),
        )
    }));
}

#[wasm_bindgen]
pub fn interpret(source: &str) -> String {
    let env = Env::new();

    env.borrow_mut()
        .declare(
            "print".to_owned(),
            Val::NativeFunc(Box::new(|args, _| {
                let joined = args.iter().fold(String::new(), |mut output, arg| {
                    let _ = write!(output, "{arg} ");
                    output
                });
                console::log_1(&joined.into());
                None
            })),
        )
        .expect("should be able to declare `print` function");

    let mut parser = Parser::new(source.to_string());

    match parser.produce_ast() {
        Ok(program) => {
            let ast = format!("{:#?}", program);
            let _ = eval(program, &env);
            return ast;
        }
        Err(reason) => {
            console::log_1(&reason.to_report().into());
            return "AST could not be produced".to_string();
        }
    };
}
