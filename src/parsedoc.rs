use crate::{
    api::{XMLNode, XMLTag},
    error,
    parsetag::{BaseXMLTag, TagKind, TagParser},
};
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::mem::discriminant;
use std::rc::Rc;

#[derive(Debug)]
enum TokenKind {
    Tag(BaseXMLTag),
    String,
    EndOfFile,
    Whitespace,
}

#[derive(Debug)]
struct DocToken<'a> {
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
        let pre = self.text == other.text
            && self.position == other.position
            && discriminant(&self.kind) == discriminant(&other.kind);

        let post: bool;

        if let (TokenKind::Tag(t1), TokenKind::Tag(t2)) = (&self.kind, &other.kind) {
            post = t1 == t2;
        } else {
            post = true;
        }
        pre && post
    }
}
pub struct XMLLexer<'a> {
    content: &'a str,
    position: RefCell<usize>,
}
impl<'a> XMLLexer<'a> {
    fn new(content: &'a str) -> Self {
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
    fn next_token(&self) -> Result<DocToken, error::ParseError> {
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
                    return Err(error::ParseError::UnterminatedAngularBracket(start));
                }
                self.next();
            }

            let tagtext = &self.content[start..self.cur() + 1];

            self.next();

            let tagparser = TagParser::new(tagtext, start);

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

pub struct XMLParser<'a> {
    lexer: XMLLexer<'a>,
}

impl<'a> XMLParser<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            lexer: XMLLexer::new(content),
        }
    }
    pub fn parse(&'a self) -> Result<Rc<XMLNode>, error::ParseError> {
        let mut node_stack: Vec<Rc<XMLNode>> = Vec::new();

        let first_tag = match self.lexer.next_token()?.kind {
            TokenKind::Tag(tag) => XMLTag::from(tag),
            _ => {
                return Err(error::ParseError::InvalidFirstToken);
            }
        };

        let first_node = Rc::new(XMLNode::new(first_tag));

        node_stack.push(Rc::clone(&first_node));

        while !self.lexer.end() {
            let cur_token = self.lexer.next_token()?;

            match cur_token.kind {
                TokenKind::Tag(tag) => match tag.kind {
                    TagKind::Opening => {
                        let _new_node = Rc::new(XMLNode::new(XMLTag::from(tag)));
                        let new_node = Rc::clone(&_new_node);
                        node_stack
                            .last()
                            .unwrap()
                            .children
                            .borrow_mut()
                            .push(Rc::clone(&new_node));
                        node_stack.push(Rc::clone(&new_node));
                    }
                    TagKind::Closing => {
                        let popped = match node_stack.pop() {
                            Some(node) => node,
                            None => {
                                return Err(error::ParseError::ClosingTagNeverOpened {
                                    obtained: tag.name.to_owned(),
                                    position: tag.pos,
                                });
                            }
                        };

                        if popped.tag.name != tag.name {
                            return Err(error::ParseError::UnexpectedClosingTag {
                                expected: popped.tag.name.to_owned(),
                                obtained: tag.name,
                                position: popped.tag._pos,
                            });
                        }
                    }
                },
                TokenKind::String => node_stack.last().unwrap().push_content(cur_token.text),
                TokenKind::Whitespace => {}
                TokenKind::EndOfFile => {
                    break;
                }
            }
        }
        Ok(first_node)
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::parsetag::TagKind;

    use super::*;

    #[test]
    fn test_tag_tokenization_success() {
        let text = "<xml> </tag1></tag2> <xml>";

        let test_lexer = XMLLexer::new(text);
        let mut obtained_tokens: Vec<DocToken> = Vec::new();

        while let Ok(tkn) = test_lexer.next_token() {
            match tkn.kind {
                TokenKind::EndOfFile => break,
                TokenKind::Whitespace => {}
                _ => obtained_tokens.push(tkn),
            }
        }

        let actual_tokens = vec![
            DocToken::new(
                "<xml>",
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("xml"),
                    HashMap::new(),
                    TagKind::Opening,
                    0,
                )),
                0,
            ),
            DocToken::new(
                "</tag1>",
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("tag1"),
                    HashMap::new(),
                    TagKind::Closing,
                    6,
                )),
                6,
            ),
            DocToken::new(
                "</tag2>",
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("tag2"),
                    HashMap::new(),
                    TagKind::Closing,
                    13,
                )),
                13,
            ),
            DocToken::new(
                "<xml>",
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("xml"),
                    HashMap::new(),
                    TagKind::Opening,
                    21,
                )),
                21,
            ),
        ];

        assert_eq!(obtained_tokens, actual_tokens);
    }

    #[test]
    fn test_tag_tokenization_failure_unterminated_angular_bracket() {
        let text = "<xml> <oopsi problem here";

        let test_lexer = XMLLexer::new(text);

        test_lexer.next_token().unwrap();
        test_lexer.next_token().unwrap();

        match test_lexer.next_token() {
            Ok(tkn) => panic!("Expected UnterminatedAngularBracket, got token: {:?}", tkn),
            Err(e) => match e {
                error::ParseError::UnterminatedAngularBracket(pos) => {
                    assert_eq!(pos, 6)
                }
                _ => panic!("Expected UnterminatedAngularBracket, got Err({:?})", e),
            },
        }
    }

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
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("xml"),
                    HashMap::new(),
                    TagKind::Opening,
                    0,
                )),
                0,
            ),
            DocToken::new(
                "< person  age='55'  >",
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("person"),
                    HashMap::from([(String::from("age"), String::from("55"))]),
                    TagKind::Opening,
                    6,
                )),
                6,
            ),
            DocToken::new("David", TokenKind::String, 28),
            DocToken::new(
                "< / person >",
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("person"),
                    HashMap::new(),
                    TagKind::Closing,
                    36,
                )),
                36,
            ),
            DocToken::new(
                "< / xml  >",
                TokenKind::Tag(BaseXMLTag::new(
                    String::from("xml"),
                    HashMap::new(),
                    TagKind::Closing,
                    48,
                )),
                48,
            ),
        ];
        assert_eq!(parsed_tokens, actual_tokens);
    }
}
