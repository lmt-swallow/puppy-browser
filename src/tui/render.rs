use crate::{
    common::dom::{Node, NodeType},
    javascript::JavaScriptRuntimeError,
};
use cursive::views::{LinearLayout, TextView};
use log::error;
use thiserror::Error;

mod a;
mod button;
mod input;

#[derive(Error, Debug, PartialEq)]
pub enum RenderError {
    #[error("failed to render; no document exists")]
    NoDocumentError,

    #[error("failed to render; unsupported input type {specified_type:?} found")]
    UnsupportedInputTypeError { specified_type: String },

    #[error("failed to render; unsupported node type found")]
    UnsupportedNodeTypeError,

    #[error("failed to render; javascript execution failed: {0:?}")]
    JavaScriptError(JavaScriptRuntimeError),
}

pub trait Renderable {
    fn render_node(&mut self, node: &Node) -> Result<(), RenderError>;
    fn render_nodes(&mut self, node: &Vec<Node>) -> Result<(), RenderError>;
}

pub type ElementContainer = LinearLayout;
impl Renderable for ElementContainer {
    fn render_node(&mut self, node: &Node) -> Result<(), RenderError> {
        match &node.node_type {
            NodeType::Element(ref element) => match element.tag_name.as_str() {
                "script" => Ok(()),
                "a" => a::render(node, element).map(|v| self.add_child(v)),
                "input" => input::render(node, element).map(|v| self.add_child(v)),
                "button" => button::render(node, element).map(|v| self.add_child(v)),
                "html" => self.render_nodes(&node.child_nodes),
                "div" | "span" | "p" => {
                    let mut child_view = LinearLayout::horizontal();
                    match child_view.render_nodes(&node.child_nodes) {
                        Ok(_) => {
                            self.add_child(child_view);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                _ => self.render_nodes(&node.child_nodes),
            },
            NodeType::Text(ref t) => {
                self.add_child(TextView::new(&t.data));
                Ok(())
            }
            _ => Err(RenderError::UnsupportedNodeTypeError),
        }
    }

    fn render_nodes(&mut self, nodes: &Vec<Node>) -> Result<(), RenderError> {
        match nodes
            .iter()
            .map(|node| self.render_node(node))
            .collect::<Result<Vec<()>, RenderError>>()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
