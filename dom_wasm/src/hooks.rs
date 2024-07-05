use dom_core::{BuiltinFn, Env, Interpreter, ModuleHook, UseHook, Val};
use dom_std::StdModule;

use std::sync::{Arc, Mutex};

use miette::{Result, SourceSpan};
use web_sys::console;

pub struct WasmUseHook;

impl UseHook for WasmUseHook {
    fn eval_use(
        &self,
        _: &Interpreter,
        _: String,
        _: &Arc<Mutex<Env>>,
        _: SourceSpan,
    ) -> Result<()> {
        Ok(())
    }
}

pub struct WasmModuleHook;

impl ModuleHook for WasmModuleHook {
    fn use_module(&self, path: String, env: &Arc<Mutex<Env>>) -> Result<Option<()>> {
        if path == "std/io" {
            env.lock().unwrap().register_builtin::<PrintFn>("io");

            return Ok(Some(()));
        }

        StdModule::default().use_module(path, env)
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
