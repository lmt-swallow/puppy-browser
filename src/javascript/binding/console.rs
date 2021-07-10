//! # Console API
//!
//! This module includes implementations of a subset of Console API (https://console.spec.whatwg.org/).

use log::info;
use rusty_v8 as v8;

use super::{create_object_under, set_function_to};

pub fn initialize_console<'s>(
    context_scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let console = create_object_under(context_scope, global, "console");
    set_function_to(
        context_scope,
        console,
        "info",
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut _retval: v8::ReturnValue| {
            info!(
                "{}",
                args.get(0)
                    .to_string(scope)
                    .unwrap()
                    .to_rust_string_lossy(scope)
            );
        },
    );
    console
}
