use super::style::Display;
use super::style::StyledNode;

#[derive(Debug)]
pub struct LayoutBox<'a> {
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}

#[derive(Debug)]
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    NoneNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

impl<'a> LayoutBox<'a> {
    pub fn inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::NoneNode(_) | BoxType::AnonymousBlock => self,
            BoxType::BlockNode(_) => {
                match self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::AnonymousBlock,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox {
                        box_type: BoxType::AnonymousBlock,
                        children: vec![],
                    }),
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}

impl<'a> From<&'a StyledNode<'a>> for LayoutBox<'a> {
    fn from(snode: &'a StyledNode) -> Self {
        let box_type = match snode.display() {
            Display::Block => BoxType::BlockNode(&snode),
            Display::Inline => BoxType::InlineNode(&snode),
            Display::None => BoxType::NoneNode(&snode),
        };
        let mut layout = LayoutBox {
            box_type: box_type,
            children: vec![],
        };

        for child in &snode.children {
            match child.display() {
                Display::Block => {
                    layout.children.push(child.into());
                }
                Display::Inline => {
                    layout.inline_container().children.push(child.into());
                }
                Display::None => {}
            }
        }

        layout
    }
}
