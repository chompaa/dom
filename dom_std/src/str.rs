use super::*;

#[derive(Debug, Default)]
pub struct LenFn;

impl BuiltinFn for LenFn {
    fn name(&self) -> &str {
        "len"
    }

    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let [Val {
            kind: ValKind::Str(str),
            ..
        }] = &args[..1]
        else {
            return None;
        };

        let value = ValKind::Int(str.len() as i32);
        Some(value.into())
    }
}
