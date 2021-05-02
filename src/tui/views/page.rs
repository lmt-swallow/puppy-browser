use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    dom::{Node, NodeType},
    javascript::{JavaScriptRuntime, JavaScriptRuntimeError},
    tui::{
        components::{alert, Link, TextInputView},
        views::{with_current_browser_view, BrowserView},
    },
    window::Window,
};
use cursive::{
    traits::Boxable,
    traits::Finder,
    view::ViewWrapper,
    views::{Button, LinearLayout, TextView},
    CbSink, Cursive, With,
};
use log::{error, info};
use thiserror::Error;

use super::PAGE_VIEW_NAME;

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

type ElementContainer = LinearLayout;

// TODO: move those implemntations to somewhere else

fn render_nodes(view: &mut ElementContainer, nodes: &Vec<Node>) -> Result<(), RenderError> {
    match nodes
        .iter()
        .map(|node| render_node(view, node))
        .collect::<Result<Vec<()>, RenderError>>()
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn render_node(view: &mut ElementContainer, node: &Node) -> Result<(), RenderError> {
    match &node.node_type {
        NodeType::Element(ref element) => match element.tag_name.as_str() {
            "script" => Ok(()),
            "a" => {
                let link_href: String = element
                    .attributes
                    .get("href")
                    .unwrap_or(&"".to_string())
                    .to_string();
                view.add_child(Link::new(node.inner_text(), move |s| {
                    with_current_browser_view(s, |b: &mut BrowserView| {
                        b.resolve_url(link_href.clone())
                            .map(|url| b.navigate_to(url))
                    });
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
                    let onclick = element
                        .attributes
                        .get("onclick")
                        .unwrap_or(&"".to_string())
                        .clone();

                    view.add_child(Button::new(
                        element.attributes.get("value").unwrap_or(&"".to_string()),
                        move |s| {
                            let result = with_current_browser_view(s, |b: &mut BrowserView| {
                                b.with_page_view_mut(|p| {
                                    p.js_runtime.execute("(inline)", onclick.as_str())
                                })
                            });
                            if result.is_none() {
                                error!("failed to run onclick event of button")
                            }
                            match result.unwrap().unwrap() {
                                Ok(message) => {
                                    info!("succeeded to run javascript; {}", message);
                                }
                                Err(e) => {
                                    error!(
                                        "failed to run javascript; {}",
                                        RenderError::JavaScriptError(e)
                                    );
                                }
                            }
                        },
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
            "html" => render_nodes(view, &node.child_nodes),
            "div" | "span" | "p" => {
                let mut child_view = LinearLayout::horizontal();
                match render_nodes(&mut child_view, &node.child_nodes) {
                    Ok(_) => {
                        view.add_child(child_view);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => render_nodes(view, &node.child_nodes),
        },
        NodeType::Text(ref t) => {
            view.add_child(TextView::new(&t.data));
            Ok(())
        }
        _ => Err(RenderError::UnsupportedNodeTypeError),
    }
}

fn extract_script(node: &Node) -> Vec<String> {
    match &node.node_type {
        NodeType::Element(ref element) => match element.tag_name.as_str() {
            "script" => vec![node.inner_text()],
            _ => node
                .child_nodes
                .iter()
                .map(|node| extract_script(node))
                .collect::<Vec<Vec<String>>>()
                .into_iter()
                .flatten()
                .collect(),
        },
        _ => {
            vec![]
        }
    }
}

pub struct PageViewAPIHandler {
    ui_cb_sink: Rc<CbSink>,
}

impl PageViewAPIHandler {
    pub fn new(ui_cb_sink: Rc<CbSink>) -> Self {
        Self {
            ui_cb_sink: ui_cb_sink,
        }
    }

    pub fn alert(&self, message: String) -> Result<(), Box<dyn Error>> {
        self.ui_cb_sink
            .send(Box::new(move |s: &mut cursive::Cursive| {
                alert(s, "from JavaScript".to_string(), message);
            }))?;

        // TODO (enhancement): do this synchronoulsly & error handling
        Ok(())
    }

    pub fn request_rerender(&self) -> Result<(), Box<dyn Error>> {
        self.ui_cb_sink
            .send(Box::new(move |s: &mut cursive::Cursive| {
                with_current_page_view(s, |v| {
                    info!("re-rendering started");
                    match v.layout_document() {
                        Ok(_) => info!("re-rendering finished"),
                        Err(e) => error!("re-rendering failed; {}", e),
                    }
                });
            }))?;
        Ok(())
    }
}

pub struct PageView {
    // on document shown in the page
    window: Option<Rc<RefCell<Window>>>,
    document: Option<Rc<RefCell<Node>>>,

    // on UI
    view: ElementContainer,

    // on rendering
    js_runtime: JavaScriptRuntime,
}

impl PageView {
    pub fn new(ui_cb_sink: Rc<CbSink>) -> Self {
        (Self {
            window: None,
            document: None,

            view: LinearLayout::vertical(),

            js_runtime: JavaScriptRuntime::new(),
        })
        .with(|v| {
            v.js_runtime
                .set_pv_api_handler(PageViewAPIHandler::new(ui_cb_sink));
        })
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

        let document = Rc::new(RefCell::new(document));

        // set basic props of this page
        self.window = Some(window.clone());
        self.document = Some(document.clone());

        // set reference to Window object of this page for JavaScript runtime
        self.js_runtime.set_window(window.clone());
        self.js_runtime.set_document(document.clone());

        // layout document to self.view
        self.layout_document()?;

        // run JavaScript
        self.execute_inline_scripts()?;

        Ok(())
    }

    /// This function resets `self.view` perfectly.
    fn reset_view(&mut self) {
        self.view = LinearLayout::vertical();
    }

    /// This function renders `self.document` to `self.view`.
    ///
    /// TODO (enhancement): layout boxes and construct "layout tree" before rendering
    fn layout_document(&mut self) -> Result<(), RenderError> {
        self.reset_view();

        // assert self.document is set
        let document = match &self.document {
            Some(w) => w,
            None => return Err(RenderError::NoDocumentError),
        };
        let document = document.borrow_mut();

        // render DOM recursively
        let top_element = document.document_element();
        render_node(&mut self.view, top_element)
    }

    /// This function run scripts.
    ///
    /// TODO (enhancement): note on "re-entrant" of HTML tree construction
    fn execute_inline_scripts(&mut self) -> Result<(), RenderError> {
        let scripts = {
            // extract inline scripts first to release borrowing of self.document.
            // This should be done before JS access DOM API.
            let document = match &self.document {
                Some(w) => w,
                None => return Err(RenderError::NoDocumentError),
            };
            let document = document.borrow_mut();

            // traverse DOM recursively
            let top_element = document.document_element();
            extract_script(top_element)
        };

        for script in scripts {
            match self.js_runtime.execute("(inline)", script.as_str()) {
                Ok(s) => {
                    info!("javascript execution succeeded; {}", s);
                }
                Err(e) => return Err(RenderError::JavaScriptError(e)),
            };
        }
        Ok(())
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

pub fn with_current_page_view<Output, F>(s: &mut Cursive, f: F) -> Option<Output>
where
    F: FnOnce(&mut PageView) -> Output,
{
    s.screen_mut().call_on_name(PAGE_VIEW_NAME, f)
}
