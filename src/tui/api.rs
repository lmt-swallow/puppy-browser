//! This module defines *the PageView API*, which provides Rust-side operations to a JavaScript runtime.

use std::{error::Error, rc::Rc};

use cursive::CbSink;
use log::{error, info};

use super::{components::alert, views::with_current_page_view};

/// `PageViewAPIHandler` is an interface which a JavaScript runtime can use.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_handler() {
        let (cb_sink, cb_recv) = crossbeam_channel::unbounded();
        let cb_sink = Rc::new(cb_sink);
        let api = Rc::new(PageViewAPIHandler::new(cb_sink));

        {
            assert!(api.alert("hello".to_string()).is_ok());
            assert!(cb_recv.try_recv().is_ok());
            assert!(cb_recv.try_recv().is_err());
        }

        {
            assert!(api.request_rerender().is_ok());
            assert!(cb_recv.try_recv().is_ok());
            assert!(cb_recv.try_recv().is_err());
        }
    }
}
