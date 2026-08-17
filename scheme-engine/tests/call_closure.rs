use scheme_engine::Expr;

#[test]
fn test_call_closure() {
    let source = include_str!("test_fibonacci.scm");
    let env = scheme_engine::new_env().unwrap();
    let expr = scheme_engine::parse(source, true).unwrap();
    let program = scheme_engine::compile(env.clone(), &expr).unwrap();

    // Run program to define variables.
    scheme_engine::eval(program.closure().clone()).expect("evaluating top-level fibonacci program");

    println!(
        "fib -> {:?}",
        env.borrow()
            .lookup_var("fib")
            .expect("variable 'fib' not found")
    );

    let fibonacci = env
        .borrow()
        .lookup_var("fib")
        .expect("variable 'fib' not found")
        .as_closure()
        .expect("variable is not a closure")
        .clone();
    let args: Vec<Expr> = vec![Expr::Number(8.0)];

    let value = scheme_engine::call(fibonacci, &args).unwrap();
    assert_eq!(value.as_number(), Some(21.0));
}
