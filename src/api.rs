use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::cmp::PartialEq;
use crate::parsetag::BaseXMLTag;

#[derive(Debug)]
pub struct XMLTag {
    pub _pos: usize,
    pub name: String,
    pub attributes: HashMap<String, String>,
}

impl XMLTag {
    pub fn new(_pos: usize, name: String, attributes: HashMap<String, String>) -> Self {
        Self {
            _pos,
            name,
            attributes,
        }
    }
    pub fn from(base: BaseXMLTag) -> Self {
        Self { _pos: base.pos, name: String::from(base.name), attributes: base.attribs }
    }
}

impl PartialEq for XMLTag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.attributes == other.attributes && self._pos == other._pos
    }
}

#[derive(Debug)]
pub struct XMLNode {
    pub tag: XMLTag,
    pub content: RefCell<String>,
    pub children: RefCell<Vec<Rc<XMLNode>>>,
}

impl XMLNode {
    pub fn new(tag: XMLTag) -> Self {
        Self {
            tag,
            content: RefCell::new(String::new()),
            children: RefCell::new(Vec::new()),
        }
    }
    pub fn append_child(&self, child: Rc<XMLNode>) {
        self.children.borrow_mut().push(child);
    }
    pub fn push_content(&self, content: &str) {
        self.content.borrow_mut().push_str(content);
    }
}

impl PartialEq for XMLNode {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.content == other.content && self.children == other.children
    }
}
