//! Parser.

use crate::expr::{Expr};
use crate::ext::*;
use crate::{
    error::{Error, Result},
    lexer::Lexer,
    token::{Token, TokenKind},
};

/// Parse the given text as Scheme code.
///
/// When the `is_sequence` flag is true the text is treated as
/// a top-level program, a sequence of expressions. When it's
/// false the text is treated as a single expression.
pub fn parse(source: &str, is_sequence: bool) -> Result<Expr> {
    let mut lexer = Lexer::new(source);

    // Position the lexer so the current token points to the first token.
    lexer.next_token();

    if is_sequence {
        // Top level of file contents
        parse_sequence(&mut lexer)
    } else {
        parse_expr(&mut lexer)
    }
}

pub fn parse_v2(source: &str) -> Result<Expr> {
    let mut lexer = Lexer::new(source);

    // Position the lexer so the current token points to the first token.
    lexer.next_token();

    parse_sequence_pair(&mut lexer)
}

fn parse_sequence_pair(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_sequence_pair({:?})", lexer.rest());

    let mut head = Expr::Nil;
    let mut tail = Expr::Nil;

    while let Some(token) = lexer.current_token() {
        if token.kind == TokenKind::EOF {
            break;
        }

        let expr = parse_expr_v2(lexer)?;

        // Push right
        match tail {
            Expr::Nil => {
                tail = Expr::new_pair(expr, Expr::Nil);
            }
            Expr::Pair(pair) => {
                tail = Expr::new_pair(expr, Expr::Nil);
                pair.borrow_mut().set_right(tail.clone());
            }
            _ => panic!("list tail must be a pair or nil"),
        }

        if head.is_nil() {
            head = tail.clone();
        }
    }

    Ok(head)
}

fn parse_expr_v2(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_expr_v2({:?})", lexer.rest());

    let token = lexer
        .current_token()
        .cloned()
        .ok_or_else(|| Error::Reason("unexpected end".to_string()))?;
    lexer.next_token();

    match token.kind {
        TokenKind::LeftParen => parse_list_v2(lexer),
        TokenKind::EOF => Err(Error::Reason("unexpected end-of-file".to_string())),
        TokenKind::RightParen => Err(Error::Reason("unexpected right parentheses".to_string())),
        TokenKind::QuoteMark => parse_quote(lexer),
        _ => {
            let fragment = token.fragment(lexer.source());
            parse_atom(token.clone(), fragment)
        }
    }
}

fn parse_list_v2(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_list_v2({:?})", lexer.rest());

    let mut head = Expr::Nil;
    let mut tail = Expr::Nil;

    while let Some(token) = lexer.current_token() {
        match token.kind {
            // Closes list.
            TokenKind::RightParen => {
                lexer.next_token();
                break;
            }
            TokenKind::EOF => {
                return Err(Error::Reason("unexpected end-of-file".to_string()));
            }
            _ => {
                let expr = parse_expr(lexer)?;

                // Push right
                match tail {
                    Expr::Nil => {
                        tail = Expr::new_pair(expr, Expr::Nil);
                    }
                    Expr::Pair(pair) => {
                        tail = Expr::new_pair(expr, Expr::Nil);
                        pair.borrow_mut().set_right(tail.clone());
                    }
                    _ => panic!("list tail must be a pair or nil"),
                }

                if head.is_nil() {
                    head = tail.clone();
                }
            }
        }
    }

    Ok(head)
}

fn parse_sequence(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_sequence({:?})", lexer.rest());

    let mut expressions = Vec::new();

    while let Some(token) = lexer.current_token() {
        if token.kind == TokenKind::EOF {
            break;
        }

        let expr = parse_expr(lexer)?;
        expressions.push(expr);
    }

    Ok(Expr::Sequence(expressions))
}

