//! Token Definition

use crate::lexer::span::Span;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    LeftParen,  // (
    RightParen, // )
    Atom,       // c, a, b, etc.
    Number,     // 1, 2, 3, etc.
    QuoteMark,  // '
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn fragment<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.as_range()]
    }
}
