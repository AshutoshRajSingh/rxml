mod api;
pub mod error;
mod parsedoc;
mod parsetag;

use api::XMLNode;
use error::ParseError;
use parsedoc::XMLParser;
use std::rc::Rc;

pub struct RXML {
    content: String,
}

impl RXML {
    pub fn new(content: String) -> Self {
        Self { content }
    }
    pub fn parse(&self) -> Result<Rc<XMLNode>, ParseError> {
        let parser = XMLParser::new(self.content.as_str());
        return Ok(parser.parse()?);
    }
}
