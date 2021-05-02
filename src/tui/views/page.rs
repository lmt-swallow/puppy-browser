use std::{cell::RefCell, rc::Rc};

use crate::{
    dom::{Node, NodeType},
    javascript::{JavaScriptRuntime, JavaScriptRuntimeError},
    tui::{
        components::{Link, TextInputView},
        traits::Clearable,
        views::{with_current_browser_view, BrowserView},
    },
    window::Window,
};
use cursive::{
    traits::Boxable,
    view::ViewWrapper,
    views::{Button, LinearLayout, TextView},
};
use log::{error, info};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RenderError {
    #[error("failed to render")]
    UnknownError,

    #[error("failed to render; no document exists")]
    NoDocumentError,

    #[error("failed to render; unsupported input type {specified_type:?} found")]
    UnsupportedInputTypeError { specified_type: String },

    #[error("failed to render; unsupported node type found")]
    UnsupportedNodeTypeError,

    #[error("failed to render; javascript execution failed")]
    JavaScriptError(JavaScriptRuntimeError),
}

type ElementContainer = LinearLayout;

impl Clearable for ElementContainer {
    fn clear(&mut self) {
        for _ in 0..self.len() {
            self.remove_child(0);
        }
    }
}

fn render_nodes(
    view: &mut ElementContainer,
    nodes: &Vec<Node>,
    js_runtime: &mut JavaScriptRuntime,
) -> Result<(), RenderError> {
    match nodes
        .iter()
        .map(|node| render_node(view, node, js_runtime))
        .collect::<Result<Vec<()>, RenderError>>()
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn render_node(
    view: &mut ElementContainer,
    node: &Node,
    js_runtime: &mut JavaScriptRuntime,
) -> Result<(), RenderError> {
    match &node.node_type {
        NodeType::Element(ref element) => match element.tag_name.as_str() {
            "script" => match js_runtime.execute("(inline)", node.inner_text().as_str()) {
                Ok(s) => {
                    info!("javascript execution succeeded; {}", s);
                    Ok(())
                }
                Err(e) => Err(RenderError::JavaScriptError(e)),
            },
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
                Ok(())
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
                    Ok(())
                }
                "button" | "submit" => {
                    view.add_child(Button::new(
                        element.attributes.get("value").unwrap_or(&"".to_string()),
                        |_s| {},
                    ));
                    Ok(())
                }
                t => {
                    info!("unsupported input tag type {} found", t);
                    Err(RenderError::UnsupportedInputTypeError {
                        specified_type: t.to_string(),
                    })
                }
            },
            "button" => {
                view.add_child(Button::new(node.inner_text(), |_s| {}));
                Ok(())
            }
            "html" => render_nodes(view, &node.child_nodes, js_runtime),
            "div" | "span" | "p" => {
                let mut child_view = LinearLayout::horizontal();
                match render_nodes(&mut child_view, &node.child_nodes, js_runtime) {
                    Ok(_) => {
                        view.add_child(child_view);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => render_nodes(view, &node.child_nodes, js_runtime),
        },
        NodeType::Text(ref t) => {
            view.add_child(TextView::new(&t.data));
            Ok(())
        }
        _ => Err(RenderError::UnsupportedNodeTypeError),
    }
}

pub struct PageView {
    window: Option<Rc<RefCell<Window>>>,
    document: Option<Node>,

    view: ElementContainer,
    js_runtime: JavaScriptRuntime,
}

impl PageView {
    pub fn new() -> Self {
        Self {
            window: None,
            document: None,
            view: LinearLayout::vertical(),
            js_runtime: JavaScriptRuntime::new(),
        }
    }

    /// This function prepares a new page with given document.
    pub fn init_page(&mut self, document: Node) -> Result<(), RenderError> {
        // assert the argument is Document.
        match document.node_type {
            NodeType::Document(ref _document) => {}
            _ => return Err(RenderError::NoDocumentError),
        };

        // prepare `Window` object for the new page
        let window = Rc::new(RefCell::new(Window {
            name: "".to_string(),
        }));

        // set basic props of this page
        self.window = Some(window.clone());
        self.document = Some(document);

        // set reference to Window object of this page for JavaScript runtime
        self.js_runtime.set_window(window.clone());

        self.render_document()
    }

    /// This function renders `self.document` to `self.view`.
    ///
    /// TODO (enhancement): layout boxes and construct "layout tree" before rendering
    fn render_document(&mut self) -> Result<(), RenderError> {
        // assert self.document is set
        let document = match &self.document {
            Some(w) => w,
            None => return Err(RenderError::NoDocumentError),
        };

        // render DOM recursively
        match document.node_type {
            NodeType::Document(ref _document) => {
                assert_eq!(document.child_nodes.len(), 1);
                if let Some(top_element) = document.child_nodes.get(0) {
                    render_node(&mut self.view, top_element, &mut self.js_runtime)
                } else {
                    Ok(())
                }
            }
            _ => Err(RenderError::UnknownError),
        }
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

impl Clearable for PageView {
    fn clear(&mut self) {
        self.view.clear()
    }
}
