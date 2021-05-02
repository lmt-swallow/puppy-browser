use log::{error, info, trace};
use rusty_v8 as v8;

use crate::{common::dom::Text, javascript::JavaScriptRuntime};

use super::{create_object_under, set_function_to};

pub fn initialize_document<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let document = create_object_under(scope, global, "document");

    // `appendChild` property
    set_function_to(
        scope,
        document,
        "appendChild",
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut _retval: v8::ReturnValue| {
            let message = args
                .get(0)
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);
            trace!("appendChild called with: {}", message);

            let document = match JavaScriptRuntime::document(scope) {
                Some(_document) => _document,
                None => {
                    error!("failed to get document reference; document is None");
                    return;
                }
            };
            let mut document = document.borrow_mut();

            let node = Text::new(message);
            let top_element = document.document_element_mut();
            top_element.append_child(node);

            let pv_api_handler = match JavaScriptRuntime::pv_api_handler(scope) {
                Some(_p) => _p,
                None => {
                    error!("failed to get document reference; pv_api_handler is None");
                    return;
                }
            };

            match pv_api_handler.request_rerender() {
                Ok(_) => {
                    info!("re-render requested due to appendChild");
                }
                Err(e) => {
                    error!("failed to request alert(); {}", e);
                }
            };
        },
    );

    document
}
