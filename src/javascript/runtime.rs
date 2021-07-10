//! This module defines a JavaScript runtime with V8.

use crate::{core::dom::Document, javascript::binding, tui::PageViewAPIHandler, window::Window};
use rusty_v8 as v8;
use std::{cell::RefCell, rc::Rc, sync::Once};
use thiserror::Error;

/// `JavaScriptRuntimeState` defines a state of JS runtime that will be stored per v8 isolate.
///
/// TODO (enhancement): In Deno, which uses `rusty_v8` as we do, they use a more flexible datastore called [GothamState](https://github.com/denoland/deno/blob/v1.10.2/core/gotham_state.rs).
/// We can adopt the similar mechanisms instead of adding new properties when we need to extend this state more.
pub struct JavaScriptRuntimeState {
    pub context: v8::Global<v8::Context>,
    pub window: Option<Rc<RefCell<Window>>>,
    pub document: Option<Rc<RefCell<Document>>>,
    pub pv_api_handler: Option<Rc<PageViewAPIHandler>>,
}

/// `JavaScriptRuntime` defines a JS runtime with v8.
/// It has a link to a V8 isolate, and the isolate includes `JavaScriptRuntimeState` in its *slot*.
#[derive(Debug)]
pub struct JavaScriptRuntime {
    v8_isolate: v8::OwnedIsolate,
}

/// `JavaScriptRuntimeError` describes an error occured during JS execution.
#[derive(Error, Debug, PartialEq)]
pub enum JavaScriptRuntimeError {
    #[error("failed to parse the script: {message:?}")]
    CompileError { message: String },

    #[error("failed to execute the script: {message:?}")]
    ExecuteError { message: String },
}

#[allow(dead_code)]
impl JavaScriptRuntime {
    pub fn new() -> Self {
        // init v8 platform just once
        static PUPPY_INIT: Once = Once::new();
        PUPPY_INIT.call_once(move || {
            let platform = v8::new_default_platform().unwrap();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        // create v8 isolate & context
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = binding::create_context_with(scope);
            v8::Global::new(scope, context)
        };

        // store state inside v8 isolate
        // NOTE: the data would be stored by Isolate::SetData (https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36)
        isolate.set_slot(Rc::new(RefCell::new(JavaScriptRuntimeState {
            context: context,
            window: None,
            document: None,
            pv_api_handler: None,
        })));

        JavaScriptRuntime {
            v8_isolate: isolate,
        }
    }

    /// `execute` runs a given source in the current context.
    pub fn execute(
        &mut self,
        filename: &str,
        source: &str,
    ) -> Result<String, JavaScriptRuntimeError> {
        let scope = &mut self.get_handle_scope();

        let source = v8::String::new(scope, source).unwrap();
        let source_map = v8::undefined(scope);
        let name = v8::String::new(scope, filename).unwrap();
        let origin = v8::ScriptOrigin::new(
            scope,
            name.into(),
            0,
            0,
            false,
            0,
            source_map.into(),
            false,
            false,
            false,
        );

        let mut tc_scope = v8::TryCatch::new(scope);
        let script = match v8::Script::compile(&mut tc_scope, source, Some(&origin)) {
            Some(script) => script,
            None => {
                assert!(tc_scope.has_caught());
                return Err(JavaScriptRuntimeError::CompileError {
                    message: to_pretty_string(tc_scope),
                });
            }
        };

        match script.run(&mut tc_scope) {
            Some(result) => Ok(result
                .to_string(&mut tc_scope)
                .unwrap()
                .to_rust_string_lossy(&mut tc_scope)),
            None => {
                assert!(tc_scope.has_caught());
                Err(JavaScriptRuntimeError::ExecuteError {
                    message: to_pretty_string(tc_scope),
                })
            }
        }
    }
}

/// Implementations for state management
#[allow(dead_code)]
impl JavaScriptRuntime {
    /// `state` returns the runtime state stored in the given isolate.
    pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<JavaScriptRuntimeState>> {
        let s = isolate
            .get_slot::<Rc<RefCell<JavaScriptRuntimeState>>>()
            .unwrap();
        s.clone()
    }

    /// `get_state` returns the runtime state for the runtime.
    pub fn get_state(&self) -> Rc<RefCell<JavaScriptRuntimeState>> {
        Self::state(&self.v8_isolate)
    }

