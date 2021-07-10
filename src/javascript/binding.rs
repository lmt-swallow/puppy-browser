//! This module consists of utility functions for V8 integration.

mod console;
mod dom;
mod window;
use rusty_v8 as v8;
use v8::READ_ONLY;

/// `create_context_with` takes a HandleScope to `v8::Isolate` object
/// and returns a new HandleScope to newly created `v8::Context`.
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

/// `create_object_under` creates an `Object` object with the given name.
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

/// `set_function_to` adds a `Function` object which calls the given Rust function,
/// into the given object,
/// with the given property name.
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

/// `set_accessor_to` adds a property with the given name and getter/setter, into the given object.
pub fn set_accessor_to<'s, GetterF, SetterF>(
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

/// `set_property_to` adds a property with the given name and value, into the given object.
pub fn set_property_to<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.set(scope, key.into(), value.into());
}

/// `set_property_to` adds a read-only property with the given name and value, into the given object.
pub fn set_constant_to<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &str,
    cvalue: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.define_own_property(scope, key.into(), cvalue, READ_ONLY);
}
