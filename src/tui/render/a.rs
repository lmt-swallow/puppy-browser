//! This module provides a feature to render `a` tags.

use cursive::View;

use crate::{
    core::{dom::element::Element, layout::LayoutBox},
    tui::{components::Link, views::with_current_browser_view, BrowserView},
};

pub fn render(lbox: &LayoutBox, element: &Element) -> Box<dyn View> {
    let link_href: String = element
        .attributes
        .get("href")
        .unwrap_or(&"".to_string())
        .to_string();
    Box::new(Link::new(lbox.inner_text(), move |s| {
        with_current_browser_view(s, |b: &mut BrowserView| {
            b.resolve_url(link_href.clone())
                .map(|url| b.navigate_to(url))
        });
    }))
}
