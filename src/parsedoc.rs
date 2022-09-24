use crate::{
    error,
    parsetag::{TagParser, XMLTag},
};
use std::cell::{Ref, RefCell};
use std::cmp::PartialEq;
use std::mem::discriminant;

#[derive(Debug)]
enum TokenKind {
    Tag(XMLTag),
    String,
    EndOfFile,
    Whitespace,
}

#[derive(Debug)]
pub struct DocToken<'a> {
    text: &'a str,
    kind: TokenKind,
    position: usize,
}
impl<'a> DocToken<'a> {
    fn new(text: &'a str, kind: TokenKind, position: usize) -> Self {
        Self {
            text,
            kind,
            position,
        }
    }
}
impl<'a> PartialEq for DocToken<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.position == other.position
            && discriminant(&self.kind) == discriminant(&other.kind)
    }
}
pub struct XMLLexer<'a> {
    content: &'a str,
    position: RefCell<usize>,
}

impl<'a> XMLLexer<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            position: RefCell::new(0),
        }
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
    fn cur(&self) -> usize {
        *self.position.borrow()
    }
    fn end(&self) -> bool {
        self.cur() >= self.content.len()
    }
    pub fn next_token(&self) -> Result<DocToken, error::ParseError> {
        let start = self.cur();
        if self.end() {
            return Ok(DocToken::new(
                &self.content[self.content.len() - 1..self.content.len() - 1],
                TokenKind::EndOfFile,
                self.content.len(),
            ));
        } else if self.current().is_whitespace() {
            self.next();
            return Ok(DocToken::new(
                &self.content[start..self.cur()],
                TokenKind::Whitespace,
                start,
            ));
        } else if self.current() == '<' {
            self.next();

            while self.current() != '>' {
                if self.end() {
                    return Err(
                        error::ParseError::UnterminatedAngularBracket(start)
                    );
                }
                self.next();
            }

            let tagtext = &self.content[start..self.cur() + 1];

            self.next();

            let tagparser = TagParser::new(tagtext);

            let tag = match tagparser.parse() {
                Ok(t) => t,
                Err(e) => {
                    return Err(error::ParseError::TagParseError(e));
                }
            };

            return Ok(DocToken::new(tagtext, TokenKind::Tag(tag), start));
        } else {
            while !self.current().is_whitespace() || self.end() {
                if self.current() == '<' {
                    break;
                }
                self.next();
            }
            return Ok(DocToken::new(
                &self.content[start..self.cur()],
                TokenKind::String,
                start,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::parsetag::TagKind;

    use super::*;

    #[test]
    fn test_xml_tokenization() {
        let text = "<xml> < person  age='55'  > David   < / person >< / xml  >";

        let test_lexer = XMLLexer::new(text);
        let mut parsed_tokens: Vec<DocToken> = Vec::new();

        while let Ok(token) = test_lexer.next_token() {
            match token.kind {
                TokenKind::Whitespace => {}
                TokenKind::EndOfFile => {
                    break;
                }
                _ => parsed_tokens.push(token),
            }
        }

        let actual_tokens = vec![
            DocToken::new(
                "<xml>",
                TokenKind::Tag(XMLTag::new(
                    String::from("xml"),
                    HashMap::new(),
                    TagKind::Opening,
                )),
                0,
            ),
            DocToken::new(
                "< person  age='55'  >",
                TokenKind::Tag(XMLTag::new(
                    String::from("person"),
                    HashMap::from([(String::from("age"), String::from("55"))]),
                    TagKind::Opening,
                )),
                6,
            ),
            DocToken::new("David", TokenKind::String, 28),
            DocToken::new(
                "< / person >",
                TokenKind::Tag(XMLTag::new(
                    String::from("person"),
                    HashMap::new(),
                    TagKind::Closing,
                )),
                36,
            ),
            DocToken::new(
                "< / xml  >",
                TokenKind::Tag(XMLTag::new(
                    String::from("xml"),
                    HashMap::new(),
                    TagKind::Closing,
                )),
                48,
            ),
        ];

        assert_eq!(parsed_tokens, actual_tokens);
    }
}
