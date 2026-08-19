//! Lexical analysis.

use crate::lexer::cursor::{Cursor, EOF_CHAR};
use crate::lexer::span::Span;
use crate::lexer::token::{Token, TokenKind};

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    /// Original source.
    source: &'a str,
    /// Byte position where the current token starts
    /// in the original source string.
    start_pos: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from the given source code.
    pub fn new(source: &'a str) -> Self {
        let mut cursor = Cursor::new(source);

        // Initial state of the cursor is a non-existent EOF char,
        // but the initial state of the lexer should be a valid
        // token starting character.
        //
        // Prime the cursor for the first iteration.
        cursor.bump();
        let start_pos = cursor.pos();

        Self {
            cursor,
            source,
            start_pos,
        }
    }

    /// Original source passed into the lexer.
    #[inline]
    pub fn source(&self) -> &str {
        self.source
    }

    /// Indicates whether the lexer is at the end of the source.
    ///
    /// Note that source can contain '\0' (end-of-file) characters,
    /// but not be at the actual end. It's thus important to verify
    /// with this function whenever a [`TokenKind::EOF`] is encountered.
    pub fn at_end(&self) -> bool {
        self.cursor.at_end()
    }

    /// Primes the lexer to consume the next token.
    fn start_token(&mut self) {
        // Default position to one after last.
        self.start_pos = self
            .cursor
            .try_pos()
            .unwrap_or_else(|| self.cursor.source().len());
    }

    fn make_token(&mut self, kind: TokenKind) -> Token {
        let start = self.start_pos; // inclusive
        let end = self.cursor.peek_offset(); // exclusive

        // start and end can be equal, and a token can have 0 size.
        debug_assert!(start <= end);
        let size = end - start;

        let span = Span::new(start, size);

        // After this token is built, the lexer's internal state
        // is no longer dedicated to this iteration, but to preparing
        // for the next iteration.
        let token = Token { kind, span };

        // Position the cursor to the starting character for the
        // next token, so the lexer's internal state is primed
        // for the next iteration.
        self.cursor.bump();

        token
    }

    /// Scan the source characters and construct the next token.
    pub fn next_token(&mut self) -> Token {
        // Shorter name for more readable match body.
        use TokenKind as T;

        loop {
            // Invariant: The cursor must be pointing to the start of the
            //            remainder of the source to be consumed next.
            self.start_token();

            let token = match self.cursor.try_char() {
                Some(ch) if ch.is_whitespace() => {
                    self.cursor.bump();
                    continue;
                }
                Some(c) if rules::is_number(c) => self.consume_number(),
                Some(';') => {
                    // Line comment
                    self.skip_line();
                    continue;
                }
                Some('(') => self.make_token(T::LeftParen),
                Some(')') => self.make_token(T::RightParen),
                Some('\'') => self.make_token(T::QuoteMark),
                Some(EOF_CHAR) => {
                    // Source may contain a \0 character but not
                    // actually be at the end of the stream.
                    self.make_token(TokenKind::EOF)
                }
                Some(_) => self.consume_atom(),
                None => self.make_token(TokenKind::EOF),
            };

            return token;
        }
    }

    /// Skip over the remainder of a line, until we encounter a newline character,
    /// or reach the end of the stream.
    fn skip_line(&mut self) {
        while let Some(ch) = self.cursor.try_char() {
            if ch == '\n' {
                break;
            } else {
                self.cursor.bump();
            }
        }
    }

    fn consume_number(&mut self) -> Token {
        while let Some(ch) = self.cursor.peek_char() {
            if !rules::is_number(ch) {
                break;
            }

            self.cursor.bump();
        }

        self.make_token(TokenKind::Number)
    }

    fn consume_atom(&mut self) -> Token {
        // Consume until whitespace, or parentheses.
        while let Some(ch) = self.cursor.peek_char() {
            if ch.is_whitespace() || matches!(ch, '(' | ')') {
                break;
            }

            self.cursor.bump();
        }

        self.make_token(TokenKind::Atom)
    }
}

/// Functions for testing characters.
mod rules {
    /// Test whether the character is considered whitespace
    /// that should be ignored by the parser later.
    #[inline(always)]
    pub fn is_whitespace(c: char) -> bool {
        matches!(
            c,
            '\u{0020}' // space
            | '\u{0009}' // tab
            | '\u{000A}' // linefeed
            | '\u{000D}' // carriage return
            | '\u{00A0}' // no-break space
            | '\u{FEFF}' // zero width no-break space
        )
    }

    #[inline(always)]
    pub fn is_number(c: char) -> bool {
        matches!(c, '0'..='9')
    }

    #[inline(always)]
    pub fn is_symbol(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }
}

impl<'a> IntoIterator for Lexer<'a> {
    type Item = Token;
    type IntoIter = LexerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LexerIter { lexer: self }
    }
}

/// Convenience iterator that wraps the lexer.
pub struct LexerIter<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lexer.at_end() {
            None
        } else {
            Some(self.lexer.next_token())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rules() {
        assert!(!rules::is_symbol('('));
        assert!(!rules::is_symbol(')'));
        assert!(rules::is_symbol('c'));
        assert!(rules::is_symbol('a'));
    }

    #[test]
    fn test_lexer_pos() {
        let mut lexer = Lexer::new("(a b c)");
        assert_eq!(lexer.cursor.current(), (0, '('));
        assert_eq!(lexer.next_token().kind, TokenKind::LeftParen);
        assert_eq!(lexer.cursor.current(), (1, 'a')); //  rest: "a b c)"
        assert_eq!(lexer.next_token().kind, TokenKind::Atom);
        assert_eq!(lexer.cursor.current(), (2, ' ')); // rest: " b c)"
        assert_eq!(lexer.next_token().kind, TokenKind::Atom); // b
        assert_eq!(lexer.cursor.current(), (4, ' ')); // rest: " c)"
        assert_eq!(lexer.next_token().kind, TokenKind::Atom); // c
        assert_eq!(lexer.cursor.current(), (6, ')')); // rest: ")"
        assert_eq!(lexer.next_token().kind, TokenKind::RightParen);
    }

    #[test]
    fn test_lexer() {}
}
