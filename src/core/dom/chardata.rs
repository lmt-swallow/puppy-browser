use super::{Node, NodeType};

// interface `CharacterData`
// definition: https://dom.spec.whatwg.org/#interface-characterdata

pub trait CharacterData {}

#[derive(Debug, PartialEq)]
pub struct Text {
    pub data: String,
}
impl Text {
    pub fn new(text: String) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Text(Text { data: text }),
            children: vec![],
        })
    }
}
impl CharacterData for Text {}
