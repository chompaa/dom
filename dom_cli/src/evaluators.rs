use dom_core::{Env, Interpreter, Parser, UseEvaluator, ValKind};

use std::{
    fs::read_to_string,
    sync::{Arc, Mutex},
};

use miette::Result;

pub struct CliUseEvaluator;

impl UseEvaluator for CliUseEvaluator {
    fn eval_use(
        &self,
        interpreter: &Interpreter,
        path: String,
        env: &Arc<Mutex<Env>>,
    ) -> Result<()> {
        // Modules are identified using the last name later, e.g.
        //
        // ```
        // use foo/bar
        // bar.call()
        // ```
        let ident = path.split('/').last().unwrap();
        let source = read_to_string(format!(".{}.dom", &path)).unwrap();

        let program = Parser::new(source.to_string()).produce_ast()?;

        let mut env = env.lock().unwrap();
        let mod_env = Env::with_builtins(Arc::clone(env.builtins()));

        let _ = interpreter.eval(program, &mod_env);

        env.declare_unchecked(ident.to_string(), ValKind::Mod(mod_env).into());

        Ok(())
    }
}