    /// `get_handle_scope` returns [a handle scope](https://v8docs.nodesource.com/node-0.8/d3/d95/classv8_1_1_handle_scope.html) for the runtime.
    pub fn get_handle_scope(&mut self) -> v8::HandleScope {
        let context = self.get_context();
        v8::HandleScope::with_context(&mut self.v8_isolate, context)
    }

    /// `get_context` returns [a handle scope](https://v8docs.nodesource.com/node-0.8/df/d69/classv8_1_1_context.html) for the runtime.
    pub fn get_context(&mut self) -> v8::Global<v8::Context> {
        let state = self.get_state();
        let state = state.borrow();
        state.context.clone()
    }
}

/// Implementations for interacting Rust-world `Window` object and the JS runtime.
#[allow(dead_code)]
impl JavaScriptRuntime {
    /// `window` returns the `Window` object in the Rust world linked to the given isolate.
    pub fn window(isolate: &v8::Isolate) -> Option<Rc<RefCell<Window>>> {
        let state = Self::state(isolate);
        let state = state.borrow();
        state.window.clone()
    }

    /// `get_window` returns the `Window` object in the Rust world linked to the runtime.
    pub fn get_window(&mut self) -> Option<Rc<RefCell<Window>>> {
        Self::window(&self.v8_isolate)
    }

    /// `set_window` links the given `Window` object to the runtime.
    pub fn set_window(&mut self, window: Rc<RefCell<Window>>) {
        self.get_state().borrow_mut().window = Some(window);
    }
}

/// Implementations for interacting Rust-world `Document` object and the JS runtime.
#[allow(dead_code)]
impl JavaScriptRuntime {
    /// `document` returns the `Document` object in the Rust world linked to the given isolate.
    pub fn document(isolate: &v8::Isolate) -> Option<Rc<RefCell<Document>>> {
        let state = Self::state(isolate);
        let state = state.borrow();
        state.document.clone()
    }

    /// `get_window` returns the `Document` object in the Rust world linked to the runtime.
    pub fn get_document(&mut self) -> Option<Rc<RefCell<Document>>> {
        Self::document(&self.v8_isolate)
    }

    /// `set_document` links the given `Document` object to the runtime.
    pub fn set_document(&mut self, document: Rc<RefCell<Document>>) {
        self.get_state().borrow_mut().document = Some(document);
    }
}

/// Implementations for proxying Rust-world PageView API and the JS runtime.
#[allow(dead_code)]
impl JavaScriptRuntime {
    /// `pv_api_handler` returns the `PageViewAPIHandler` object in the Rust world linked to the given isolate.
    pub fn pv_api_handler(isolate: &v8::Isolate) -> Option<Rc<PageViewAPIHandler>> {
        let state = Self::state(isolate);
        let state = state.borrow();
        state.pv_api_handler.clone()
    }

    /// `get_pv_api_handler` returns the `PageViewAPIHandler` object in the Rust world linked to the runtime.
    pub fn get_pv_api_handler(&mut self) -> Option<Rc<PageViewAPIHandler>> {
        Self::pv_api_handler(&self.v8_isolate)
    }

    /// `set_pv_api_handler` links the given `PageViewAPIHandler` object to the runtime.
    pub fn set_pv_api_handler(&mut self, view_api_handler: Rc<PageViewAPIHandler>) {
        self.get_state().borrow_mut().pv_api_handler = Some(view_api_handler);
    }
}

/// `to_pretty_string` formats the `TryCatch` instance into the prettified error string for puppy.
///
/// NOTE: See the following to get full error information.
/// https://github.com/denoland/rusty_v8/blob/0d093a02f658781d52e6d70d138768fc19a79d54/examples/shell.rs#L158
fn to_pretty_string(mut try_catch: v8::TryCatch<v8::HandleScope>) -> String {
    // TODO (enhancement): better error handling needed! wanna remove uncareful unwrap().
    let exception_string = try_catch
        .exception()
        .unwrap()
        .to_string(&mut try_catch)
        .unwrap()
        .to_rust_string_lossy(&mut try_catch);
    let message = try_catch.message().unwrap();

    let filename = message
        .get_script_resource_name(&mut try_catch)
        .map_or_else(
            || "(unknown)".into(),
            |s| {
                s.to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch)
            },
        );
    let line_number = message.get_line_number(&mut try_catch).unwrap_or_default();
    format!("{}:{}: {}", filename, line_number, exception_string)
}

