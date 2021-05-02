// `Node` interface
// definition: https://dom.spec.whatwg.org/#interface-node
#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub child_nodes: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Element(super::element::Element),
    Text(super::chardata::Text),
    Document(super::document::Document),
}

#[allow(dead_code)]
impl Node {
    pub fn append_child(&mut self, n: Node) -> &Node {
        self.child_nodes.push(n);
        self.child_nodes.last().unwrap()
    }

    pub fn inner_text(&self) -> String {
        self.child_nodes
            .iter()
            .clone()
            .into_iter()
            .map(|node| match node {
                Node {
                    node_type: NodeType::Text(t),
                    ..
                } => t.data.clone(),
                n => n.inner_text(),
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

#[cfg(test)]
mod tests {
    use crate::dom::{AttrMap, Element};

    #[test]
    fn test_append_child() {
        let mut node = Element::new("p".to_string(), AttrMap::new(), vec![]);

        node.append_child(Element::new("p".to_string(), AttrMap::new(), vec![]));
        assert_eq!(node.child_nodes.len(), 1);

        node.append_child(Element::new("p".to_string(), AttrMap::new(), vec![]));
        assert_eq!(node.child_nodes.len(), 2)
    }
}