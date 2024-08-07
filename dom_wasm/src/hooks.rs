use dom_core::{
    environment::{BuiltinFn, Env, Val},
    interpreter::{Interpreter, ModuleHook, UseHook},
};
use dom_std::StdModule;

use std::sync::{Arc, Mutex};

use miette::Result;
use web_sys::console;

#[derive(Default)]
pub struct WasmUseHook;

impl UseHook for WasmUseHook {
    fn eval_use(&self, _: &Interpreter, _: String, _: &Arc<Mutex<Env>>) -> Result<Option<()>> {
        Ok(Some(()))
    }
}

#[derive(Default)]
pub struct WasmModuleHook;

impl ModuleHook for WasmModuleHook {
    fn use_module(&self, path: String, env: &Arc<Mutex<Env>>) -> Option<()> {
        if path == "std/io" {
            env.lock().unwrap().register_builtin::<PrintFn>("io");

            return Some(());
        }

        StdModule.use_module(path, env)
    }
}

#[derive(Debug, Default)]
pub struct PrintFn;

impl BuiltinFn for PrintFn {
    fn name(&self) -> &str {
        "print"
    }

    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let joined = args.iter().fold(String::new(), |mut output, arg| {
            output.push_str(&format!("{arg}"));
            output
        });

        console::log_1(&joined.into());

        None
    }
}
