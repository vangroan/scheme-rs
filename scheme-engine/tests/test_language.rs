//! Aggregated tests for language features, in Scheme files.
//!
//! See scripts in [`./language`]
use scheme_engine::{error::Error, Closure, Env, Expr, Handle};

fn compile_closure_env(source: &str) -> Result<(Handle<Env>, Handle<Closure>), Error> {
    let env = scheme_engine::new_env()?;
    let expr = scheme_engine::parse(source, true)?;
    let program = scheme_engine::compile(env.clone(), &expr)?;
    Ok((env, program.closure().clone()))
}

#[test]
fn test_booleans() {
    let (_env, closure) = compile_closure_env(include_str!("language/boolean.scm"))
        .expect("compiling closure and environment");
    let value = scheme_engine::eval(closure).expect("evaluation");
    println!("Result value: {:?}", value);
}

#[test]
fn test_conditionals() {
    let (_env, closure) = compile_closure_env(include_str!("language/conditionals.scm"))
        .expect("compiling closure and environment");
    let value = scheme_engine::eval(closure).expect("evaluation");
    println!("Result value: {:?}", value);
}

#[test]
fn test_numbers() {
    let (_env, closure) = compile_closure_env(include_str!("language/number.scm"))
        .expect("compiling closure and environment");
    let value = scheme_engine::eval(closure).expect("evaluation");
    println!("Result value: {:?}", value);
}

#[test]
fn test_define() {
    let (env, closure) = compile_closure_env(include_str!("language/define.scm"))
        .expect("compiling closure and environment");
    let _ = scheme_engine::eval(closure).expect("evaluation");

    let symbol_x = env.borrow().resolve_var("x").unwrap();
    let x = env.borrow().get_var(symbol_x).cloned().unwrap();
    assert_eq!(x, Expr::Number(42.0));
}

#[test]
fn test_lambda() {
    let (_env, closure) = compile_closure_env(include_str!("language/lambda.scm"))
        .expect("compiling closure and environment");
    let _ = scheme_engine::eval(closure).expect("evaluation");
}

#[test]
fn test_pair() {
    let (_env, closure) = compile_closure_env(include_str!("language/pair.scm"))
        .expect("compiling closure and environment");
    let _ = scheme_engine::eval(closure).expect("evaluation");
}
