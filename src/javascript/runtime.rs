use crate::{javascript::binding, window::Window};
use rusty_v8 as v8;
use std::{cell::RefCell, rc::Rc, sync::Once};
use thiserror::Error;

#[derive(Debug)]
pub struct JavaScriptRuntimeState {
    pub context: v8::Global<v8::Context>,

    // TODO (enhancement): remove this by GothamState like Deno does.
    pub window: Option<Rc<RefCell<Window>>>,
}

#[derive(Debug)]
pub struct JavaScriptRuntime {
    pub v8_isolate: v8::OwnedIsolate,
}

#[derive(Error, Debug, PartialEq)]
pub enum JavaScriptRuntimeError {
    #[error("failed to parse the script: {message:?}")]
    CompileError { message: String },

    #[error("failed to execute the script: {message:?}")]
    ExecuteError { message: String },
}

/// TODO (docs): write here
impl JavaScriptRuntime {
    pub fn new() -> Self {
        // init v8 platform just once
        static PUPPY_INIT: Once = Once::new();
        PUPPY_INIT.call_once(move || super::v8_init::init_platform());

        // create v8 isolate & context
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = binding::initialize_context(scope);
            v8::Global::new(scope, context)
        };

        // store state inside v8 isolate
        // NOTE: the data would be stored by Isolate::SetData (https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36)
        isolate.set_slot(Rc::new(RefCell::new(JavaScriptRuntimeState {
            context: context,
            window: None,
        })));

        JavaScriptRuntime {
            v8_isolate: isolate,
        }
    }

    pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<JavaScriptRuntimeState>> {
        let s = isolate
            .get_slot::<Rc<RefCell<JavaScriptRuntimeState>>>()
            .unwrap();
        s.clone()
    }

    pub fn get_state(&self) -> Rc<RefCell<JavaScriptRuntimeState>> {
        JavaScriptRuntime::state(&self.v8_isolate)
    }

    pub fn get_handle_scope(&mut self) -> v8::HandleScope {
        let context = self.get_context();
        v8::HandleScope::with_context(&mut self.v8_isolate, context)
    }

    pub fn get_context(&mut self) -> v8::Global<v8::Context> {
        let state = self.get_state();
        let state = state.borrow();
        state.context.clone()
    }

    pub fn get_window(&mut self) -> Option<Rc<RefCell<Window>>> {
        let state = self.get_state();
        let state = state.borrow();
        state.window.clone()
    }

    pub fn set_window(&mut self, window: Rc<RefCell<Window>>) {
        self.get_state().borrow_mut().window = Some(window);
    }

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

// NOTE: See the following to get full error information.
// https://github.com/denoland/rusty_v8/blob/0d093a02f658781d52e6d70d138768fc19a79d54/examples/shell.rs#L158
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