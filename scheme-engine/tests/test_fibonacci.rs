#[test]
fn test_fibonacci_sequence() {
    let source = include_str!("test_fibonacci.scm");
    let env = scheme_engine::new_env().unwrap();
    let expr = scheme_engine::parse(source, true).unwrap();
    let program = scheme_engine::compile(env.clone(), &expr).unwrap();

    scheme_engine::eval(program.closure().clone()).expect("fibonacci sequence failed");
}
