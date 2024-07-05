mod list;

use dom_core::{BuiltinFn, Env, ModuleHook, Val, ValKind};

use ::std::sync::{Arc, Mutex};

use miette::Result;

#[derive(Default)]
pub struct StdModule;

impl ModuleHook for StdModule {
    fn use_module(&self, path: String, env: &Arc<Mutex<Env>>) -> Result<Option<()>> {
        let mut path = path.split('/');

        let Some("std") = path.next() else {
            return Ok(None);
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
            Some(_) | None => return Ok(None),
        };

        Ok(Some(()))
    }
}
