use std::collections::HashMap;

use super::{
    dom::{Node, NodeType},
};
// See: https://www.w3.org/TR/css-values-3/#component-types
#[derive(Debug)]
pub enum CSSValue {
    Keyword(String),
}

pub type PropertyMap = HashMap<String, CSSValue>;

#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node: &'a Box<Node>,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

pub enum Display {
    Inline,
    Block,
    None,
}

impl<'a> StyledNode<'a> {
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(CSSValue::Keyword(s)) => match s.as_str() {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    pub fn value(&self, name: &str) -> Option<&CSSValue> {
        self.properties.get(name)
    }
}

// TODO (enhancement): link with CSS here
impl<'a> From<&'a Box<Node>> for StyledNode<'a> {
    fn from(node: &'a Box<Node>) -> Self {
        // prepare basic information of StyledNode
        let mut props = PropertyMap::new();
        let mut children = node.children.iter().map(|x| x.into()).collect();

        // set default styles
        match &node.node_type {
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
                "a" => {
                    children = vec![];
                }
                _ => {}
            },
            _ => {}
        };

        // all set :-)
        StyledNode {
            node,
            properties: props,
            children: children,
        }
    }
}
