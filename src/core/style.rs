use std::collections::HashMap;

use super::{
    css::CSSValue,
    dom::{Node, NodeType},
};

pub type PropertyMap = HashMap<String, CSSValue>;

#[derive(Debug, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

/// `StyledNode` wraps `Node` with related CSS properties.
/// It forms a tree as `Node` does.
#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node_type: &'a NodeType,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

impl<'a> StyledNode<'a> {
    pub fn display(&self) -> Display {
        match self.get_style_property("display") {
            Some(CSSValue::Keyword(s)) => match s.as_str() {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    pub fn inner_text(&self) -> String {
        self.children
            .iter()
            .clone()
            .into_iter()
            .map(|node| match &node.node_type {
                NodeType::Text(t) => t.data.clone(),
                _ => node.inner_text(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn get_style_property(&self, name: &str) -> Option<&CSSValue> {
        self.properties.get(name)
    }

    pub fn set_style_property(&mut self, key: &str, value: CSSValue) -> Option<CSSValue> {
        self.properties.insert(key.to_string(), value)
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
            node_type: &node.node_type,
            properties: props,
            children: children,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        dom::{AttrMap, Element},
        style::{CSSValue, Display, StyledNode},
    };

    #[test]
    fn test_properties() {
        let e = &Element::new("p".to_string(), AttrMap::new(), vec![]);
        let mut styled_e: StyledNode<'_> = e.into();
        assert_eq!(
            styled_e.set_style_property("display", CSSValue::Keyword("block".to_string())),
            None
        );
        assert_eq!(
            styled_e.get_style_property("display"),
            Some(&CSSValue::Keyword("block".to_string()))
        );
        assert_eq!(
            styled_e.set_style_property("display", CSSValue::Keyword("inline".to_string())),
            Some(CSSValue::Keyword("block".to_string()))
        );
    }

    #[test]
    fn test_display() {
        let e = &Element::new("p".to_string(), AttrMap::new(), vec![]);
        let mut styled_e: StyledNode<'_> = e.into();

        styled_e.set_style_property("display", CSSValue::Keyword("block".to_string()));
        assert_eq!(styled_e.display(), Display::Block);

        styled_e.set_style_property("display", CSSValue::Keyword("inline".to_string()));
        assert_eq!(styled_e.display(), Display::Inline);

        styled_e.set_style_property("display", CSSValue::Keyword("none".to_string()));
        assert_eq!(styled_e.display(), Display::None);
    }
}
