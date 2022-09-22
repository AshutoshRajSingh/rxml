use crate::err;
use std::cell::{Ref, RefCell};
use std::cmp::PartialEq;
use std::collections::HashMap;
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
            return '\0';
        }
        self.content.as_bytes()[self.cur()] as char
    }
    fn next(&self) {
        *self.position.borrow_mut() += 1;
    }
    fn end(&self) -> bool {
        *self.position.borrow() >= self.content.len()
    }
    pub fn next_token(&self) -> Result<TagToken, Box<dyn err::TagParseError>> {
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
        } else if self.current().is_alphabetic() || self.current() == '_' {
            while self.current().is_alphanumeric() || self.current() == '_' {
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
pub struct XMLTag {
    pub name: String,
    pub attribs: HashMap<String, String>,
}

impl XMLTag {
    pub fn new(name: String, attribs: HashMap<String, String>) -> Self {
        Self { name, attribs }
    }
}

#[derive(Debug)]
pub struct TagParser<'a> {
    pub content: &'a str,
    pub lexer: TagLexer<'a>,
    position: RefCell<usize>,
    tokens: RefCell<Vec<TagToken<'a>>>,
}

impl<'a> TagParser<'a> {
    pub fn new(content: &'a str) -> Self {
        let tokens: RefCell<Vec<TagToken>> = RefCell::new(Vec::new());
        if content.starts_with("<") && content.ends_with(">") {
            let trimmed = &content[1..content.len() - 1];
            let lexer = TagLexer::new(trimmed);
            return Self {
                content: trimmed,
                lexer,
                position: RefCell::new(0),
                tokens,
            };
        }
        let lexer = TagLexer::new(content);
        Self {
            content,
            lexer,
            position: RefCell::new(0),
            tokens,
        }
    }

    fn tokenize(&'a self) -> Result<(), Box<dyn err::TagParseError>> {
        loop {
            let cur_token = self.lexer.next_token()?;
            if let TokenKind::EndOfLine = cur_token.kind {
                self.tokens.borrow_mut().push(cur_token);
                break;
            }
            if let TokenKind::Whitespace = cur_token.kind {
            } else {
                self.tokens.borrow_mut().push(cur_token);
            }
        }
        Ok(())
    }

    fn peek(&self, offset: i64) -> Result<Ref<'a, TagToken>, err::PeekOutOfBoundsError> {
        let pos_copy = *self.position.borrow() as i64;
        if pos_copy + offset < 0 || pos_copy + offset >= self.content.len() as i64 {
            return Err(err::PeekOutOfBoundsError {
                peek_offset: offset,
                cur_idx: *self.position.borrow(),
                len: self.content.len(),
            });
        }
        let idx = (pos_copy + offset) as usize;
        return Ok(Ref::map(self.tokens.borrow(), |tkns| &tkns[idx]));
    }

    fn cur_token(&self) -> Ref<'a, TagToken> {
        Ref::map(self.tokens.borrow(), |tkns| &tkns[*self.position.borrow()])
    }

    fn next(&self) {
        *self.position.borrow_mut() += 1;
    }

    fn end(&self) -> bool {
        *self.position.borrow() >= self.tokens.borrow().len()
    }

    pub fn parse(&'a self) -> Result<XMLTag, Box<dyn err::TagParseError>> {
        self.tokenize()?;
        let first = self.cur_token();
        let name: String;

        if let TokenKind::String = first.kind {
            name = String::from(first.text);
        } else {
            return Err(Box::new(err::InvalidFirstTokenError));
        }

        let mut attribs: HashMap<String, String> = HashMap::new();

        while !self.end() {
            let cur = self.cur_token();

            if let TokenKind::Equals = cur.kind {
                let left = match self.peek(-1) {
                    Ok(tkn) => tkn,
                    Err(_) => {
                        return Err(Box::new(err::NoTokenAtLocationError {
                            expected_kind: String::from("String"),
                            direction: String::from("left"),
                            current: String::from("Equals"),
                        }));
                    }
                };
                let right = match self.peek(1) {
                    Ok(tkn) => tkn,
                    Err(_) => {
                        return Err(Box::new(err::NoTokenAtLocationError {
                            expected_kind: String::from("StringLiteral"),
                            direction: String::from("right"),
                            current: String::from("Equals"),
                        }));
                    }
                };
                if let (TokenKind::String, TokenKind::StringLiteral) = (&left.kind, &right.kind) {
                    let k = String::from(left.text);
                    let v = String::from(&right.text[1..right.text.len() - 1]);
                    attribs.insert(k, v);
                } else {
                    return Err(Box::new(err::UnexpectedTagTokenError));
                }
            }
            self.next();
        }
        Ok(XMLTag::new(name, attribs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_tokenization() {
        let text = "tagname string1 __string2 _string_3_";

        let actual_tokens = vec![
            TagToken::new("tagname", TokenKind::String, 0),
            TagToken::new(" ", TokenKind::Whitespace, 7),
            TagToken::new("string1", TokenKind::String, 8),
            TagToken::new(" ", TokenKind::Whitespace, 15),
            TagToken::new("__string2", TokenKind::String, 16),
            TagToken::new(" ", TokenKind::Whitespace, 25),
            TagToken::new("_string_3_", TokenKind::String, 26),
        ];

        let test_lexer = TagLexer::new(text);
        let mut tokens: Vec<TagToken> = Vec::new();

        while let Ok(token) = test_lexer.next_token() {
            if let TokenKind::EndOfLine = token.kind {
                break;
            }
            println!("{:?}", token);
            tokens.push(token);
        }

        assert_eq!(tokens, actual_tokens);
    }

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

    #[test]
    fn test_tag_parser_success() {
        let text = "tagname attribute1 = 'value1'";

        let test_parser = TagParser::new(text);
        let test_tag = test_parser.parse().unwrap();

        let mut actual_attribs: HashMap<String, String> = HashMap::new();

        actual_attribs.insert(String::from("attribute1"), String::from("value1"));

        assert_eq!(test_tag.name, String::from("tagname"));
        assert_eq!(test_tag.attribs, actual_attribs);
    }
}
