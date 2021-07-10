//! This module includes some implementations on a *page*, which renders a webpage.

use cursive::{traits::Finder, view::ViewWrapper, views::LinearLayout, CbSink, Cursive, With};
use std::{cell::RefCell, rc::Rc};

use crate::{
    core::{
        dom::Document,
        layout::{to_layout_document, LayoutDocument},
        style::{to_styled_document, StyledDocument},
    },
    javascript::{JavaScriptRuntime, JavaScriptRuntimeError},
    tui::{
        render::{to_element_container, ElementContainer},
        PageViewAPIHandler,
    },
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

pub struct PageView {
    // on document shown in the page
    window: Option<Rc<RefCell<Window>>>,
    document: Option<Rc<RefCell<Document>>>,

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
                .set_pv_api_handler(Rc::new(PageViewAPIHandler::new(ui_cb_sink)));
        })
    }

    /// `init_page` shows the given document to the PageView. 
    pub fn init_page(&mut self, document: Document) -> Result<(), PageError> {
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

    /// `render_document` renders `self.document` to `self.view`.
    pub fn render_document(&mut self) -> Result<(), PageError> {
        // assert self.document is set
        let document = match &self.document {
            Some(w) => w,
            None => return Err(PageError::NoDocumentError),
        };
        let document = &*document.borrow_mut();

        // render document
        let styled: StyledDocument = to_styled_document(document);
        let layout: LayoutDocument = to_layout_document(styled);
        self.view = to_element_container(&layout.top_box);

        Ok(())
    }

    /// `execute_inline_scripts` runs all the inline scripts.
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
            document.get_script_inners()
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

/// `with_current_page_view` returns the PageView instance shown in the given cursive screen instance.
pub fn with_current_page_view<Output, F>(s: &mut Cursive, f: F) -> Option<Output>
where
    F: FnOnce(&mut PageView) -> Output,
{
    s.screen_mut().call_on_name(PAGE_VIEW_NAME, f)
}
