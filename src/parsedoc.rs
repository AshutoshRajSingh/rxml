use crate::{
    error,
    parsetag::{TagKind, TagParser, XMLTag},
};
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::mem::discriminant;
use std::rc::Rc;

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

#[derive(Debug)]
pub struct XMLDocNode {
    content: RefCell<String>,
    pub tag: XMLTag,
    pub children: RefCell<Vec<Rc<XMLDocNode>>>,
}

impl XMLDocNode {
    fn new(tag: XMLTag) -> Self {
        Self {
            content: RefCell::new(String::new()),
            children: RefCell::new(Vec::new()),
            tag,
        }
    }
}

impl PartialEq for XMLDocNode {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content && self.children == other.children && self.tag == other.tag
    }
}

pub struct XMLParser<'a> {
    lexer: XMLLexer<'a>,
    tokens: RefCell<Vec<DocToken<'a>>>,
}

impl<'a> XMLParser<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            lexer: XMLLexer::new(content),
            tokens: RefCell::new(Vec::new()),
        }
    }
    pub fn tokenize(&'a self) -> Result<(), error::ParseError> {
        loop {
            let token = self.lexer.next_token()?;
            if let TokenKind::EndOfFile = token.kind {
                break;
            }
            if let TokenKind::Whitespace = token.kind {
            } else {
                self.tokens.borrow_mut().push(token);
            }
        }
        Ok(())
    }
    pub fn parse(&'a self) -> Result<Rc<XMLDocNode>, error::ParseError> {
        self.tokenize()?;
        let mut node_stack: Vec<Rc<XMLDocNode>> = Vec::new();

        let tokens = self.tokens.borrow();

        let first_tag: XMLTag;

        let _first = match tokens.first() {
            Some(tkn) => match &tkn.kind {
                TokenKind::Tag(tag) => match tag.kind {
                    TagKind::Opening => {
                        first_tag = tag.to_owned();
                        tkn
                    }
                    _ => {
                        return Err(error::ParseError::InvalidFirstToken);
                    }
                },
                _ => return Err(error::ParseError::InvalidFirstToken),
            },
            None => {
                return Err(error::ParseError::NoTokensToParse);
            }
        };

        let mut _root = XMLDocNode::new(first_tag);
        let root = Rc::new(_root);

        node_stack.push(Rc::clone(&root));

        for token in self.tokens.borrow()[1..].iter() {
            match &token.kind {
                TokenKind::String => {
                    let target_node = node_stack.last().unwrap();
                    target_node.content.borrow_mut().push_str(token.text);
                }
                TokenKind::Tag(tag) => {
                    if let TagKind::Opening = tag.kind {
                        let _new_node = XMLDocNode::new(tag.to_owned());
                        let new_node = Rc::new(_new_node);

                        node_stack
                            .last_mut()
                            .unwrap()
                            .children
                            .borrow_mut()
                            .push(Rc::clone(&new_node));
                        node_stack.push(Rc::clone(&new_node));
                    } else {
                        let popped = match node_stack.pop() {
                            Some(node) => node,
                            None => {
                                return Err(error::ParseError::ClosingTagNeverOpened {
                                    obtained: tag.name.to_owned(),
                                    position: 0,
                                })
                            }
                        };

                        if popped.tag.name != tag.name {
                            return Err(error::ParseError::UnexpectedClosingTag {
                                expected: popped.tag.name.to_owned(),
                                obtained: tag.name.to_owned(),
                                position: 0,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(root)
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
                    0,
                )),
                0,
            ),
            DocToken::new(
                "< person  age='55'  >",
                TokenKind::Tag(XMLTag::new(
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
                TokenKind::Tag(XMLTag::new(
                    String::from("person"),
                    HashMap::new(),
                    TagKind::Closing,
                    36,
                )),
                36,
            ),
            DocToken::new(
                "< / xml  >",
                TokenKind::Tag(XMLTag::new(
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

    #[test]
    fn test_xml_parsing() {
        let test_str = "<xml version='1.0' encoding='utf-8' sapghet> <name> John <child age='55'> Mike </child> </name> </xml>";

        let test_parser = XMLParser::new(test_str);

        let root = test_parser.parse().unwrap();

        println!("{:?}", root);
    }
}
