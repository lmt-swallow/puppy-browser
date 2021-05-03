use std::collections::HashMap;

use super::{
    dom::Node,
    layout::{BoxType, LayoutBox},
};
// See: https://www.w3.org/TR/css-values-3/#component-types
#[derive(Debug)]
pub enum CSSValue {
    Keyword(String),
}

pub type PropertyMap = HashMap<String, CSSValue>;

#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
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
        self.properties.get(name).map(|v| v.clone())
    }
}

impl<'a> Into<LayoutBox<'a>> for &'a StyledNode<'a> {
    fn into(self) -> LayoutBox<'a> {
        let box_type = match self.display() {
            Display::Block => BoxType::BlockNode(&self),
            Display::Inline => BoxType::InlineNode(&self),
            Display::None => BoxType::NoneNode(&self),
        };
        let mut layout = LayoutBox {
            box_type: box_type,
            children: vec![],
        };

        for node in &self.children {
            match node.display() {
                Display::Block => {
                    layout.children.push(node.into());
                }
                Display::Inline => {
                    layout.inline_container().children.push(node.into());
                }
                Display::None => {}
            }
        }

        layout
    }
}
