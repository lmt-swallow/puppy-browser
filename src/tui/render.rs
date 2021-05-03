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
            BoxType::NoneNode(_) => {
                return LinearLayout::horizontal();
            }
            BoxType::BlockNode(_) => LinearLayout::vertical(),
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => LinearLayout::horizontal(),
        };
        for child in self.children {
            let e: ElementContainer = child.into();
            if e.len() != 0 {
                container.add_child(e);
            }
        }

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
                NodeType::Text(ref t) => {
                    // NOTE: This is puppy original behaviour, not a standard one.
                    let text_to_display = t.data.clone();
                    let text_to_display = text_to_display.replace("\n", "");
                    if text_to_display.trim() != "" {
                        Some(Box::new(TextView::new(text_to_display.trim())) as Box<dyn View>)
                    } else {
                        None
                    }
                }
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
