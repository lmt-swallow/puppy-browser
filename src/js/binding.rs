use rusty_v8 as v8;

pub fn initialize_context<'s>(scope: &mut v8::HandleScope<'s, ()>) -> v8::Local<'s, v8::Context> {
    let scope = &mut v8::EscapableHandleScope::new(scope);
    let context = v8::Context::new(scope);

    // TODO: initialization
    // let template = v8::ObjectTemplate::new(isolate_handle_scope);
    // template.set(
    //     v8::String::new(isolate_handle_scope, "log").unwrap().into(),
    //     v8::FunctionTemplate::new(
    //         isolate_handle_scope,
    //         |scope: &mut v8::HandleScope,
    //          args: v8::FunctionCallbackArguments,
    //          mut _retval: v8::ReturnValue| {
    //             let message = args
    //                 .get(0)
    //                 .to_string(scope)
    //                 .unwrap()
    //                 .to_rust_string_lossy(scope);

    //             println!("Logged: {}", message);
    //         },
    //     )
    //     .into(),
    // );

    scope.escape(context)
}
