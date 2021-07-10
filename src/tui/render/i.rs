//! This module provides a feature to render `i` tags.

use cursive::{theme::Effect, utils::markup::StyledString, views::TextView, View};

use crate::core::{dom::element::Element, layout::LayoutBox};

pub fn render(lbox: &LayoutBox, _element: &Element) -> Box<dyn View> {
    Box::new(TextView::new(StyledString::styled(
        lbox.inner_text(),
        Effect::Italic,
    )))
}
