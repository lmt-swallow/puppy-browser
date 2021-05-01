mod console;
use rusty_v8 as v8;

/// `initialize_context` takes a HandleScope to `v8::Isolate` object and returns a new HandleScope to newly created `v8::Context`.
pub fn initialize_context<'s>(
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

    // return with a handle to newly created v8::Context
    scope.escape(context)
}

pub fn create_object_under<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
) -> v8::Local<'s, v8::Object> {
    let key = v8::String::new(scope, name).unwrap();
    let value = v8::Object::new(scope);
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
