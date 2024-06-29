use crate::{Env, Val};

use ::std::{
    fmt::Write as _,
    sync::{Arc, Mutex},
};

use web_sys::console;

pub fn print(args: Vec<Val>, _: Arc<Mutex<Env>>) -> Option<Val> {
    let joined = args.iter().fold(String::new(), |mut output, arg| {
        let _ = write!(output, "{arg} ");
        output
    });

    console::log_1(&joined.into());

    None
}
