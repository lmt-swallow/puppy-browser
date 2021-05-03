use cursive::{traits::Finder, view::ViewWrapper, views::LinearLayout, CbSink, Cursive, With};
use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    common::{layout::LayoutBox, StyledNode},
    dom::{Node, NodeType},
    javascript::{JavaScriptRuntime, JavaScriptRuntimeError},
    tui::{components::alert, render::ElementContainer},
    window::Window,
};
use log::{error, info};
use thiserror::Error;

use super::PAGE_VIEW_NAME;
#[derive(Error, Debug, PartialEq)]
pub enum PageError {
    #[error("failed to render; no document exists")]
    NoDocumentError,

    #[error("failed to render; javascript execution failed: {0:?}")]
    JavaScriptError(JavaScriptRuntimeError),
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
                    match v.render_document() {
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
    pub js_runtime: JavaScriptRuntime,
}

impl PageView {
    pub fn new(ui_cb_sink: Rc<CbSink>) -> Self {
        (Self {
            window: None,
            document: None,

            view: ElementContainer::vertical(),

            js_runtime: JavaScriptRuntime::new(),
        })
        .with(|v| {
            v.js_runtime
                .set_pv_api_handler(PageViewAPIHandler::new(ui_cb_sink));
        })
    }

    /// This function prepares a new page with given document.
    pub fn init_page(&mut self, document: Node) -> Result<(), PageError> {
        // assert the argument is Document.
        match document.node_type {
            NodeType::Document(ref _document) => {}
            _ => return Err(PageError::NoDocumentError),
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
        self.render_document()?;

        // run JavaScript
        self.execute_inline_scripts()?;

        Ok(())
    }

    /// This function renders `self.document` to `self.view`.
    fn render_document(&mut self) -> Result<(), PageError> {
        // assert self.document is set
        let document = match &self.document {
            Some(w) => w,
            None => return Err(PageError::NoDocumentError),
        };
        let document = document.borrow_mut();

        // render document
        let top_element = document.document_element();
        let styled: &StyledNode = &top_element.into();
        let layout: LayoutBox = styled.into();
        self.view = layout.into();

        Ok(())
    }

    /// This function run scripts.
    ///
    /// TODO (enhancement): note on "re-entrant" of HTML tree construction
    fn execute_inline_scripts(&mut self) -> Result<(), PageError> {
        let scripts = {
            // extract inline scripts first to release borrowing of self.document.
            // This should be done before JS access DOM API.
            let document = match &self.document {
                Some(w) => w,
                None => return Err(PageError::NoDocumentError),
            };
            let document = document.borrow_mut();
            document.get_inline_scripts_recursively()
        };

        for script in scripts {
            match self.js_runtime.execute("(inline)", script.as_str()) {
                Ok(s) => {
                    info!("javascript execution succeeded; {}", s);
                }
                Err(e) => return Err(PageError::JavaScriptError(e)),
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
