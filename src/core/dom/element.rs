//! This module defines some interfaces related to `Element` interface.

use super::node::{Node, NodeType};
use std::collections::HashMap;

pub type AttrMap = HashMap<String, String>;

/// `Element` is a kind of `Node` defined at [DOM Standard](https://dom.spec.whatwg.org/#interface-element).
#[derive(Debug, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl Element {
    pub fn new(name: String, attributes: AttrMap, children: Vec<Box<Node>>) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Element(Element {
                tag_name: name,
                attributes: attributes,
            }),
            children,
        })
    }

    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn attributes(&self) -> Vec<(String, String)> {
        self.attributes
            .iter()
            .clone()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
}
