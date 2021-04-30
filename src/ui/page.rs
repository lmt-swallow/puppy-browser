use super::traits::Clearable;
use crate::dom::{Node, NodeType};
use cursive::{
    traits::Boxable,
    views::{Button, EditView, LinearLayout, TextView},
};
use log::{debug, info};

use super::{components::Link, resolve_and_navigate};

pub type ElementContainer = LinearLayout;

impl Clearable for ElementContainer {
    fn clear(&mut self) {
        for _ in 0..self.len() {
            self.remove_child(0);
        }
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
                    resolve_and_navigate(s, link_href.clone())
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
                        EditView::new()
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

pub fn render_node_from_document(view: &mut ElementContainer, node: &Node) {
    debug!("{:?}", node);
    match node.node_type {
        NodeType::Document(ref _document) => {
            assert_eq!(node.child_nodes.len(), 1);
            if let Some(top_elem) = node.child_nodes.get(0) {
                render_node(view, top_elem);
            }
        }
        _ => {}
    }
}
