use crate::common::{
    dom::NodeType,
    layout::{BoxType, LayoutBox},
    StyledNode,
};
use cursive::{
    views::{LinearLayout, TextView},
    View,
};

mod a;
mod button;
mod input;

pub type ElementContainer = LinearLayout;

impl<'a> Into<ElementContainer> for LayoutBox<'a> {
    fn into(self) -> ElementContainer {
        // render the children
        let mut container = match self.box_type {
            BoxType::BlockNode(_) => {
                let mut container = LinearLayout::vertical();
                for child in self.children {
                    container.add_child::<ElementContainer>(child.into());
                }
                container
            }
            BoxType::InlineNode(_) => {
                let mut container = LinearLayout::horizontal();
                for child in self.children {
                    container.add_child::<ElementContainer>(child.into());
                }
                container
            }
            BoxType::AnonymousBlock => {
                let mut container = LinearLayout::horizontal();
                for child in self.children {
                    container.add_child::<ElementContainer>(child.into());
                }
                container
            }
            BoxType::NoneNode(_) => LinearLayout::horizontal(),
        };

        // render the node of layout box
        let element = match self.box_type {
            BoxType::BlockNode(&StyledNode { node, .. })
            | BoxType::InlineNode(&StyledNode { node, .. }) => match node.node_type {
                NodeType::Element(ref element) => match element.tag_name.as_str() {
                    "a" => Some(a::render(node, element)),
                    "input" => Some(input::render(node, element)),
                    "button" => Some(button::render(node, element)),
                    _ => None,
                },
                NodeType::Text(ref t) => Some(Box::new(TextView::new(&t.data)) as Box<dyn View>),
                _ => None,
            },
            _ => None,
        };
        if let Some(element) = element {
            container.insert_child(0, element);
        }

        container
    }
}
