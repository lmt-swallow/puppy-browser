mod console;
mod dom;
mod window;
use log::{error, info};
use rusty_v8 as v8;
use v8::READ_ONLY;

use super::JavaScriptRuntime;

/// `initialize_context` takes a HandleScope to `v8::Isolate` object and returns a new HandleScope to newly created `v8::Context`.
pub fn create_context_with<'s>(
    isolate_scope: &mut v8::HandleScope<'s, ()>,
) -> v8::Local<'s, v8::Context> {
    let scope = &mut v8::EscapableHandleScope::new(isolate_scope);

    // create context
    let context = v8::Context::new(scope);

    // get global proxy object
    let global = context.global(scope);

    // bind `console` object
    let scope = &mut v8::ContextScope::new(scope, context);
    console::initialize_console(scope, global);

    // bind `window` object
    let scope = &mut v8::ContextScope::new(scope, context);
    window::initialize_window(scope, global);

    // bind `document` object
    let scope = &mut v8::ContextScope::new(scope, context);
    dom::initialize_dom(scope, global);

    // return with a handle to newly created v8::Context
    scope.escape(context)
}

pub fn create_object_under<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
) -> v8::Local<'s, v8::Object> {
    let template = v8::ObjectTemplate::new(scope);
    let key = v8::String::new(scope, name).unwrap();
    let value = template.new_instance(scope).unwrap();
    target.set(scope, key.into(), value.into());
    value
}

pub fn set_function_to(
    scope: &mut v8::HandleScope<'_>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let key = v8::String::new(scope, name).unwrap();
    let tmpl = v8::FunctionTemplate::new(scope, callback);
    let val = tmpl.get_function(scope).unwrap();
    target.set(scope, key.into(), val.into());
}

pub fn set_property_with_accessor<'s, GetterF, SetterF>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    getter: GetterF,
    setter: SetterF,
) where
    GetterF: Sized
        + Copy
        + Fn(
            &mut v8::HandleScope,
            v8::Local<v8::Name>,
            v8::PropertyCallbackArguments,
            v8::ReturnValue,
        ),
    SetterF: Sized
        + Copy
        + Fn(
            &mut v8::HandleScope,
            v8::Local<v8::Name>,
            v8::Local<v8::Value>,
            v8::PropertyCallbackArguments,
        ),
{
    let key = v8::String::new(scope, name).unwrap();
    target.set_accessor_with_setter(scope, key.into(), getter, setter);
}

pub fn set_property<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.set(scope, key.into(), value.into());
}

pub fn set_readonly_constant<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &str,
    cvalue: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.define_own_property(scope, key.into(), cvalue, READ_ONLY);
}

fn request_rerender<'s>(scope: &mut v8::HandleScope<'s>, caller: &'static str) {
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
