use std::fmt::Write as _;

use dom::{Env, Interpreter, Parser, Val};
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

    env.lock().unwrap().declare_unchecked(
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

    let (ast, program) = match Parser::new(source.to_string()).produce_ast() {
        Ok(program) => (format!("{program:#?}"), program),
        Err(error) => {
            let error = error.with_source_code(source.to_string());
            console::log_1(&format!("{error:?}").into());
            return "AST could not be produced".to_string();
        }
    };

    if let Err(error) = Interpreter::new().eval(program, &env) {
        let error = error.with_source_code(source.to_string());
        console::log_1(&format!("{error:?}").into());
    }

    return ast;
}
