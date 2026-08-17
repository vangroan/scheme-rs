//! Tests for pairs (*cons*)

use scheme_engine::utils::{cons, nil};
use scheme_engine::{Expr, Handle, Pair};

#[test]
fn test_split_first() {
    let expr = cons(1.0, 2.0);
    let pair = expr.as_pair();

    if let (first, rest) = pair.split_first() {
        println!("{first:?}, {rest:?}");
    }
}

#[test]
fn test_make_list() {
    let list = cons(1.0, cons(2.0, nil()));
    println!("{list:?}");
    println!("{}", list.repr());

    let list = Pair::new_list(&[Expr::Number(1.0), Expr::Number(2.0)]).unwrap();
    println!("{list:?}");
    println!("{}", Expr::Pair(Handle::new(list)).repr());

    let list = Pair::new(
        Expr::Number(1.0),
        Expr::Pair(Handle::new(Pair::new(Expr::Number(2.0), Expr::Number(3.0)))),
    );
    println!("{list:?}");
    println!("{}", Expr::Pair(Handle::new(list)).repr());
}

#[test]
fn test_make_list_from_vec() {
    let a = Pair::new_list_vec(vec![Expr::Number(1.0), Expr::Number(2.0)]).unwrap();
    println!("{a:?}");
    println!("{}", Expr::Pair(Handle::new(a)).repr());

    let b = Pair::new_list_vec(vec![
        Expr::Number(1.0),
        Expr::Number(2.0),
        Expr::Number(3.0),
    ])
    .unwrap();
    println!("{b:?}");
    println!("{}", Expr::Pair(Handle::new(b)).repr());
}
