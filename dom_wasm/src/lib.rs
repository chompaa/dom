mod hooks;

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

#[wasm_bindgen]
pub fn interpret(source: &str) -> String {
    let env = Env::new();

    let (ast, program) = match Parser::new(source.to_string()).produce_ast() {
        Ok(program) => (format!("{program:#?}"), program),
        Err(error) => {
            let error = error.with_source_code(source.to_string());
            console::log_1(&format!("{error:?}").into());
            return "AST could not be produced".to_string();
        }
    };

    let use_hook = Box::new(hooks::WasmUseHook);
    let module_hook = Box::new(hooks::WasmModuleHook);

    if let Err(error) = Interpreter::new(use_hook, module_hook).eval(program, &env) {
        let error = error.with_source_code(source.to_string());
        console::log_1(&format!("{error:?}").into());
    }

    ast
}