fn parse_expr(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_expr({:?})", lexer.rest());

    let token = lexer
        .current_token()
        .cloned()
        .ok_or_else(|| Error::Reason("unexpected end".to_string()))?;
    lexer.next_token();

    match token.kind {
        TokenKind::LeftParen => parse_list(lexer),
        TokenKind::EOF => Err(Error::Reason("unexpected end-of-file".to_string())),
        TokenKind::RightParen => Err(Error::Reason("unexpected right parentheses".to_string())),
        TokenKind::QuoteMark => parse_quote(lexer),
        _ => {
            let fragment = token.fragment(lexer.source());
            parse_atom(token.clone(), fragment)
        }
    }
}

fn parse_list(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_list({:?})", lexer.rest());

    let mut expressions = Vec::new();

    while let Some(token) = lexer.current_token() {
        match token.kind {
            TokenKind::RightParen => {
                lexer.next_token();
                break;
            }
            TokenKind::EOF => {
                return Err(Error::Reason("unexpected end-of-file".to_string()));
            }
            _ => {
                let expr = parse_expr(lexer)?;
                expressions.push(expr);
            }
        }
    }

    Ok(Expr::List(expressions))
}

fn parse_quote(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_quote({:?})", lexer.rest());
    parse_expr(lexer).map(Box::new).map(Expr::Quote)
}

fn parse_atom(token: Token, fragment: &str) -> Result<Expr> {
    println!("parse_atom({:?}, {:?})", token, fragment);

    use TokenKind::*;
    debug_assert_eq!(token.kind, Atom);

    if let Some((ch, rest)) = fragment.split_first_char() {
        match ch {
            '0'..='9' => parse_number(token, fragment),
            '#' => match rest.first() {
                Some('t') => Ok(Expr::Bool(true)),
                Some('f') => Ok(Expr::Bool(false)),
                Some('b') => todo!("parse binary number"),
                Some('x') => todo!("parse hexadecimal number"),
                _ => match rest {
                    "void" => Ok(Expr::Void),
                    _ => Err(Error::Reason(format!("unknown atom: {ch:?}"))),
                },
            },
            '+' | '-' | '*' | '=' | '<' | '>' | 'a'..='z' => {
                // TODO: The complex identifier rules
                parse_identifier(token, fragment)
            }
            _ => Err(Error::Reason(format!("unexpected character: {ch:?}"))),
        }
    } else {
        Err(Error::Reason("expected atom".to_string()))
    }
}

fn parse_number(_token: Token, fragment: &str) -> Result<Expr> {
    let number = fragment
        .parse::<f64>()
        .map_err(|err| Error::Reason(format!("failed to parse number: {err}")))?;

    Ok(Expr::Number(number))
}

fn parse_identifier(_token: Token, fragment: &str) -> Result<Expr> {
    // TODO: The complex identifier rules
    Ok(Expr::Ident(fragment.into()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numbers() {
        let expr = parse("(1 2 (3 4) (5 (6 7 8)))", false).expect("parse failed");
        println!("{:#?}", expr);

        let list1 = expr.as_slice().unwrap();
        assert_eq!(list1[0], Expr::Number(1.0));
        assert_eq!(list1[1], Expr::Number(2.0));

        let list2 = list1[2].as_slice().unwrap();
        assert_eq!(list2[0], Expr::Number(3.0));
        assert_eq!(list2[1], Expr::Number(4.0));

        let list3 = list1[3].as_slice().unwrap();
        assert_eq!(list3[0], Expr::Number(5.0));

        let list4 = list3[1].as_slice().unwrap();
        assert_eq!(list4[0], Expr::Number(6.0));
        assert_eq!(list4[1], Expr::Number(7.0));
        assert_eq!(list4[2], Expr::Number(8.0));
    }

    #[test]
    fn test_boolean() {
        let expr = parse("(#t #f)", false).expect("parse failed");
        assert!(matches!(expr, Expr::List(_)));

        let list = expr.as_slice().unwrap();
        assert_eq!(list[0], Expr::Bool(true));
        assert_eq!(list[1], Expr::Bool(false));
    }

    #[test]
    fn test_sequence() {
        let source = r#"
        (one 1)
        (two 2)
        (three 3)
        "#;

        let expr = parse(source, true).expect("parse failed");
        assert!(matches!(expr, Expr::Sequence(_)));
    }
}
