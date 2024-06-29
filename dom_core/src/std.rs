use crate::{Env, Val};

use ::std::{
    fmt::Write as _,
    io::{self, Write},
    sync::{Arc, Mutex},
};

pub fn print(args: Vec<Val>, _: Arc<Mutex<Env>>) -> Option<Val> {
    let joined = args.iter().fold(String::new(), |mut output, arg| {
        let _ = write!(output, "{arg} ");
        output
    });

    println!("{}", &joined);

    None
}

pub fn input(_: Vec<Val>, _: Arc<Mutex<Env>>) -> Option<Val> {
    io::stdout().flush().unwrap();

    // Retrieve input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("should be able to read line");

    // Remove `\n` from `read_line`
    let input = input.trim_end_matches('\n').to_string();

    Some(Val::Str(input))
}
