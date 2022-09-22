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

#[derive(Debug)]
pub struct PeekOutOfBoundsError {
    pub peek_offset: i64,
    pub cur_idx: usize,
    pub len: usize,
}
impl Display for PeekOutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Peek out of bounds, offset {} out of bounds for current index {} and total length {}", self.peek_offset, self.cur_idx, self.len)
    }
}
impl Error for PeekOutOfBoundsError {}
impl ParseError for PeekOutOfBoundsError {}

#[derive(Debug)]
pub struct InvalidFirstTokenError;
impl Display for InvalidFirstTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "First token for any tag should be of kind String")
    }
}
impl Error for InvalidFirstTokenError {}
impl ParseError for InvalidFirstTokenError {}
impl TagParseError for InvalidFirstTokenError {}
