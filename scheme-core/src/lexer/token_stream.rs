use crate::error::Error;
use crate::lexer::{Lexer, Span, Token, TokenKind};

const INIT_TOKEN: Token = Token {
    kind: TokenKind::EOF,
    span: Span::new(0, 0),
};

/// Token stream that allows for peeking at the next token without consuming it.
pub struct TokenStream<'a> {
    lexer: Lexer<'a>,
    peeked: Token,
}

impl<'a> TokenStream<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            peeked: INIT_TOKEN,
        }
    }

    pub fn source(&self) -> &str {
        self.lexer.source()
    }

    pub fn at_end(&self) -> bool {
        self.peeked.kind == TokenKind::EOF && self.lexer.at_end()
    }

    pub fn next_token(&mut self) -> Token {
        if self.peeked.kind != TokenKind::EOF {
            std::mem::replace(&mut self.peeked, INIT_TOKEN)
        } else {
            self.lexer.next_token()
        }
    }

    pub fn peek(&mut self) -> &Token {
        if self.peeked.kind == TokenKind::EOF {
            self.peeked = self.lexer.next_token();
        }
        &self.peeked
    }

    pub fn peek_kind(&mut self) -> TokenKind {
        self.peek().kind
    }

    pub fn consume(&mut self, expected_kind: TokenKind) -> Result<Token, Error> {
        let token = self.next_token();
        if token.kind == expected_kind {
            Ok(token)
        } else {
            Err(Error::UnexpectedToken(
                token,
                format!("expected token kind {expected_kind:?}"),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_stream_creation() {
        let source = "(+ 1 2)";
        let lexer = Lexer::new(source);
        let stream = TokenStream::new(lexer);

        assert_eq!(stream.source(), source);
    }

    #[test]
    fn test_peek_returns_token_without_consuming() {
        let source = "(+ 1 2)";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Peek at the first token multiple times
        let first_peek = stream.peek().kind;
        let second_peek = stream.peek().kind;
        let third_peek = stream.peek().kind;

        // All peeks should return the same token without consuming
        assert_eq!(first_peek, TokenKind::LeftParen);
        assert_eq!(second_peek, TokenKind::LeftParen);
        assert_eq!(third_peek, TokenKind::LeftParen);
    }

    #[test]
    fn test_next_token_advances_stream() {
        let source = "( )";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Get first token
        let first = stream.next_token();
        assert_eq!(first.kind, TokenKind::LeftParen);

        // Get second token
        let second = stream.next_token();
        assert_eq!(second.kind, TokenKind::RightParen);
    }

    #[test]
    fn test_next_token_after_peek() {
        let source = "( )";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Peek at the first token
        let peeked = stream.peek().kind;
        assert_eq!(peeked, TokenKind::LeftParen);

        // Next should return the same token
        let next = stream.next_token();
        assert_eq!(next.kind, TokenKind::LeftParen);

        // Peek again should show the next token
        let next_peek = stream.peek().kind;
        assert_eq!(next_peek, TokenKind::RightParen);
    }

    #[test]
    fn test_peek_kind_method() {
        let source = "(";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        let kind = stream.peek_kind();
        assert_eq!(kind, TokenKind::LeftParen);

        // Peek kind should not consume the token
        let next_peek_kind = stream.peek_kind();
        assert_eq!(next_peek_kind, TokenKind::LeftParen);
    }

    #[test]
    fn test_at_end_after_consuming_all_tokens() {
        let source = "()";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Not at end yet
        assert!(!stream.at_end());

        // Consume all tokens
        stream.next_token(); // (
        stream.next_token(); // )

        // Now should be at end
        assert!(stream.at_end());
    }

    #[test]
    fn test_peek_and_at_end_interaction() {
        let source = "(";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Not at end before consuming
        assert!(!stream.at_end());

        // Peek doesn't consume
        stream.peek();
        assert!(!stream.at_end());

        // Consume the token
        stream.next_token();

        // Now should be at end
        assert!(stream.at_end());
    }

    #[test]
    fn test_multiple_peeks_do_not_advance() {
        let source = "()";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Multiple peeks
        for _ in 0..5 {
            stream.peek();
        }

        // First next should return the first token
        let first = stream.next_token();
        assert_eq!(first.kind, TokenKind::LeftParen);

        // Second next should return the second token
        let second = stream.next_token();
        assert_eq!(second.kind, TokenKind::RightParen);
    }

    #[test]
    fn test_source_consistency() {
        let source = "(define x 42)";
        let lexer = Lexer::new(source);
        let stream = TokenStream::new(lexer);

        // Source should always return the same string
        assert_eq!(stream.source(), source);
        assert_eq!(stream.source(), source);
    }

    #[test]
    fn test_token_span_and_fragment() {
        let source = "(+ x)";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // First token should be (
        let first = stream.next_token();
        assert_eq!(first.kind, TokenKind::LeftParen);
        assert_eq!(first.fragment(source), "(");

        // Check that the span is correct
        assert_eq!(first.span.low(), 0);
        assert_eq!(first.span.high(), 1);
    }

    #[test]
    fn test_peek_returns_same_token() {
        let source = "(";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Multiple peeks should return the same token
        let peek1_kind = stream.peek().kind;
        let peek1_low = stream.peek().span.low();
        let peek1_high = stream.peek().span.high();

        // Peek again and verify
        let peek2_kind = stream.peek().kind;
        let peek2_low = stream.peek().span.low();
        let peek2_high = stream.peek().span.high();

        assert_eq!(peek1_kind, peek2_kind);
        assert_eq!(peek1_low, peek2_low);
        assert_eq!(peek1_high, peek2_high);
    }

    #[test]
    fn test_interleaved_peek_and_next() {
        let source = "( )";
        let lexer = Lexer::new(source);
        let mut stream = TokenStream::new(lexer);

        // Peek, then next
        assert_eq!(stream.peek().kind, TokenKind::LeftParen);
        assert_eq!(stream.next_token().kind, TokenKind::LeftParen);

        // Peek, peek, then next
        assert_eq!(stream.peek().kind, TokenKind::RightParen);
        assert_eq!(stream.peek().kind, TokenKind::RightParen);
        assert_eq!(stream.next_token().kind, TokenKind::RightParen);

        // Should be at end
        assert!(stream.at_end());
    }
}
