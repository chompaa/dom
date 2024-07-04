use dom_core::{Interpreter, UseEvaluator};

pub struct WasmUseEvaluator;

impl UseEvaluator for WasmUseEvaluator {
    fn eval_use(
        &self,
        _: &Interpreter,
        _: String,
        _: &std::sync::Arc<std::sync::Mutex<dom_core::Env>>,
        _: miette::SourceSpan,
    ) -> miette::Result<()> {
        Ok(())
    }
}
