use crate::{
    environment::{Val, ValKind},
    Env,
};

use ::std::{
    fmt::Write as _,
    io::{self, Write},
    sync::{Arc, Mutex},
};

#[must_use]
pub fn print(args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
    let joined = args.iter().fold(String::new(), |mut output, arg| {
        let _ = write!(output, "{arg} ");
        output
    });

    println!("{}", &joined);

    None
}

#[must_use]
pub fn input(_: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
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

#[must_use]
pub fn get(args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
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

#[must_use]
pub fn set(args: &[Val], env: &Arc<Mutex<Env>>) -> Option<Val> {
    let [Val {
        ident: Some(ident),
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

    Env::assign_unchecked(env, ident.to_string(), ValKind::List(list).into());

    None
}

#[must_use]
pub fn push(args: &[Val], env: &Arc<Mutex<Env>>) -> Option<Val> {
    let [Val {
        ident: Some(ident),
        kind: ValKind::List(list),
    }, value] = &args[..2]
    else {
        return None;
    };

    let mut list = list.clone();
    list.push(value.clone());

    Env::assign_unchecked(env, ident.to_string(), ValKind::List(list).into());

    None
}

#[must_use]
pub fn del(args: &[Val], env: &Arc<Mutex<Env>>) -> Option<Val> {
    let [Val {
        ident: Some(ident),
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

    Env::assign_unchecked(env, ident.to_string(), ValKind::List(list).into());

    None
}

#[must_use]
pub fn len(args: &[Val], _: &Arc<Mutex<Env>>) -> Option<Val> {
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
