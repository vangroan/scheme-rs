use std::cell::RefCell;
use std::rc::Rc;

use crate::store::Store;
use scheme_gc::GcHeap;

use crate::error::Error;
use crate::lexer::{TokenKind, TokenStream};
use crate::object::PairObject;
use crate::value::{Number, Value};

macro_rules! unexpected_eof {
    () => {
        Error::UnexpectedEOF
    };
}

macro_rules! unexpected_token {
    ($token:expr, $message:expr) => {
        Error::UnexpectedToken($token.clone(), $message.to_string())
    };
}

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
    heap: Rc<RefCell<GcHeap>>,
}

impl<'a> Parser<'a> {
    pub fn new(store: &Store, tokens: TokenStream<'a>) -> Self {
        Self {
            tokens,
            heap: store.heap.clone(),
        }
    }

    pub fn parse_program(&mut self) -> Result<Value, Error> {
        // TODO: Parse imports
        // TODO: Parse definitions
        todo!()
    }

    pub fn parse_expression(&mut self) -> Result<Value, Error> {
        match self.tokens.peek_kind() {
            TokenKind::LeftParen => self.parse_procedure_call(),
            TokenKind::QuoteMark => self.parse_quote(),
            TokenKind::Atom => self.parse_atom(),
            TokenKind::Number => self.parse_number(),
            TokenKind::RightParen => {
                Err(unexpected_token!(self.tokens.peek(), "expected expression"))
            }
            TokenKind::EOF => Err(unexpected_eof!()),
        }
    }

    /// Parse a procedure call expression of the form:
    ///
    /// ```lisp
    /// (<operator> <operand>*)
    /// ```
    fn parse_procedure_call(&mut self) -> Result<Value, Error> {
        self.tokens.consume(TokenKind::LeftParen)?;

        let operator = self.parse_expression()?;
        let operands = self.parse_procedure_arguments_recursive()?;

        Ok(PairObject::alloc(&mut self.heap.borrow_mut(), operator, operands).into())
    }

    /// ```lisp
    /// <operand> <operand> ... <operand>
    /// ```
    fn parse_procedure_arguments_recursive(&mut self) -> Result<Value, Error> {
        if self.tokens.peek_kind() == TokenKind::RightParen {
            self.tokens.consume(TokenKind::RightParen)?;
            return Ok(Value::Nil);
        }

        let car = self.parse_expression()?;
        let cdr = self.parse_procedure_arguments_recursive()?;

        Ok(PairObject::alloc(&mut self.heap.borrow_mut(), car, cdr).into())
    }

    fn parse_quote(&mut self) -> Result<Value, Error> {
        todo!()
    }

    fn parse_atom(&mut self) -> Result<Value, Error> {
        todo!()
    }

    fn parse_number(&mut self) -> Result<Value, Error> {
        let token = self.tokens.consume(TokenKind::Number)?;
        let fragment = token.fragment(self.tokens.source());

        match fragment.parse::<i64>() {
            Ok(num) => Ok(Value::Num(Number::from_i64(num))),
            Err(_) => Err(unexpected_token!(token, "invalid number literal")),
        }
    }
}
