use crate::dom::{Node, NodeType};
use cursive::views::{LinearLayout, TextView};
use log::debug;

pub type ElementContainer = LinearLayout;

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
            "html" => {
                for node in node.child_nodes.iter() {
                    render_node(view, node);
                }
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
