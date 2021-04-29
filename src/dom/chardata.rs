use super::{Node, NodeType};

// interface `CharacterData`
// definition: https://dom.spec.whatwg.org/#interface-characterdata

pub trait CharacterData {}

#[derive(Debug, PartialEq)]
pub struct Text {
    pub data: String,
}
impl Text {
    pub fn new(text: String) -> Node {
        Node {
            node_type: NodeType::Text(Text { data: text }),
            child_nodes: vec![],
        }
    }
}
impl CharacterData for Text {}
