use dom_macros::expected_args;

use super::*;

#[derive(Debug, Default)]
pub struct GetFn;

impl BuiltinFn for GetFn {
    fn name(&self) -> &str {
        "get"
    }

    #[expected_args(List(list), Int(index))]
    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
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

    #[expected_args(List(list), Int(index), Val(value))]
    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let mut list = list.clone();

        let index = index.to_wrapped_index(list.len());
        list[index] = value.clone();

        Some(list.into())
    }
}

#[derive(Debug, Default)]
pub struct PushFn;

impl BuiltinFn for PushFn {
    fn name(&self) -> &str {
        "push"
    }

    #[expected_args(List(list), Val(value))]
    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let mut list = list.clone();

        list.push(value.clone());

        Some(list.into())
    }
}

#[derive(Debug, Default)]
pub struct PopFn;

impl BuiltinFn for PopFn {
    fn name(&self) -> &str {
        "pop"
    }

    #[expected_args(List(list), Int(index))]
    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
        let mut list = list.clone();

        let index = index.to_wrapped_index(list.len());
        list.remove(index);

        Some(list.into())
    }
}

#[derive(Debug, Default)]
pub struct LenFn;

impl BuiltinFn for LenFn {
    fn name(&self) -> &str {
        "len"
    }

    #[expected_args(List(list))]
    fn run(&self, args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
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
