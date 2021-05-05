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
#[derive(Debug, PartialEq)]
pub struct StyledNode<'a> {
    pub node_type: &'a NodeType,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

impl<'a> StyledNode<'a> {
    pub fn display(&self) -> Display {
        match self.properties.get("display") {
            Some(CSSValue::Keyword(s)) => match s.as_str() {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
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
        css::{Declaration, Rule, SimpleSelector, Stylesheet},
        dom::{AttrMap, Element},
    };

    #[test]
    fn test_properties() {
        let e = &Element::new("p".to_string(), AttrMap::new(), vec![]);
        let styled_e: StyledNode<'_> = to_styled_node(
            e,
            &Stylesheet::new(vec![Rule {
                selectors: vec![SimpleSelector::UniversalSelector],
                declarations: vec![Declaration {
                    name: "display".to_string(),
                    value: CSSValue::Keyword("block".to_string()),
                }],
            }]),
        );
        assert_eq!(
            styled_e,
            StyledNode {
                node_type: &e.node_type,
                properties: [(
                    "display".to_string(),
                    CSSValue::Keyword("block".to_string())
                )]
                .iter()
                .cloned()
                .collect(),
                children: vec![],
            }
        );
    }
}
