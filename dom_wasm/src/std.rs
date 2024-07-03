pub use dom_core::std::*;
use dom_core::{BuiltinFn, Env, Val};

use ::std::sync::{Arc, Mutex};

use web_sys::console;

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
