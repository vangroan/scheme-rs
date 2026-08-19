//! External representation.

use std::fmt;
use std::fmt::Display;

use crate::object::PairObject;
use crate::value::{Number, Value};

pub struct ReprDisplay<'a> {
    value: &'a Value,
}

impl<'a> ReprDisplay<'a> {
    pub fn new(value: &'a Value) -> Self {
        Self { value }
    }

    fn fmt_value(value: &Value, f: &mut fmt::Formatter) -> fmt::Result {
        match value {
            Value::Nil => write!(f, "'()"),
            Value::Num(num) => Self::fmt_number(num, f),
            Value::Pair(pair) => Self::fmt_list(&pair, f),
            _ => todo!(),
        }
    }

    fn fmt_number(number: &Number, f: &mut fmt::Formatter) -> fmt::Result {
        match number {
            Number::Integer(n) => Display::fmt(n, f),
            Number::Float(n) => Display::fmt(n, f),
        }
    }

    fn fmt_list(pair: &PairObject, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        Self::fmt_value(&pair.car(), f)?;
        match &pair.cdr() {
            Value::Nil => write!(f, ")"),
            Value::Pair(_) => {
                Self::fmt_rest_recursive(&pair, f)?;
                write!(f, ")")
            }
            _ => {
                write!(f, " . ")?;
                Self::fmt_value(&pair.cdr(), f)?;
                write!(f, ")")
            }
        }
    }

    fn fmt_rest_recursive(pair: &PairObject, f: &mut fmt::Formatter) -> fmt::Result {
        match pair.cdr() {
            Value::Nil => Ok(()),
            Value::Pair(next_pair) => {
                write!(f, " ")?;
                Self::fmt_value(&next_pair.car(), f)?;
                Self::fmt_rest_recursive(&next_pair, f)
            }
            _ => {
                write!(f, " . ")?;
                Self::fmt_value(&pair.cdr(), f)
            }
        }
    }
}

impl fmt::Display for ReprDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Self::fmt_value(self.value, f)
    }
}
