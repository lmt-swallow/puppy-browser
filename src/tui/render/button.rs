use cursive::{views::Button, View};

use crate::common::dom::{element::Element, Node};

use super::RenderError;

pub fn render(node: &Node, _element: &Element) -> Result<impl View, RenderError> {
    Ok(Button::new(node.inner_text(), |_s| {}))
}
