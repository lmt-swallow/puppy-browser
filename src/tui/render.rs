use crate::core::{
    dom::NodeType,
    layout::{BoxProps, BoxType, LayoutBox},
};
use cursive::{
    views::{LinearLayout, TextView},
    View,
};

mod a;
mod input;

pub type ElementContainer = LinearLayout;

pub fn to_element_container<'a>(layout: &LayoutBox<'a>) -> ElementContainer {
    // render the children
    let mut container = match layout.box_type {
        BoxType::NoneBox => {
            return LinearLayout::horizontal();
        }
        BoxType::BlockBox => LinearLayout::vertical(),
        BoxType::InlineBox | BoxType::AnonymousBox => LinearLayout::horizontal(),
    };
    for child in &layout.children {
        let e: ElementContainer = to_element_container(child);
        if e.len() != 0 {
            container.add_child(e);
        }
    }

    // render the node of layout box
    let element = match layout.box_props {
        Some(BoxProps {
            node_type: NodeType::Element(ref element),
            ..
        }) => match element.tag_name.as_str() {
            "a" => Some(a::render(layout, element)),
            "input" => Some(input::render(layout, element)),
            _ => None,
        },
        Some(BoxProps {
            node_type: NodeType::Text(ref t),
            ..
        }) => {
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
    };
    if let Some(element) = element {
        container.insert_child(0, element);
    }

    container
}
