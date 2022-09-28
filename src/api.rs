use crate::parsetag::BaseXMLTag;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
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
        Self {
            _pos: base.pos,
            name: String::from(base.name),
            attributes: base.attribs,
        }
    }
}

impl PartialEq for XMLTag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.attributes == other.attributes && self._pos == other._pos
    }
}

impl Display for XMLTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} {:?}>", self.name, self.attributes)
    }
}

#[derive(Debug, Clone)]
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
    fn pretty_format(&self) -> String {
        let mut out_string = String::new();
        let mut node_stack: Vec<(usize, Rc<XMLNode>)> = Vec::new();

        let depth = 0;

        node_stack.push((depth, Rc::new(self.to_owned())));
        while !node_stack.is_empty() {
            let mut prefix = String::new();

            let (depth, top) = match node_stack.pop() {
                Some(v) => v,
                None => {
                    return String::new();
                }
            };

            for _ in 0..depth {
                prefix.push_str(" ")
            }

            let suffix: String;

            if !top.content.borrow().is_empty() {
                suffix = format!(
                    "{}{} '{}'\n",
                    prefix,
                    top.tag.to_string(),
                    top.content.borrow()
                );
            } else {
                suffix = format!("{}{}\n", prefix, top.tag.to_string());
            }

            out_string.push_str(&suffix);

            for child in top.children.borrow().iter() {
                node_stack.push((depth + 1, Rc::clone(child)));
            }
        }
        out_string
    }
}

impl PartialEq for XMLNode {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.content == other.content && self.children == other.children
    }
}

impl Display for XMLNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_format())
    }
}
