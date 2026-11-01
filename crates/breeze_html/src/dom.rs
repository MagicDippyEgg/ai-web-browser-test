use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub type NodeRef = Rc<RefCell<Node>>;
pub type WeakNodeRef = Weak<RefCell<Node>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeData {
    Document,
    DocumentType {
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
    },
    Element(Element),
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub id: Option<String>,
    pub class_list: Vec<String>,
}

impl Element {
    pub fn new(tag_name: impl Into<String>, attributes: HashMap<String, String>) -> Self {
        let tag_name = tag_name.into().to_ascii_lowercase();
        let id = attributes.get("id").cloned();
        let class_list = attributes
            .get("class")
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();
        Self {
            tag_name,
            attributes,
            id,
            class_list,
        }
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
        if name == "id" {
            self.id = Some(value.to_string());
        } else if name == "class" {
            self.class_list = value.split_whitespace().map(|s| s.to_string()).collect();
        }
    }

    pub fn remove_attribute(&mut self, name: &str) {
        self.attributes.remove(name);
        if name == "id" {
            self.id = None;
        } else if name == "class" {
            self.class_list.clear();
        }
    }

    pub fn has_class(&self, class_name: &str) -> bool {
        self.class_list.iter().any(|c| c == class_name)
    }
}

pub struct Node {
    pub data: NodeData,
    pub parent: Option<WeakNodeRef>,
    pub children: Vec<NodeRef>,
}

impl Node {
    pub fn new(data: NodeData) -> NodeRef {
        Rc::new(RefCell::new(Self {
            data,
            parent: None,
            children: Vec::new(),
        }))
    }

    pub fn new_element(tag_name: &str, attributes: HashMap<String, String>) -> NodeRef {
        Self::new(NodeData::Element(Element::new(tag_name, attributes)))
    }

    pub fn new_text(text: &str) -> NodeRef {
        Self::new(NodeData::Text {
            content: text.to_string(),
        })
    }

    pub fn new_comment(comment: &str) -> NodeRef {
        Self::new(NodeData::Comment {
            content: comment.to_string(),
        })
    }

    pub fn new_document() -> NodeRef {
        Self::new(NodeData::Document)
    }
}

#[derive(Clone)]
pub struct Document {
    pub root: NodeRef,
}

impl Document {
    pub fn new() -> Self {
        Self {
            root: Node::new_document(),
        }
    }

    pub fn append_child(parent: &NodeRef, child: &NodeRef) {
        if let Some(old_parent_weak) = &child.borrow().parent {
            if let Some(old_parent) = old_parent_weak.upgrade() {
                old_parent.borrow_mut().children.retain(|c| !Rc::ptr_eq(c, child));
            }
        }

        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(Rc::clone(child));
    }

    pub fn remove_child(parent: &NodeRef, child: &NodeRef) -> bool {
        let mut p = parent.borrow_mut();
        let prev_len = p.children.len();
        p.children.retain(|c| !Rc::ptr_eq(c, child));
        if p.children.len() < prev_len {
            child.borrow_mut().parent = None;
            true
        } else {
            false
        }
    }

    pub fn insert_before(parent: &NodeRef, new_node: &NodeRef, reference_node: Option<&NodeRef>) {
        if let Some(ref_node) = reference_node {
            let mut p = parent.borrow_mut();
            if let Some(pos) = p.children.iter().position(|c| Rc::ptr_eq(c, ref_node)) {
                new_node.borrow_mut().parent = Some(Rc::downgrade(parent));
                p.children.insert(pos, Rc::clone(new_node));
                return;
            }
        }
        Self::append_child(parent, new_node);
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<NodeRef> {
        Self::find_first(&self.root, &|node| {
            if let NodeData::Element(elem) = &node.borrow().data {
                elem.id.as_deref() == Some(id)
            } else {
                false
            }
        })
    }

    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeRef> {
        let tag = tag_name.to_ascii_lowercase();
        let mut results = Vec::new();
        Self::find_all(&self.root, &|node| {
            if let NodeData::Element(elem) = &node.borrow().data {
                if tag == "*" || elem.tag_name == tag {
                    return true;
                }
            }
            false
        }, &mut results);
        results
    }

    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<NodeRef> {
        let mut results = Vec::new();
        Self::find_all(&self.root, &|node| {
            if let NodeData::Element(elem) = &node.borrow().data {
                elem.has_class(class_name)
            } else {
                false
            }
        }, &mut results);
        results
    }

    pub fn text_content(node: &NodeRef) -> String {
        let mut text = String::new();
        let n = node.borrow();
        match &n.data {
            NodeData::Text { content } => text.push_str(content),
            _ => {
                for child in &n.children {
                    text.push_str(&Self::text_content(child));
                }
            }
        }
        text
    }

    pub fn set_text_content(node: &NodeRef, text: &str) {
        let mut n = node.borrow_mut();
        n.children.clear();
        let text_node = Node::new_text(text);
        text_node.borrow_mut().parent = Some(Rc::downgrade(node));
        n.children.push(text_node);
    }

    fn find_first<F>(node: &NodeRef, predicate: &F) -> Option<NodeRef>
    where
        F: Fn(&NodeRef) -> bool,
    {
        if predicate(node) {
            return Some(Rc::clone(node));
        }
        for child in &node.borrow().children {
            if let Some(found) = Self::find_first(child, predicate) {
                return Some(found);
            }
        }
        None
    }

    fn find_all<F>(node: &NodeRef, predicate: &F, results: &mut Vec<NodeRef>)
    where
        F: Fn(&NodeRef) -> bool,
    {
        if predicate(node) {
            results.push(Rc::clone(node));
        }
        for child in &node.borrow().children {
            Self::find_all(child, predicate, results);
        }
    }
}
