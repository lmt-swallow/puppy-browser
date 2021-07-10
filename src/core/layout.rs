//! This module includes some implementations on rendering tree.

use super::style::{Display, StyledDocument};
use super::{
    dom::NodeType,
    style::{PropertyMap, StyledNode},
};

pub struct LayoutDocument<'a> {
    pub top_box: LayoutBox<'a>,
}

#[derive(Debug)]
pub struct LayoutBox<'a> {
    pub box_type: BoxType,
    pub box_props: Option<BoxProps<'a>>,
    pub children: Vec<LayoutBox<'a>>,
}

#[derive(Debug)]
pub enum BoxType {
    BlockBox,
    InlineBox,
    NoneBox,
    AnonymousBox,
}

#[derive(Debug)]
pub struct BoxProps<'a> {
    pub node_type: &'a NodeType,
    pub properties: PropertyMap,
}

impl<'a> LayoutBox<'a> {
    pub fn inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineBox | BoxType::NoneBox | BoxType::AnonymousBox => self,
            BoxType::BlockBox => {
                match self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::AnonymousBox,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox {
                        box_type: BoxType::AnonymousBox,
                        box_props: None,
                        children: vec![],
                    }),
                }
                self.children.last_mut().unwrap()
            }
        }
    }

    pub fn inner_text(&self) -> String {
        self.children
            .iter()
            .clone()
            .into_iter()
            .map(|node| match node.box_props {
                Some(BoxProps {
                    node_type: NodeType::Text(t),
                    ..
                }) => t.data.clone(),
                _ => node.inner_text(),
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

pub fn to_layout_document<'a>(document: StyledDocument<'a>) -> LayoutDocument<'a> {
    let layout_box = to_layout_box(document.document_element);
    LayoutDocument {
        top_box: layout_box,
    }
}

fn to_layout_box<'a>(snode: StyledNode<'a>) -> LayoutBox<'a> {
    let box_type = match snode.display() {
        Display::Block => BoxType::BlockBox,
        Display::Inline => BoxType::InlineBox,
        Display::None => BoxType::NoneBox,
    };

    let box_props = BoxProps {
        node_type: snode.node_type,
        properties: snode.properties,
    };

    let mut layout = LayoutBox {
        box_type: box_type,
        box_props: Some(box_props),
        children: vec![],
    };

    for child in snode.children {
        match child.display() {
            Display::Block => {
                layout.children.push(to_layout_box(child));
            }
            Display::Inline => {
                layout
                    .inline_container()
                    .children
                    .push(to_layout_box(child));
            }
            Display::None => {}
        }
    }

    layout
}
