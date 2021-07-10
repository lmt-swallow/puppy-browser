//! # Window interface
//!
//! This module includes implementations of a subset of Window interface (https://html.spec.whatwg.org/multipage/window-object.html#window).

use log::{error, trace};
use rusty_v8 as v8;

use crate::javascript::JavaScriptRuntime;

use super::{create_object_under, set_accessor_to, set_function_to};

pub fn initialize_window<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let window = create_object_under(scope, global, "window");

    // `alert` property
    set_function_to(
        scope,
        window,
        "alert",
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut _retval: v8::ReturnValue| {
            let message = args
                .get(0)
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);
            trace!("alert called with: {}", message);

            let pv_api_handler = JavaScriptRuntime::pv_api_handler(scope).unwrap();
            match pv_api_handler.alert(message) {
                Ok(_) => {}
                Err(e) => {
                    error!("failed to request alert(); {}", e);
                }
            };
        },
    );

    // `name` property
    set_accessor_to(
        scope,
        window,
        "name",
        |scope: &mut v8::HandleScope,
         key: v8::Local<v8::Name>,
         _args: v8::PropertyCallbackArguments,
         mut rv: v8::ReturnValue| {
            trace!("Read access to: {}", key.to_rust_string_lossy(scope));

            let window = JavaScriptRuntime::window(scope);
            let window = window.unwrap();
            let window = window.borrow_mut();

            let value = window.name.as_str();
            rv.set(v8::String::new(scope, value).unwrap().into());
        },
        |scope: &mut v8::HandleScope,
         key: v8::Local<v8::Name>,
         value: v8::Local<v8::Value>,
         _args: v8::PropertyCallbackArguments| {
            trace!("Write access to: {}", key.to_rust_string_lossy(scope));

            let window = JavaScriptRuntime::window(scope);
            let window = window.unwrap();
            let mut window = window.borrow_mut();

            let value = value.to_rust_string_lossy(scope);
            window.name = value;
        },
    );

    window
}
