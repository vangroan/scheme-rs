//! Tests for the external display representation of values.
use scheme_engine::utils::{cons, nil};
use scheme_engine::Pair;

#[test]
fn test_list_repr() {
    assert_eq!(
        Pair::new_list(&[1.0.into(), 2.0.into()])
            .unwrap()
            .to_expr()
            .repr()
            .to_string(),
        "(1 2)"
    );

    assert_eq!(
        Pair::new_list(&[1.0.into(), 2.0.into(), 3.0.into()])
            .unwrap()
            .to_expr()
            .repr()
            .to_string(),
        "(1 2 3)"
    );

    assert_eq!(cons(1.0, cons(2.0, nil())).repr().to_string(), "(1 2)");
    assert_eq!(cons(1.0, cons(2.0, 3.0)).repr().to_string(), "(1 2 . 3)");
}
