mod evaluators;
mod std;

use dom_core::{Env, Interpreter, Parser};

use ::std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn init_miette_hook() {
    // This is important since in wasm builds unicode is disabled by default.
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

fn register_builtins(env: &Arc<Mutex<Env>>) {
    env.lock()
        .unwrap()
        .register_builtin::<std::PrintFn>()
        .register_builtin::<std::InputFn>()
        .register_builtin::<std::GetFn>()
        .register_builtin::<std::SetFn>()
        .register_builtin::<std::PushFn>()
        .register_builtin::<std::PopFn>()
        .register_builtin::<std::LenFn>();
}

#[wasm_bindgen]
pub fn interpret(source: &str) -> String {
    let env = Env::new();
    register_builtins(&env);

    let (ast, program) = match Parser::new(source.to_string()).produce_ast() {
        Ok(program) => (format!("{program:#?}"), program),
        Err(error) => {
            let error = error.with_source_code(source.to_string());
            console::log_1(&format!("{error:?}").into());
            return "AST could not be produced".to_string();
        }
    };

    let module_evaluator = Box::new(evaluators::WasmUseEvaluator);
    if let Err(error) = Interpreter::new(module_evaluator).eval(program, &env) {
        let error = error.with_source_code(source.to_string());
        console::log_1(&format!("{error:?}").into());
    }

    ast
}
