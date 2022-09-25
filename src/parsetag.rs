use crate::error;
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
    ForwardSlash,
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
    pub fn next_token(&self) -> Result<TagToken, error::TagParseError> {
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
                    return Err(error::TagParseError::UnterminatedStringLiteral(start));
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
            while !self.end() && (self.current().is_alphanumeric() || self.current() == '_') {
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
        } else if self.current() == '/' {
            self.next();
            return Ok(TagToken::new(
                &self.content[start..start + 1],
                TokenKind::ForwardSlash,
                start,
            ));
        } else {
            self.next();

            while !self.current().is_whitespace() && !self.end() {
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

#[derive(Debug, Clone)]
pub enum TagKind {
    Opening,
    Closing,
}
#[derive(Debug, Clone)]
pub struct XMLTag {
    pub name: String,
    pub attribs: HashMap<String, String>,
    pub kind: TagKind,
}

impl XMLTag {
    pub fn new(name: String, attribs: HashMap<String, String>, kind: TagKind) -> Self {
        Self {
            name,
            attribs,
            kind,
        }
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

    fn tokenize(&'a self) -> Result<(), error::TagParseError> {
        loop {
            let cur_token = self.lexer.next_token()?;

            match cur_token.kind {
                TokenKind::EndOfLine => {
                    break;
                }
                TokenKind::Whitespace => {}
                _ => {
                    self.tokens.borrow_mut().push(cur_token);
                }
            }
        }
        Ok(())
    }

    fn peek(&self, offset: i64) -> Result<Ref<'a, TagToken>, error::TagParseError> {
        let pos_copy = *self.position.borrow() as i64;
        if pos_copy + offset < 1 || pos_copy + offset >= self.tokens.borrow().len() as i64 {
            return Err(error::TagParseError::PeekOutOfBounds {
                offset: offset,
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

    pub fn parse(&'a self) -> Result<XMLTag, error::TagParseError> {
        self.tokenize()?;
        let first = self.cur_token();

        let name: String;
        let kind: TagKind;

        if let TokenKind::String = first.kind {
            kind = TagKind::Opening;
            name = String::from(first.text);
        } else if let TokenKind::ForwardSlash = first.kind {
            kind = TagKind::Closing;
            self.next();

            let second = self.cur_token();

            if let TokenKind::String = second.kind {
                name = String::from(second.text);
            } else {
                return Err(error::TagParseError::InvalidFirstToken);
            }
        } else {
            return Err(error::TagParseError::InvalidFirstToken);
        }

        let mut attribs: HashMap<String, String> = HashMap::new();

        while !self.end() {
            let cur = self.cur_token();
            if let TokenKind::Equals = cur.kind {
                let left = match self.peek(-1) {
                    Ok(tkn) => tkn,
                    Err(_) => {
                        return Err(error::TagParseError::NoTokenAtLocation {
                            expected_kind: String::from("String"),
                            direction: String::from("left"),
                            current: String::from("Equals"),
                        });
                    }
                };
                let right = match self.peek(1) {
                    Ok(tkn) => tkn,
                    Err(_) => {
                        return Err(error::TagParseError::NoTokenAtLocation {
                            expected_kind: String::from("StringLiteral"),
                            direction: String::from("right"),
                            current: String::from("Equals"),
                        });
                    }
                };
                if let (TokenKind::String, TokenKind::StringLiteral) = (&left.kind, &right.kind) {
                    let k = String::from(left.text);
                    let v = String::from(&right.text[1..right.text.len() - 1]);
                    attribs.insert(k, v);
                } else {
                    return Err(error::TagParseError::UnexpectedTagToken);
                }
            }
            self.next();
        }
        Ok(XMLTag::new(name, attribs, kind))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::TagParseError;

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
            tokens.push(token);
        }

        assert_eq!(tokens, actual_tokens);
    }

    #[test]
    fn test_string_literal_tokenization() {
        let text = "'literal1' 'literal2''literal3' \"literal4\"\"literal5\"";

        let test_lexer = TagLexer::new(text);
        let mut obtained_tokens: Vec<TagToken> = Vec::new();

        let actual_tokens = vec![
            TagToken::new("'literal1'", TokenKind::StringLiteral, 0),
            TagToken::new("'literal2'", TokenKind::StringLiteral, 11),
            TagToken::new("'literal3'", TokenKind::StringLiteral, 21),
            TagToken::new("\"literal4\"", TokenKind::StringLiteral, 32),
            TagToken::new("\"literal5\"", TokenKind::StringLiteral, 42),
        ];

        while let Ok(token) = test_lexer.next_token() {
            if let TokenKind::EndOfLine = token.kind {
                break;
            } else if let TokenKind::Whitespace = token.kind {
                continue;
            }
            obtained_tokens.push(token);
        }
        assert_eq!(obtained_tokens, actual_tokens);
    }

    #[test]
    fn test_equals_tokenization() {
        let text = "= == ===";

        let test_lexer = TagLexer::new(text);

        let mut obtained_tokens: Vec<TagToken> = Vec::new();

        let actual_tokens = vec![
            TagToken::new("=", TokenKind::Equals, 0),
            TagToken::new("=", TokenKind::Equals, 2),
            TagToken::new("=", TokenKind::Equals, 3),
            TagToken::new("=", TokenKind::Equals, 5),
            TagToken::new("=", TokenKind::Equals, 6),
            TagToken::new("=", TokenKind::Equals, 7),
        ];

        while let Ok(token) = test_lexer.next_token() {
            if let TokenKind::Whitespace = token.kind {
                continue;
            } else if let TokenKind::EndOfLine = token.kind {
                break;
            }
            obtained_tokens.push(token);
        }

        assert_eq!(obtained_tokens, actual_tokens);
    }

    #[test]
    fn test_forward_slash_tokenization() {
        let text = "/ // ///";

        let test_lexer = TagLexer::new(text);

        let mut obtained_tokens: Vec<TagToken> = Vec::new();

        let actual_tokens = vec![
            TagToken::new("/", TokenKind::ForwardSlash, 0),
            TagToken::new("/", TokenKind::ForwardSlash, 2),
            TagToken::new("/", TokenKind::ForwardSlash, 3),
            TagToken::new("/", TokenKind::ForwardSlash, 5),
            TagToken::new("/", TokenKind::ForwardSlash, 6),
            TagToken::new("/", TokenKind::ForwardSlash, 7),
        ];

        while let Ok(token) = test_lexer.next_token() {
            if let TokenKind::Whitespace = token.kind {
                continue;
            } else if let TokenKind::EndOfLine = token.kind {
                break;
            }
            obtained_tokens.push(token);
        }

        assert_eq!(obtained_tokens, actual_tokens);
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
    fn test_closing_tag_lexer() {
        let text = "/tagname";
        let actual_tokens = vec![
            TagToken::new("/", TokenKind::ForwardSlash, 0),
            TagToken::new("tagname", TokenKind::String, 1),
        ];

        let test_lexer = TagLexer::new(text);
        let mut obtained_tokens: Vec<TagToken> = Vec::new();
        while let Ok(token) = test_lexer.next_token() {
            if let TokenKind::EndOfLine = token.kind {
                break;
            }
            obtained_tokens.push(token);
        }
        assert_eq!(actual_tokens, obtained_tokens);
    }

    #[test]
    fn test_opening_tag_parser_success() {
        let text = "<tagname attribute1='value1'>";

        let test_parser = TagParser::new(text);
        let test_tag = test_parser.parse().unwrap();

        let mut actual_attribs: HashMap<String, String> = HashMap::new();

        actual_attribs.insert(String::from("attribute1"), String::from("value1"));

        assert_eq!(test_tag.name, String::from("tagname"));
        assert_eq!(test_tag.attribs, actual_attribs);
    }

    #[test]
    fn test_opening_tag_parser_failure() {
        let text = "<tagname attribute1 = 'oopsie no closing quote>";

        let test_parser = TagParser::new(text);

        match test_parser.parse() {
            Ok(_tag) => panic!("Blimey mate it was supposed to fail 'ere"),
            Err(e) => match e {
                error::TagParseError::UnterminatedStringLiteral(_loc) => {}
                _ => {
                    panic!(
                        "Bugger, got wrong error, expected UnterminatedStringLiteral, got {:?}",
                        e
                    )
                }
            },
        }
    }

    #[test]
    fn test_closing_tag_parser_success() {
        let text = "</tagname>";

        let test_parser = TagParser::new(text);
        let test_tag = test_parser.parse().unwrap();

        assert_eq!(test_tag.name, "tagname");

        match test_tag.kind {
            TagKind::Closing => {}
            _ => panic!("Inputted closing tag string, got opening tag output"),
        }
    }

    #[test]
    fn test_tag_attribute_parsing_success() {
        let text = "<person name='John' age=\"55\" ssn='67771020'>";

        let test_parser = TagParser::new(text);

        let obtained_tag = test_parser.parse().unwrap();

        let actual_tag = XMLTag::new(
            String::from("person"),
            HashMap::from([
                (String::from("name"), String::from("John")),
                (String::from("age"), String::from("55")),
                (String::from("ssn"), String::from("67771020")),
            ]),
            TagKind::Opening,
        );

        assert_eq!(obtained_tag.name, actual_tag.name);
        assert_eq!(obtained_tag.attribs, actual_tag.attribs);
        assert_eq!(
            discriminant(&obtained_tag.kind),
            discriminant(&actual_tag.kind)
        );
    }

    #[test]
    fn test_attribute_parsing_failure_no_token_on_right() {
        let text = "<tagname attrib1=>";

        let test_parser = TagParser::new(text);

        match test_parser.parse() {
            Ok(tag) => panic!("Expected NoTokenAtLocation, got tag: {:?}", tag),
            Err(e) => match e {
                TagParseError::NoTokenAtLocation {
                    expected_kind: _,
                    direction: _,
                    current: _,
                } => {}
                _ => panic!("Expected NoTokenAtLocation got Err({:?})", e),
            },
        }
    }

    #[test]
    fn test_attribute_parsing_failure_no_token_on_left() {
        let text = "<tagname = 'attrib'>";

        let test_parser = TagParser::new(text);

        match test_parser.parse() {
            Ok(tag) => panic!("Expected NoTokenAtLocation, got tag: {:?}", tag),
            Err(e) => match e {
                TagParseError::NoTokenAtLocation {
                    expected_kind: _,
                    direction: _,
                    current: _,
                } => {}
                _ => panic!("Expected NoTokenAtLocation, got Err({:?})", e),
            },
        }
    }

    #[test]
    fn test_attribute_parsing_failure_wrong_token_on_left() {
        let text = "<tagname 'attrib1' = 'attrib2'>";

        let test_parser = TagParser::new(text);

        match test_parser.parse() {
            Ok(tag) => panic!("Expected UnexpectedTagToken, got tag: {:?}", tag),
            Err(e) => match e {
                TagParseError::UnexpectedTagToken => {}
                _ => panic!("Expected UnexpectedTagToken got: Err({:?})", e),
            },
        }
    }

    #[test]
    fn test_attribute_parsing_failure_wrong_token_on_right() {
        let text = "<tagname var1 = oopsie_wongr_heer>";

        let test_parser = TagParser::new(text);

        match test_parser.parse() {
            Ok(tag) => panic!("Expected UnexpectedTagToken, got tag: {:?}", tag),
            Err(e) => match e {
                TagParseError::UnexpectedTagToken => {}
                _ => panic!("Expected UnexpectedTagToken got: Err({:?})", e),
            },
        }
    }
}
