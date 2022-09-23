use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Display};

pub trait ParseError: Debug + Error {
    fn as_any(&self) -> &dyn Any;
}
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
impl ParseError for UnterminatedStringLiteral {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl TagParseError for UnterminatedStringLiteral {}

#[derive(Debug)]
pub struct UnterminatedAngularBracket {
    pub loc: usize,
}
impl Display for UnterminatedAngularBracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unterminated angular bracket, found at location {}",
            self.loc
        )
    }
}
impl Error for UnterminatedAngularBracket {}
impl ParseError for UnterminatedAngularBracket {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct PeekOutOfBoundsError {
    pub peek_offset: i64,
    pub cur_idx: usize,
    pub len: usize,
}
impl Display for PeekOutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Peek out of bounds, offset {} out of bounds for current index {} and total length {}",
            self.peek_offset, self.cur_idx, self.len
        )
    }
}
impl Error for PeekOutOfBoundsError {}
impl ParseError for PeekOutOfBoundsError {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct InvalidFirstTokenError;
impl Display for InvalidFirstTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "First token for any tag should be of kind String")
    }
}
impl Error for InvalidFirstTokenError {}
impl ParseError for InvalidFirstTokenError {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl TagParseError for InvalidFirstTokenError {}

#[derive(Debug)]
pub struct NoTokenAtLocationError {
    pub expected_kind: String,
    pub direction: String,
    pub current: String,
}
impl Display for NoTokenAtLocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Expected {} on the {} of {} token but found nothing",
            self.expected_kind, self.direction, self.current
        )
    }
}
impl Error for NoTokenAtLocationError {}
impl ParseError for NoTokenAtLocationError {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl TagParseError for NoTokenAtLocationError {}

#[derive(Debug)]
pub struct UnexpectedTagTokenError;
impl Display for UnexpectedTagTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected String token on the left and StringLiteral token on the right of Equals token")
    }
}
impl Error for UnexpectedTagTokenError {}
impl ParseError for UnexpectedTagTokenError {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl TagParseError for UnexpectedTagTokenError {}

#[derive(Debug)]
pub struct TagParseErrorWrapper {
    pub inner_err: Box<dyn TagParseError>,
}
impl Display for TagParseErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner_err)
    }
}
impl Error for TagParseErrorWrapper {}
impl ParseError for TagParseErrorWrapper {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
