use super::super::components::{Link, TextInputView};
use crate::{
    dom::{Node, NodeType},
    ui::{browser_view::with_current_browser_view, traits::Clearable, BrowserView},
};
use cursive::{
    traits::Boxable,
    view::ViewWrapper,
    views::{Button, LinearLayout, TextView},
};
use log::{error, info};

type ElementContainer = LinearLayout;

impl Clearable for ElementContainer {
    fn clear(&mut self) {
        for _ in 0..self.len() {
            self.remove_child(0);
        }
    }
}

impl Clearable for PageView {
    fn clear(&mut self) {
        self.view.clear()
    }
}

pub struct PageView {
    view: LinearLayout,
    document: Option<Node>,
}

impl PageView {
    pub fn new() -> Self {
        PageView {
            view: LinearLayout::vertical(),
            document: None,
        }
    }

    pub fn render_document(&mut self, node: Node) {
        match node.node_type {
            NodeType::Document(ref _document) => {
                assert_eq!(node.child_nodes.len(), 1);
                if let Some(top_elem) = node.child_nodes.get(0) {
                    render_node(&mut self.view, top_elem);
                }
            }
            _ => {}
        };
        self.document = Some(node);
    }
}

impl ViewWrapper for PageView {
    type V = LinearLayout;

    fn with_view<F, R>(&self, f: F) -> ::std::option::Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&self.view))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> ::std::option::Option<R>
    where
        F: ::std::ops::FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut self.view))
    }

    fn into_inner(self) -> ::std::result::Result<Self::V, Self>
    where
        Self::V: ::std::marker::Sized,
    {
        Ok(self.view)
    }
}

pub fn render_node(view: &mut ElementContainer, node: &Node) {
    match node.node_type {
        NodeType::Element(ref element) => match element.tag_name.as_str() {
            "p" => {
                let mut inline_view = LinearLayout::horizontal();
                for node in node.child_nodes.iter() {
                    render_node(&mut inline_view, node);
                }
                view.add_child(inline_view);
            }
            "a" => {
                let link_href: String = element
                    .attributes
                    .get("href")
                    .unwrap_or(&"".to_string())
                    .to_string();
                view.add_child(Link::new(node.inner_text(), move |s| {
                    if with_current_browser_view(s, |b: &mut BrowserView| {
                        b.resolve_url(link_href.clone())
                            .and_then(|url| b.navigate_to(url))
                    })
                    .is_none()
                    {
                        error!("failed to initiate navigation by link")
                    }
                }));
            }
            "input" => match element
                .attributes
                .get("type")
                .unwrap_or(&"".to_string())
                .as_str()
            {
                "text" => {
                    view.add_child(
                        TextInputView::new()
                            .content(element.attributes.get("value").unwrap_or(&"".to_string()))
                            .min_width(10)
                            .max_width(10),
                    );
                }
                "button" | "submit" => {
                    view.add_child(Button::new(
                        element.attributes.get("value").unwrap_or(&"".to_string()),
                        |_s| {},
                    ));
                }
                t => {
                    info!("unsupported input tag type {} found", t);
                }
            },
            "button" => {
                view.add_child(Button::new(node.inner_text(), |_s| {}));
            }
            "html" => {
                for node in node.child_nodes.iter() {
                    render_node(view, node);
                }
            }
            "div" => {
                let mut child_view = LinearLayout::horizontal();
                for node in node.child_nodes.iter() {
                    render_node(&mut child_view, node);
                }
                view.add_child(child_view);
            }
            _ => {
                for node in node.child_nodes.iter() {
                    render_node(view, node);
                }
            }
        },
        NodeType::Text(ref t) => {
            view.add_child(TextView::new(&t.data));
        }
        _ => {}
    }
}
