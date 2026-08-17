//! Core standard library.

use crate::env::Env;
use crate::error::{Error, Result};
use crate::expr::{Expr, Pair};

pub fn init_core(env: &mut Env) -> Result<()> {
    env.bind_native_func("assert", ext_assert)?;
    env.bind_native_func("assert-eq", ext_assert_eq)?;
    env.bind_native_func("display", display)?;
    env.bind_native_func("newline", newline)?;

    env.bind_native_func("number?", number_is_number)?;
    env.bind_native_func("+", number_add)?;
    env.bind_native_func("-", number_sub)?;
    env.bind_native_func("*", number_mul)?;
    env.bind_native_func("=", number_eq)?;
    env.bind_native_func("<", number_lt)?;
    env.bind_native_func(">", number_gt)?;
    env.bind_native_func("<=", number_lt_eq)?;
    env.bind_native_func(">=", number_gt_eq)?;

    env.bind_native_func("boolean?", boolean_is_boolean)?;
    env.bind_native_func("not", boolean_not)?;
    env.bind_native_func("and", boolean_and)?;
    env.bind_native_func("or", boolean_or)?;

    env.bind_native_func("pair?", pair_is_pair)?;
    env.bind_native_func("list?", pair_is_list)?;
    env.bind_native_func("list", pair_list)?;
    env.bind_native_func("cons", pair_cons)?;
    env.bind_native_func("car", pair_car)?;
    env.bind_native_func("cdr", pair_cdr)?;

    Ok(())
}

macro_rules! unexpected_type {
    ($expected:expr) => {
        Err(Error::Reason(format!(
            "expected argument to be a {}",
            $expected
        )))
    };
}

macro_rules! wrong_arg_count {
    () => {
        Err(Error::Reason(format!(
            "[{}:{}] wrong number of arguments passed to procedure",
            file!(),
            line!()
        )))
    };
}

fn args1(args: &[Expr]) -> Result<&Expr> {
    match args {
        [arg1] => Ok(arg1),
        [..] => wrong_arg_count!(),
    }
}

fn args2(args: &[Expr]) -> Result<[&Expr; 2]> {
    match args {
        [arg1, arg2] => Ok([arg1, arg2]),
        [..] => wrong_arg_count!(),
    }
}

fn args2_numbers(args: &[Expr]) -> Result<[f64; 2]> {
    // println!("args2_numbers({:?})", args);
    match args {
        [Expr::Number(arg1), Expr::Number(arg2)] => Ok([*arg1, *arg2]),
        [..] => wrong_arg_count!(),
    }
}

/// There is no assert in Scheme. This is our own extension to assist with unit testing.
///
/// This must move to a library once they're implemented.
///
/// ```scheme
/// (assert <expr> <message>?)
/// ```
fn ext_assert(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let expr = args
        .get(0)
        .ok_or_else(|| Error::Reason("expected assertion expression".to_string()))?;
    let msg = args.get(1); // optional

    if let Expr::Bool(false) = expr {
        match msg {
            Some(Expr::String(message)) => {
                Err(Error::Reason(format!("assertion error: {message}")))
            }
            Some(_) => {
                // TODO: to_string solution that's cogent with Scheme's specification.
                Err(Error::Reason("invalid assertion message type".to_string()))
            }
            None => Err(Error::Reason(format!("assertion failed: {expr:?}"))),
        }
    } else {
        Ok(expr.clone())
    }
}

fn ext_assert_eq(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let [arg1, arg2] = args2(args)?;
    if arg1 == arg2 {
        Ok(Expr::List(vec![arg1.clone(), arg2.clone()]))
    } else {
        Err(Error::Reason(format!(
            "assertion failed: {} == {}",
            arg1.repr(),
            arg2.repr()
        )))
    }
}

fn display(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let arg0 = args1(args)?;
    let repr = arg0.repr();

    print!("{repr}");

    Ok(Expr::Void)
}

fn newline(_env: &mut Env, _args: &[Expr]) -> Result<Expr> {
    // TODO: Output port argument
    println!();

    Ok(Expr::Void)
}

// ----------------------------------------------------------------------------
// Number

fn number_is_number(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let arg0 = args.get(0).ok_or_else(|| {
        Error::Reason("wrong number of arguments passed to procedure".to_string())
    })?;
    Ok(Expr::Bool(arg0.is_number()))
}

fn number_add(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = 0.0;

    for (index, arg) in args.iter().enumerate() {
        match arg {
            Expr::Number(number) => sum += number,
            _ => {
                return Err(Error::Reason(format!(
                    "expected argument {index} to be a number, but encountered {arg:?}"
                )))
            }
        }
    }

    println!("number_add -> {sum}");
    Ok(Expr::Number(sum))
}

