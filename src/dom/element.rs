use super::node::{Node, NodeType};
use std::collections::HashMap;

pub type AttrMap = HashMap<String, String>;

// `Element` interface
// definition: https://dom.spec.whatwg.org/#interface-element
#[derive(Debug, PartialEq)]
pub struct Element {
    tag_name: String,
    attributes: AttrMap,
}

impl Element {
    pub fn new(name: String, attributes: AttrMap, child_nodes: Vec<Node>) -> Node {
        Node {
            node_type: NodeType::Element(Element {
                tag_name: name,
                attributes: attributes,
            }),
            child_nodes: child_nodes,
        }
    }
}
