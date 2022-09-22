use crate::err;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::mem::discriminant;

#[derive(Debug, Clone)]
pub enum TokenKind {
    String,
    StringLiteral,
    Equals,
    Unknown,
    Whitespace,
    EndOfLine,
}

#[derive(Debug, Clone)]
pub struct TagToken<'a> {
    kind: TokenKind,
    text: &'a str,
    _position: usize,
}
impl<'a> TagToken<'a> {
    fn new(text: &'a str, kind: TokenKind, _position: usize) -> Self {
        Self {
            text,
            kind,
            _position,
        }
    }
}
impl<'a> PartialEq for TagToken<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self._position == other._position
            && discriminant(&self.kind) == discriminant(&other.kind)
    }
}

#[derive(Debug)]
pub struct TagLexer<'a> {
    content: &'a str,
    position: RefCell<usize>,
}
impl<'a> TagLexer<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content,
            position: RefCell::new(0),
        }
    }
    fn cur(&self) -> usize {
        *self.position.borrow()
    }
    fn current(&self) -> char {
        if self.end() {
            return self.content.as_bytes()[self.content.len() - 1] as char;
        }
        self.content.as_bytes()[self.cur()] as char
    }
    fn next(&self) {
        *self.position.borrow_mut() += 1;
    }
    fn end(&self) -> bool {
        *self.position.borrow() >= self.content.len()
    }
    pub fn next_token(&self) -> Result<TagToken, Box<dyn err::TagParseError + 'a>> {
        let start = self.cur();
        if self.end() {
            return Ok(TagToken::new(
                &self.content[self.content.len() - 1..self.content.len() - 1],
                TokenKind::EndOfLine,
                self.content.len(),
            ));
        } else if self.current().is_whitespace() {
            self.next();
            return Ok(TagToken::new(
                &self.content[start..start + 1],
                TokenKind::Whitespace,
                start,
            ));
        } else if self.current() == '\'' || self.current() == '"' {
            let quote_type = self.current();

            self.next();

            while self.current() != quote_type {
                if self.end() {
                    return Err(Box::new(err::UnterminatedStringLiteral { loc: start }));
                }
                self.next();
            }
            self.next();

            let end = self.cur();

            return Ok(TagToken::new(
                &self.content[start..end],
                TokenKind::StringLiteral,
                start,
            ));
        } else if self.current().is_alphabetic() {
            while self.current().is_alphanumeric() {
                self.next();
            }

            return Ok(TagToken::new(
                &self.content[start..self.cur()],
                TokenKind::String,
                start,
            ));
        } else if self.current() == '=' {
            self.next();

            return Ok(TagToken::new(
                &self.content[start..start + 1],
                TokenKind::Equals,
                start,
            ));
        } else {
            self.next();

            while !self.current().is_whitespace() {
                self.next();
            }

            return Ok(TagToken::new(
                &self.content[start..self.cur()],
                TokenKind::Unknown,
                start,
            ));
        }
    }
}

#[derive(Debug)]
pub struct TagParser<'a> {
    pub content: &'a str,
    pub lexer: TagLexer<'a>,
}

impl<'a> TagParser<'a> {
    pub fn new(content: &'a str) -> Self {
        let lexer = TagLexer::new(content);
        Self { content, lexer }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_lexer() {
        let text = "tagname attribute1 = 'value1'";

        let actual_tokens = vec![
            TagToken::new("tagname", TokenKind::String, 0),
            TagToken::new(" ", TokenKind::Whitespace, 7),
            TagToken::new("attribute1", TokenKind::String, 8),
            TagToken::new(" ", TokenKind::Whitespace, 18),
            TagToken::new("=", TokenKind::Equals, 19),
            TagToken::new(" ", TokenKind::Whitespace, 20),
            TagToken::new("'value1'", TokenKind::StringLiteral, 21),
        ];

        let mut obtained_tokens: Vec<TagToken> = Vec::new();
        let test_lexer = TagLexer::new(text);

        while let Ok(token) = test_lexer.next_token() {
            if let TokenKind::EndOfLine = token.kind {
                break;
            }
            obtained_tokens.push(token);
        }
        assert_eq!(obtained_tokens, actual_tokens);
    }
}
