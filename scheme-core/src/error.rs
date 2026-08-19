use core::fmt;

use crate::lexer::Token;

#[derive(Debug)]
pub enum Error {
    UnexpectedToken(Token, String),
    UnexpectedEOF,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedToken(token, message) => {
                write!(f, "Unexpected token: {:?}, message: {message}", token.kind)
            }
            Error::UnexpectedEOF => write!(f, "Unexpected end of input"),
        }
    }
}

impl std::error::Error for Error {}
