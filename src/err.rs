use std::error::Error;
use std::fmt::Display;

pub trait ParseError {}
pub trait TagParseError: Error + ParseError {}

#[derive(Debug)]
pub struct UnterminatedStringLiteral {
    pub loc: usize,
}
impl Display for UnterminatedStringLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unterminated string literal, found at {}", self.loc)
    }
}
impl Error for UnterminatedStringLiteral {} 
impl ParseError for UnterminatedStringLiteral {}
impl TagParseError for UnterminatedStringLiteral {}

#[derive(Debug)]
pub struct UnterminatedAngularBracket {
    pub loc: usize
}
impl Display for UnterminatedAngularBracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unterminated angular bracket, found at location {}", self.loc)
    }
}
impl Error for UnterminatedAngularBracket {}
impl ParseError for UnterminatedAngularBracket {}