fn number_sub(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = args
        .get(0)
        .ok_or_else(|| Error::Reason("wrong number of arguments passed to procedure".to_string()))?
        .as_number()
        .ok_or_else(|| {
            Error::Reason(format!(
                "expected first argument to be a number, but encountered {:?}",
                &args[0]
            ))
        })?;

    let rest = &args[1..];

    for (index, arg) in rest.iter().enumerate() {
        // println!("arg [{index}] {arg:?}; sum -> {sum}");
        match arg {
            Expr::Number(number) => sum -= number,
            _ => {
                return Err(Error::Reason(format!(
                    "expected argument {index} to be a number, but encountered {arg:?}"
                )))
            }
        }
    }

    // println!("number_sub -> {sum}");
    Ok(Expr::Number(sum))
}

fn number_mul(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = 1.0;

    for (index, arg) in args.iter().enumerate() {
        match arg {
            Expr::Number(number) => sum *= number,
            _ => {
                return Err(Error::Reason(format!(
                    "expected argument {index} to be a number, but encountered {arg:?}"
                )))
            }
        }
    }

    Ok(Expr::Number(sum))
}

// TODO: Does this short circuit, or always evaluate all arguments?
fn number_eq(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    for ab in args.windows(2) {
        match ab {
            [] | [_] => break,
            [Expr::Number(a), Expr::Number(b), ..] => {
                if a != b {
                    return Ok(Expr::Bool(false));
                }
            }
            [..] => {
                return Err(Error::Reason(
                    "expected argument to be a number".to_string(),
                ));
            }
        }
    }

    Ok(Expr::Bool(true))
}

fn number_lt(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let [arg1, arg2] = args2_numbers(args)?;
    Ok(Expr::Bool(arg1 < arg2))
}

fn number_gt(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let [arg1, arg2] = args2_numbers(args)?;
    Ok(Expr::Bool(arg1 > arg2))
}

fn number_lt_eq(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    // println!("number_lt_eq({:?})", args);
    let [arg1, arg2] = args2_numbers(args)?;
    Ok(Expr::Bool(arg1 <= arg2))
}

fn number_gt_eq(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let [arg1, arg2] = args2_numbers(args)?;
    Ok(Expr::Bool(arg1 >= arg2))
}

// ----------------------------------------------------------------------------
// Boolean

fn boolean_is_boolean(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let arg0 = args.get(0).ok_or_else(|| {
        Error::Reason("wrong number of arguments passed to procedure".to_string())
    })?;
    Ok(Expr::Bool(arg0.is_boolean()))
}

fn boolean_not(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    if args.len() > 1 {
        Err(Error::Reason(
            "wrong number of arguments passed to procedure".to_string(),
        ))
    } else {
        let arg0 = &args[0];
        if let Expr::Bool(false) = arg0 {
            Ok(Expr::Bool(true))
        } else {
            Ok(Expr::Bool(false))
        }
    }
}

fn boolean_and(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    // Default return value if procedure has no arguments.
    let mut expr = &Expr::Bool(true);

    for arg in args.iter() {
        if let Expr::Bool(false) = arg {
            // If any #f is encountered, return early.
            return Ok(Expr::Bool(false));
        } else {
            // Storing reference to argument to avoid cloning on each iteration.
            expr = arg;
        }
    }

    Ok(expr.clone())
}

fn boolean_or(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    todo!()
}

// ----------------------------------------------------------------------------
// Pair

/// Checks if the given argument is a pair.
fn pair_is_pair(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let arg1 = args1(args)?;
    Ok(Expr::Bool(matches!(arg1, Expr::Pair(_))))
}

fn pair_is_list(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let arg1 = args1(args)?;
    Ok(Expr::Bool(Pair::is_list(arg1)))
}

/// Creates a new valid list from the given arguments.
fn pair_list(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    // Nil is the empty list '()
    Ok(Pair::new_list(args)
        .map(|pair| pair.to_expr())
        .unwrap_or(Expr::Nil))
}

/// Creates a pair from two arguments.
fn pair_cons(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let [arg1, arg2] = args2(args)?;
    Ok(Pair(arg1.clone(), arg2.clone()).to_expr())
}

/// Retrieve the first element of a pair.
fn pair_car(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let arg1 = args1(args)?;

    match arg1 {
        Expr::Pair(pair_handle) => Ok(pair_handle.borrow().0.clone()),
        _ => unexpected_type!("pair"),
    }
}

/// Retrieve the second element of a pair.
fn pair_cdr(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let arg1 = args1(args)?;

    match arg1 {
        Expr::Pair(pair_handle) => Ok(pair_handle.borrow().1.clone()),
        _ => unexpected_type!("pair"),
    }
}
