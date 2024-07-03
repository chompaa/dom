mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;
pub mod std;
mod util;

pub use ast::Expr;
pub use environment::{Env, Val, ValKind};
pub use interpreter::{Interpreter, UseEvaluator};
pub use parser::Parser;

#[macro_export]
macro_rules! declare_native_func {
    ($env:expr, $func:path) => {
        $env.lock().unwrap().declare_unchecked(
            stringify!($func)
                .split("::")
                .last()
                .expect("function path should contain `::`")
                .to_string(),
            ValKind::NativeFunc(Box::new($func)).into(),
        );
    };
}
