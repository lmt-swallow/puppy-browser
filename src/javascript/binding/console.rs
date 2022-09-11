//! # Console API
//!
//! This module includes implementations of a subset of Console API (https://console.spec.whatwg.org/).

use rusty_v8 as v8;

use super::{create_object_under};

fn attach_printable_function_to_console(
    scope: &mut v8::HandleScope<'_>,
    console: v8::Local<v8::Object>,
    name: &'static str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let key = v8::String::new(scope, name).unwrap();
    let tmpl = v8::FunctionTemplate::new(scope, callback);
    let val = tmpl.get_function(scope).unwrap();
    console.set(scope, key.into(), val.into());
}

pub fn initialize_console<'s>(
    context_scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let console = create_object_under(context_scope, global, "console");
    let callback = |scope: &mut v8::HandleScope,
                    args: v8::FunctionCallbackArguments,
                    mut _retval: v8::ReturnValue| {
        println!("{}",
            args.get(0)
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope));
    };
    ["log", "info", "trace", "error"].map(|name| {
        attach_printable_function_to_console(context_scope, console, name, callback);
    });

    console
}
