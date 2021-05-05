use super::{
    css::{self, CSSValue, Stylesheet},
    dom::{Document, Node, NodeType},
};
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, CSSValue>;

#[derive(Debug, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

/// `StyledDocument` wraps `Document` with related CSS properties.
#[derive(Debug)]
pub struct StyledDocument<'a> {
    pub document_element: StyledNode<'a>,
}

pub fn to_styled_document<'a>(document: &'a Document) -> StyledDocument<'a> {
    let styles = document.get_style_inners().join("\n");
    let stylesheet = css::parse(styles).unwrap_or(Stylesheet::new(vec![]));
    let document_element = to_styled_node(&document.document_element, &stylesheet);

    StyledDocument {
        document_element: document_element,
    }
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

fn to_styled_node<'a>(node: &'a Box<Node>, stylesheet: &Stylesheet) -> StyledNode<'a> {
    // prepare basic information of StyledNode
    let mut props = PropertyMap::new();
    let children = match &node.node_type {
        NodeType::Element(e) => match e.tag_name.as_str() {
            "a" => {
                vec![]
            }
            _ => to_styled_nodes(&node.children, stylesheet),
        },
        _ => to_styled_nodes(&node.children, stylesheet),
    };

    // match CSS rules
    for matched_rule in stylesheet.rules.iter().filter(|r| r.matches(node)) {
        for declaration in &matched_rule.declarations {
            props.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    // all set :-)
    StyledNode {
        node_type: &node.node_type,
        properties: props,
        children: children,
    }
}

fn to_styled_nodes<'a>(nodes: &'a Vec<Box<Node>>, stylesheet: &Stylesheet) -> Vec<StyledNode<'a>> {
    nodes
        .iter()
        .map(|x| to_styled_node(x, stylesheet))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        css::Stylesheet,
        dom::{AttrMap, Element},
    };

    #[test]
    fn test_properties() {
        let e = &Element::new("p".to_string(), AttrMap::new(), vec![]);
        let mut styled_e: StyledNode<'_> = to_styled_node(e, &Stylesheet::new(vec![]));
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
        let mut styled_e: StyledNode<'_> = to_styled_node(e, &Stylesheet::new(vec![]));

        styled_e.set_style_property("display", CSSValue::Keyword("block".to_string()));
        assert_eq!(styled_e.display(), Display::Block);

        styled_e.set_style_property("display", CSSValue::Keyword("inline".to_string()));
        assert_eq!(styled_e.display(), Display::Inline);

        styled_e.set_style_property("display", CSSValue::Keyword("none".to_string()));
        assert_eq!(styled_e.display(), Display::None);
    }
}
