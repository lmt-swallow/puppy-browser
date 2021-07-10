//! This module includes some *glue* implementations for PageView APIs.

use super::JavaScriptRuntime;
use log::{error, info};
use rusty_v8 as v8;

/// `request_rerender` invokes PageView API and causes re-rendering.
pub fn request_rerender<'s>(scope: &mut v8::HandleScope<'s>, caller: &'static str) {
    let pv_api_handler = match JavaScriptRuntime::pv_api_handler(scope) {
        Some(_p) => _p,
        None => {
            error!("failed to get document reference; pv_api_handler is None");
            return;
        }
    };
    match pv_api_handler.request_rerender() {
        Ok(_) => {
            info!("re-render requested from {}", caller);
        }
        Err(e) => {
            error!("failed to request alert(); {}", e);
        }
    };
}
