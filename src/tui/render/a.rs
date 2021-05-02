use cursive::View;

use crate::{
    common::dom::{element::Element, Node},
    tui::{components::Link, views::with_current_browser_view, BrowserView},
};

use super::RenderError;

pub fn render(node: &Node, element: &Element) -> Result<impl View, RenderError> {
    let link_href: String = element
        .attributes
        .get("href")
        .unwrap_or(&"".to_string())
        .to_string();
    Ok(Link::new(node.inner_text(), move |s| {
        with_current_browser_view(s, |b: &mut BrowserView| {
            b.resolve_url(link_href.clone())
                .map(|url| b.navigate_to(url))
        });
    }))
}