#[cfg(test)]
mod tests {
    use crate::core::dom::{
        element::{AttrMap, Element},
        Document, Text,
    };

    use super::*;
    #[test]
    fn test_execute() {
        let mut runtime = JavaScriptRuntime::new();
        {
            // a simple math
            let r = runtime.execute("", "1 + 1");
            assert!(r.is_ok());
            assert_eq!(r.unwrap(), "2");
        }
        {
            // simple string operation
            let r = runtime.execute("", "'test' + \"func\" + `012${1+1+1}`");
            assert!(r.is_ok());
            assert_eq!(r.unwrap(), "testfunc0123");
        }
        {
            // use of undefined variable
            let r = runtime.execute("", "test");
            assert!(r.is_err());
        }
        {
            // lambda definition
            let r = runtime.execute("", "let inc = (i) => { return i + 1 }; inc(1)");
            assert!(r.is_ok());
            assert_eq!(r.unwrap(), "2");
        }
        {
            // variable reuse
            let r = runtime.execute("", "inc(4)");
            assert!(r.is_ok());
            assert_eq!(r.unwrap(), "5");
        }
    }

    fn setup_runtime(
        runtime: &mut JavaScriptRuntime,
    ) -> (
        Rc<RefCell<Document>>,
        Rc<RefCell<Window>>,
        Rc<PageViewAPIHandler>,
    ) {
        let document = Rc::new(RefCell::new(Document::new(
            "http://example.com".to_string(),
            "http://example.com".to_string(),
            Element::new(
                "p".to_string(),
                AttrMap::new(),
                vec![Text::new("hi".to_string())],
            ),
        )));
        runtime.set_document(document.clone());

        let window = Rc::new(RefCell::new(Window {
            name: "test".to_string(),
        }));
        runtime.set_window(window.clone());

        let (cb_sink, _) = crossbeam_channel::unbounded();
        let cb_sink = Rc::new(cb_sink);
        let api = Rc::new(PageViewAPIHandler::new(cb_sink));
        runtime.set_pv_api_handler(api.clone());

        (document.clone(), window.clone(), api.clone())
    }

    #[test]
    fn test_state_store() {
        let mut runtime = JavaScriptRuntime::new();

        // check pre-state
        {
            let state = runtime.get_state();
            let state = state.borrow_mut();
            assert!(state.document.is_none());
            assert!(state.window.is_none());
            assert!(state.pv_api_handler.is_none());
        }

        // change state
        let (document, window, _) = setup_runtime(&mut runtime);

        // change post-state
        {
            let state = runtime.get_state();
            let state = state.borrow_mut();
            assert_eq!(state.document, Some(document.clone()));
            assert_eq!(state.window, Some(window.clone()));
            assert!(state.pv_api_handler.is_some());
        }
    }

    #[test]
    fn test_window() {
        let mut runtime = JavaScriptRuntime::new();
        let (_, window, _) = setup_runtime(&mut runtime);

        let r_window = runtime.get_window();
        assert_eq!(r_window, Some(window.clone()));
        assert_eq!(
            JavaScriptRuntime::window(&mut runtime.v8_isolate),
            Some(window.clone())
        );
    }

    #[test]
    fn test_document() {
        let mut runtime = JavaScriptRuntime::new();
        let (document, _, _) = setup_runtime(&mut runtime);

        let r_document = runtime.get_document();
        assert_eq!(r_document, Some(document.clone()));
        assert_eq!(
            JavaScriptRuntime::document(&mut runtime.v8_isolate),
            Some(document.clone())
        );
    }

    #[test]
    fn test_api_handler() {
        let mut runtime = JavaScriptRuntime::new();
        let _ = setup_runtime(&mut runtime);

        // run set_pv_api_handler again to mock the channel
        let (cb_sink, cb_recv) = crossbeam_channel::unbounded();
        let cb_sink = Rc::new(cb_sink);
        let api = Rc::new(PageViewAPIHandler::new(cb_sink));
        runtime.set_pv_api_handler(api.clone());

        // run scripts and check api is appropriately called
        assert!(runtime.execute("", "window.alert(1)").is_ok());
        assert!(cb_recv.try_recv().is_ok());
        assert!(cb_recv.try_recv().is_err());
    }
}
