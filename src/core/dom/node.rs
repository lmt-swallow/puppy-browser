use super::super::html::parse_without_normalziation;
use std::error::Error;

// `Node` interface
// definition: https://dom.spec.whatwg.org/#interface-node
#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Box<Node>>,
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Element(super::element::Element),
    Text(super::chardata::Text),
}

impl Node {
    pub fn append_child(&mut self, n: Box<Node>) -> &Box<Node> {
        self.children.push(n);
        self.children.last().unwrap()
    }

    pub fn inner_text(&self) -> String {
        self.children
            .iter()
            .clone()
            .into_iter()
            .map(|node| match &**node {
                Node {
                    node_type: NodeType::Text(t),
                    ..
                } => t.data.clone(),
                n => n.inner_text(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn inner_html(&self) -> String {
        self.children
            .iter()
            .clone()
            .into_iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn set_inner_html(&mut self, html: String) -> Result<(), Box<dyn Error>> {
        let node = parse_without_normalziation(html.as_bytes().into())?;
        self.children = node;
        Ok(())
    }

    pub fn children_map<T, F>(&mut self, f: &mut F) -> Vec<T>
    where
        F: FnMut(&mut Box<Node>) -> T,
    {
        let mut v: Vec<T> = vec![];

        for child in &mut self.children {
            v.push(f(child));
            v.extend(child.children_map(f));
        }

        v
    }

    pub fn children_filter_map<T, F>(&mut self, f: &mut F) -> Vec<T>
    where
        F: FnMut(&mut Box<Node>) -> Option<T>,
    {
        let mut v: Vec<T> = vec![];

        for child in &mut self.children {
            if let Some(r) = f(child) {
                v.push(r);
            }
            v.extend(child.children_filter_map(f));
        }

        v
    }

 
    pub fn get_inline_scripts_recursively(&self) -> Vec<String> {
        match self.node_type {
            NodeType::Element(ref element) => match element.tag_name.as_str() {
                "script" => return vec![self.inner_text()],
                _ => (),
            },
            _ => (),
        };

        self.children
            .iter()
            .map(|node| node.get_inline_scripts_recursively())
            .collect::<Vec<Vec<String>>>()
            .into_iter()
            .flatten()
            .collect()
    }
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match self.node_type {
            NodeType::Element(ref e) => {
                let attrs = e
                    .attributes
                    .iter()
                    .clone()
                    .into_iter()
                    .map(|(k, v)| {
                        // TODO (security): do this securely! This might causes mXSS.
                        format!("{}=\"{}\"", k, v)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let children = self
                    .children
                    .iter()
                    .clone()
                    .into_iter()
                    .map(|node| node.inner_html())
                    .collect::<Vec<_>>()
                    .join("");
                format!("<{} {}>{}</{}>", e.tag_name, attrs, children, e.tag_name)
            }
            NodeType::Text(ref t) => t.data.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::dom::Text,
        dom::{AttrMap, Element},
    };

    #[test]
    fn test_append_child() {
        let mut node = Element::new("p".to_string(), AttrMap::new(), vec![]);

        node.append_child(Element::new("p".to_string(), AttrMap::new(), vec![]));
        assert_eq!(node.children.len(), 1);

        node.append_child(Element::new("p".to_string(), AttrMap::new(), vec![]));
        assert_eq!(node.children.len(), 2)
    }

    #[test]
    fn test_inner_text() {
        {
            let node = Element::new(
                "p".to_string(),
                AttrMap::new(),
                vec![Text::new("hello world".to_string())],
            );
            assert_eq!(node.inner_text(), "hello world".to_string());
        }
        {
            let node = Element::new(
                "div".to_string(),
                AttrMap::new(),
                vec![
                    Text::new("hello world".to_string()),
                    Element::new(
                        "p".to_string(),
                        AttrMap::new(),
                        vec![
                            Element::new(
                                "p".to_string(),
                                AttrMap::new(),
                                vec![Text::new("1".to_string())],
                            ),
                            Element::new("p".to_string(), AttrMap::new(), vec![]),
                            Element::new(
                                "p".to_string(),
                                AttrMap::new(),
                                vec![Text::new("3".to_string())],
                            ),
                        ],
                    ),
                ],
            );
            assert_eq!(node.inner_text(), "hello world13".to_string());
        }
    }
}
