use dom_macros::expected_args;

use super::*;

#[derive(Debug, Default)]
pub struct LenFn;

impl BuiltinFn for LenFn {
    fn name(&self) -> &str {
        "len"
    }

    #[expected_args(Str(string))]
    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let value = ValKind::Int(string.len() as i32);
        Some(value.into())
    }
}
