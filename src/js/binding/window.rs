use log::trace;
use rusty_v8 as v8;

use crate::js::JavaScriptRuntime;

use super::{create_object_under, set_property};

pub fn initialize_window<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let window = create_object_under(scope, global, "window");

    set_property(
        scope,
        window,
        "name",
        |scope: &mut v8::HandleScope,
         key: v8::Local<v8::Name>,
         _args: v8::PropertyCallbackArguments,
         mut rv: v8::ReturnValue| {
            trace!("Read access to: {}", key.to_rust_string_lossy(scope));

            let state = JavaScriptRuntime::state(scope);
            let state = state.borrow_mut();

            let window = state.window.clone();
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

            let state = JavaScriptRuntime::state(scope);
            let state = state.borrow_mut();

            let window = state.window.clone();
            let window = window.unwrap();
            let mut window = window.borrow_mut();

            let value = value.to_rust_string_lossy(scope);
            window.name = value;
        },
    );

    window
}
