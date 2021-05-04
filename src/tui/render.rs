use crate::core::{
    dom::NodeType,
    layout::{BoxType, LayoutBox},
};
use cursive::{
    views::{LinearLayout, TextView},
    View,
};

mod a;
mod input;

pub type ElementContainer = LinearLayout;

impl<'a> From<LayoutBox<'a>> for ElementContainer {
    fn from(layout: LayoutBox<'a>) -> Self {
        // render the children
        let mut container = match layout.box_type {
            BoxType::NoneNode(_) => {
                return LinearLayout::horizontal();
            }
            BoxType::BlockNode(_) => LinearLayout::vertical(),
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => LinearLayout::horizontal(),
        };
        for child in layout.children {
            let e: ElementContainer = child.into();
            if e.len() != 0 {
                container.add_child(e);
            }
        }

        // render the node of layout box
        let element = match layout.box_type {
            BoxType::BlockNode(snode)
            | BoxType::InlineNode(snode) => match snode.node_type {
                NodeType::Element(ref element) => match element.tag_name.as_str() {
                    "a" => Some(a::render(snode, element)),
                    "input" => Some(input::render(snode, element)),
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
            },
            _ => None,
        };
        if let Some(element) = element {
            container.insert_child(0, element);
        }

        container
    }
}
