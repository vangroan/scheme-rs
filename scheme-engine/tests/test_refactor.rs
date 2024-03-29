//! Tests for the refactor from Vec to Pair representation.
use scheme_engine::{parse_v2, Expr};

#[test]
fn test_parse_pair() {
    const SOURCE: &str = "(+ 1 2) (- 3 4)";
    let root = parse_v2(SOURCE).expect("failed to parse");

    let pair = root.into_pair();

    let a = pair.left().as_pair();
    assert_eq!(a.left(), &Expr::new_ident("+"));
    let b = a.right().as_pair();
    assert_eq!(b.left(), &Expr::Number(1.0));
    let c = b.right().as_pair();
    assert_eq!(c.left(), &Expr::Number(2.0));

    let right = pair.right().as_pair();
    let d = right.left().as_pair();
    assert_eq!(d.left(), &Expr::new_ident("-"));
    let e = d.right().as_pair();
    assert_eq!(e.left(), &Expr::Number(3.0));
    let f = e.right().as_pair();
    assert_eq!(f.left(), &Expr::Number(4.0));

    assert_eq!(
        right.right(),
        &Expr::Nil,
        "sequence must be a well-formed list"
    );
}
