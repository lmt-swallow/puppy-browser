use crate::dom::{Node, NodeType};
use cursive::views::{LinearLayout, TextView};

pub type ElementContainer = LinearLayout;

pub fn render_node(view: &mut ElementContainer, node: &Node) {
    match node.node_type {
        NodeType::Element(ref element) => match element.tag_name.as_str() {
            "p" => {
                let text = node
                    .child_nodes
                    .get(0)
                    .and_then(|n| {
                        if let NodeType::Text(t) = &n.node_type {
                            Some(t)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                view.add_child(TextView::new(&text.data));
            }
            "html" => {                
                for node in node.child_nodes.iter() {
                    render_node(view, node);
                }
            }
            _ => (),
        },
        _ => {}
    }
}

pub fn render_node_from_document(view: &mut ElementContainer, node: &Node) {
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
