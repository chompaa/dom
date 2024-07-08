use super::*;

#[derive(Debug, Default)]
pub struct GetFn;

impl BuiltinFn for GetFn {
    fn name(&self) -> &str {
        "get"
    }

    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let [Val {
            kind: ValKind::List(list),
            ..
        }, Val {
            kind: ValKind::Int(index),
            ..
        }] = &args[..2]
        else {
            return None;
        };

        let index = index.to_wrapped_index(list.len());
        list.get(index).cloned()
    }
}

#[derive(Debug, Default)]
pub struct SetFn;

impl BuiltinFn for SetFn {
    fn name(&self) -> &str {
        "set"
    }

    fn run(&self, args: &[Val], env: &Arc<Mutex<Env>>) -> Option<Val> {
        let [Val {
            ident,
            kind: ValKind::List(list),
        }, Val {
            kind: ValKind::Int(index),
            ..
        }, value] = &args[..3]
        else {
            return None;
        };

        let mut list = list.clone();
        let index = index.to_wrapped_index(list.len());
        list[index] = value.clone();

        let Some(ident) = ident else {
            return Some(list.into());
        };

        Some(Env::assign_unchecked(env, ident, list.into()))
    }
}

#[derive(Debug, Default)]
pub struct PushFn;

impl BuiltinFn for PushFn {
    fn name(&self) -> &str {
        "push"
    }

    fn run(&self, args: &[Val], env: &Arc<Mutex<Env>>) -> Option<Val> {
        let [Val {
            ident,
            kind: ValKind::List(list),
        }, value] = &args[..2]
        else {
            return None;
        };

        let mut list = list.clone();
        list.push(value.clone());

        let Some(ident) = ident else {
            return Some(list.into());
        };

        Some(Env::assign_unchecked(env, ident, list.into()))
    }
}

#[derive(Debug, Default)]
pub struct PopFn;

impl BuiltinFn for PopFn {
    fn name(&self) -> &str {
        "pop"
    }

    fn run(&self, args: &[Val], env: &Arc<Mutex<Env>>) -> Option<Val> {
        let [Val {
            ident,
            kind: ValKind::List(list),
        }, Val {
            kind: ValKind::Int(index),
            ..
        }] = &args[..2]
        else {
            return None;
        };

        let mut list = list.clone();
        let index = index.to_wrapped_index(list.len());
        list.remove(index);

        let Some(ident) = ident else {
            return Some(list.into());
        };

        Some(Env::assign_unchecked(env, ident, list.into()))
    }
}

#[derive(Debug, Default)]
pub struct LenFn;

impl BuiltinFn for LenFn {
    fn name(&self) -> &str {
        "len"
    }

    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let Val {
            kind: ValKind::List(list),
            ..
        } = &args[0]
        else {
            return None;
        };

        let len = list.len();

        Some(ValKind::Int(len as i32).into())
    }
}

trait Int32Ext {
    fn to_wrapped_index(&self, len: usize) -> usize;
}

impl Int32Ext for i32 {
    fn to_wrapped_index(&self, len: usize) -> usize {
        let len = i32::try_from(len).expect("should be able to cast index to `i32`");
        let index = (len + self) % len;
        index as usize
    }
}