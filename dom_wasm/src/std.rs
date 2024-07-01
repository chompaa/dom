use dom_core::{Env, Val};

pub use dom_core::std::*;

use ::std::{
    fmt::Write as _,
    sync::{Arc, Mutex},
};

use web_sys::console;

pub fn print(args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
    let joined = args.iter().fold(String::new(), |mut output, arg| {
        let _ = write!(output, "{arg} ");
        output
    });

    console::log_1(&joined.into());

    None
}
