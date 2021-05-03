use crate::common::{CSSValue, PropertyMap, StyledNode};

// `Node` interface
// definition: https://dom.spec.whatwg.org/#interface-node
#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Element(super::element::Element),
    Text(super::chardata::Text),
    Document(super::document::Document),
}

impl Node {
    pub fn append_child(&mut self, n: Node) -> &Node {
        self.children.push(n);
        self.children.last().unwrap()
    }

    pub fn inner_text(&self) -> String {
        self.children
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

    pub fn document_element(&self) -> &Self {
        match self.node_type {
            NodeType::Document(ref _document) => {
                assert_eq!(self.children.len(), 1);
                self.children.get(0).unwrap()
            }
            _ => {
                panic!("failed to extract documentElement");
            }
        }
    }

    pub fn document_element_mut(&mut self) -> &mut Node {
        match self.node_type {
            NodeType::Document(ref _document) => {
                assert_eq!(self.children.len(), 1);
                self.children.get_mut(0).unwrap()
            }
            _ => {
                panic!("failed to extract documentElement");
            }
        }
    }

    pub fn get_inline_scripts_recursively(&self) -> Vec<String> {
        match self.node_type {
            NodeType::Document(ref _document) => self
                .children
                .iter()
                .map(|node| node.get_inline_scripts_recursively())
                .collect::<Vec<Vec<String>>>()
                .into_iter()
                .flatten()
                .collect(),
            NodeType::Element(ref element) => match element.tag_name.as_str() {
                "script" => vec![self.inner_text()],
                _ => self
                    .children
                    .iter()
                    .map(|node| node.get_inline_scripts_recursively())
                    .collect::<Vec<Vec<String>>>()
                    .into_iter()
                    .flatten()
                    .collect(),
            },
            _ => {
                vec![]
            }
        }
    }
}

// TODO (enhancement): link with CSS here
impl<'a> Into<StyledNode<'a>> for &'a Node {
    fn into(self) -> StyledNode<'a> {
        let mut props = PropertyMap::new();
        let mut children = self.children.iter().map(|x| x.into()).collect();

        // set default styles
        match &self.node_type {
            NodeType::Element(e) => match e.tag_name.as_str() {
                "script" => {
                    props.insert("display".to_string(), CSSValue::Keyword("none".to_string()));
                }
                "div" => {
                    props.insert(
                        "display".to_string(),
                        CSSValue::Keyword("block".to_string()),
                    );
                }
                "button" | "a" => {
                    children = vec![];
                }
                _ => {}
            },
            _ => {}
        };

        StyledNode {
            node: self,
            properties: props,
            children: children,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dom::{AttrMap, Element};

    #[test]
    fn test_append_child() {
        let mut node = Element::new("p".to_string(), AttrMap::new(), vec![]);

        node.append_child(Element::new("p".to_string(), AttrMap::new(), vec![]));
        assert_eq!(node.children.len(), 1);

        node.append_child(Element::new("p".to_string(), AttrMap::new(), vec![]));
        assert_eq!(node.children.len(), 2)
    }
}
