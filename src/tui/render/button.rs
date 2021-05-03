use cursive::{views::Button, View};

use crate::common::dom::{element::Element, Node};

pub fn render(node: &Node, _element: &Element) -> Box<dyn View> {
    Box::new(Button::new(node.inner_text(), |_s| {}))
}
