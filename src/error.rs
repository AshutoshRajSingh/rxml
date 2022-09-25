use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum TagParseError {
    UnterminatedStringLiteral(usize),
    PeekOutOfBounds {
        offset: i64,
        cur_idx: usize,
        len: usize,
    },
    NoTokenAtLocation {
        expected_kind: String,
        direction: String,
        current: String,
    },
    UnexpectedTagToken,
    InvalidFirstToken,
}
impl Display for TagParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagParseError::UnterminatedStringLiteral(loc) => {
                write!(f, "Unterminated string literal, found at {}", loc)
            }
            TagParseError::PeekOutOfBounds {
                offset,
                cur_idx,
                len,
            } => {
                write!(
                    f,
                    "Peek out of bounds, offset {} out of bounds for current index {} and total length {}",
                    offset, cur_idx, len
                )
            }
            TagParseError::NoTokenAtLocation {
                expected_kind,
                direction,
                current,
            } => {
                write!(
                    f,
                    "Expected {} on the {} of {} token but found nothing",
                    expected_kind, direction, current
                )
            }
            TagParseError::UnexpectedTagToken => {
                write!(f, "Expected String token on the left and StringLiteral token on the right of Equals token")
            }
            TagParseError::InvalidFirstToken => {
                write!(
                    f,
                    "First token of any tag should either be of type String or ForwardSlash"
                )
            }
        }
    }
}
impl Error for TagParseError {}

#[derive(Debug)]
pub enum ParseError {
    UnterminatedAngularBracket(usize),
    TagParseError(TagParseError),
    NoTokensToParse,
    InvalidFirstToken,
    UnexpectedClosingTag {
        expected: String,
        obtained: String,
        position: usize,
    },
    ClosingTagNeverOpened {
        obtained: String,
        position: usize,
    },
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnterminatedAngularBracket(loc) => {
                write!(f, "Unterminated angular bracket, found at location {}", loc)
            }
            ParseError::TagParseError(internal_err) => {
                write!(f, "{}", internal_err.to_string())
            }
            ParseError::NoTokensToParse => {
                write!(f, "Lexer could not produce any tokens")
            }
            ParseError::InvalidFirstToken => {
                write!(f, "First token of xml document should be of type XMLTag")
            }
            ParseError::UnexpectedClosingTag {
                expected,
                obtained,
                position,
            } => {
                write!(
                    f,
                    "Unexpected closing tag at location {}, expected: {}, obtained: {}",
                    position, expected, obtained
                )
            }
            ParseError::ClosingTagNeverOpened { obtained, position } => {
                write!(
                    f,
                    "Closing tag: {}, found at position: {} was never opened",
                    obtained, position
                )
            }
        }
    }
}

impl Error for ParseError {}
