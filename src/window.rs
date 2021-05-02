use crate::dom::Node;

#[derive(Debug)]
pub struct Window {
    pub name: String,
    pub document: Option<Node>,
}
