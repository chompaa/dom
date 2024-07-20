use dom_core::{
    environment::{BuiltinFn, Env, Val, ValKind},
    interpreter::{Interpreter, ModuleHook, UseHook},
    parser::Parser,
};
use dom_std::StdModule;

use std::{
    fs::read_to_string,
    io::{self, Write as _},
    sync::{Arc, Mutex},
};

use miette::Result;

#[derive(Default)]
pub struct CliUseHook;

impl UseHook for CliUseHook {
    fn eval_use(
        &self,
        interpreter: &Interpreter,
        path: String,
        env: &Arc<Mutex<Env>>,
    ) -> Result<Option<()>> {
        // Modules are identified using the last name later, e.g.
        //
        // ```
        // use foo/bar
        // bar.call()
        // ```
        let ident = path.split('/').last().unwrap();
        let Ok(source) = read_to_string(format!("./{}.dom", &path)) else {
            return Ok(None);
        };

        let program = Parser::new(&source).produce_ast()?;

        let mut env = env.lock().unwrap();
        let mod_env = Env::with_builtins(Arc::clone(env.builtins()));

        let _ = interpreter.eval(program, &mod_env);

        env.declare_unchecked(ident, ValKind::Mod(mod_env).into());

        Ok(Some(()))
    }
}

#[derive(Default)]
pub struct CliModuleHook;

impl ModuleHook for CliModuleHook {
    fn use_module(&self, path: String, env: &Arc<Mutex<Env>>) -> Option<()> {
        if path == "std/io" {
            env.lock()
                .unwrap()
                .register_builtin::<PrintFn>("io")
                .register_builtin::<InputFn>("io");

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

        println!("{}", &joined);

        None
    }
}

#[derive(Debug, Default)]
pub struct InputFn;

impl BuiltinFn for InputFn {
    fn name(&self) -> &str {
        "input"
    }

    fn run(&self, _: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        io::stdout().flush().unwrap();

        // Retrieve input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("should be able to read line");

        // Remove `\n` from `read_line`
        let input = input.trim_end_matches('\n').to_string();

        Some(ValKind::Str(input).into())
    }
}
