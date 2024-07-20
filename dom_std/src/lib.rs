mod list;
mod str;

use dom_core::{
    environment::{BuiltinFn, Env, Val, ValKind},
    interpreter::ModuleHook,
};

use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct StdModule;

impl ModuleHook for StdModule {
    fn use_module(&self, path: String, env: &Arc<Mutex<Env>>) -> Option<()> {
        let mut path = path.split('/');

        let Some("std") = path.next() else {
            return None;
        };

        let mut env = env.lock().unwrap();

        match path.next() {
            Some("list") => {
                env.register_builtin::<list::GetFn>("list")
                    .register_builtin::<list::SetFn>("list")
                    .register_builtin::<list::PushFn>("list")
                    .register_builtin::<list::PopFn>("list")
                    .register_builtin::<list::LenFn>("list");
            }
            Some("str") => {
                env.register_builtin::<str::LenFn>("str");
            }
            Some(_) | None => return None,
        };

        Some(())
    }
}
