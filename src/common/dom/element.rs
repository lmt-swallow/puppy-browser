use super::node::{Node, NodeType};
use std::collections::HashMap;

pub type AttrMap = HashMap<String, String>;

// `Element` interface
// definition: https://dom.spec.whatwg.org/#interface-element
#[derive(Debug, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl Element {
    pub fn new(name: String, attributes: AttrMap, children: Vec<Node>) -> Node {
        Node {
            node_type: NodeType::Element(Element {
                tag_name: name,
                attributes: attributes,
            }),
            children,
        }
    }
}
