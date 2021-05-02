use std::{cell::RefCell, rc::Rc};

use super::super::components::{Link, TextInputView};
use crate::{
    dom::{Node, NodeType},
    js::{
        binding::{create_object_under, set_property},
        JavaScriptRuntime, JavaScriptRuntimeError,
    },
    ui::{browser_view::with_current_browser_view, traits::Clearable, BrowserView},
    window::Window,
};
use cursive::{
    traits::Boxable,
    view::ViewWrapper,
    views::{Button, LinearLayout, TextView},
};
use log::{error, info, trace};
use rusty_v8 as v8;
use thiserror::Error;

type ElementContainer = LinearLayout;

impl Clearable for ElementContainer {
    fn clear(&mut self) {
        for _ in 0..self.len() {
            self.remove_child(0);
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum RenderError {
    #[error("failed to render")]
    UnknownError,

    #[error("failed to render; unsupported input type {specified_type:?} found")]
    UnsupportedInputTypeError { specified_type: String },

    #[error("failed to render; unsupported node type found")]
    UnsupportedNodeTypeError,

    #[error("failed to render; javascript execution failed")]
    JavaScriptError(JavaScriptRuntimeError),
}

fn render_nodes(
    view: &mut ElementContainer,
    nodes: &Vec<Node>,
    js_runtime: &mut JavaScriptRuntime,
) -> Result<(), RenderError> {
    match nodes
        .iter()
        .map(|node| render_node(view, node, js_runtime))
        .collect::<Result<Vec<()>, RenderError>>()
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn render_node(
    view: &mut ElementContainer,
    node: &Node,
    js_runtime: &mut JavaScriptRuntime,
) -> Result<(), RenderError> {
    match &node.node_type {
        NodeType::Element(ref element) => match element.tag_name.as_str() {
            "script" => match js_runtime.execute("(inline)", node.inner_text().as_str()) {
                Ok(s) => {
                    info!("javascript execution succeeded; {}", s);
                    Ok(())
                }
                Err(e) => Err(RenderError::JavaScriptError(e)),
            },
            "a" => {
                let link_href: String = element
                    .attributes
                    .get("href")
                    .unwrap_or(&"".to_string())
                    .to_string();
                view.add_child(Link::new(node.inner_text(), move |s| {
                    if with_current_browser_view(s, |b: &mut BrowserView| {
                        b.resolve_url(link_href.clone())
                            .and_then(|url| b.navigate_to(url))
                    })
                    .is_none()
                    {
                        error!("failed to initiate navigation by link")
                    }
                }));
                Ok(())
            }
            "input" => match element
                .attributes
                .get("type")
                .unwrap_or(&"".to_string())
                .as_str()
            {
                "text" => {
                    view.add_child(
                        TextInputView::new()
                            .content(element.attributes.get("value").unwrap_or(&"".to_string()))
                            .min_width(10)
                            .max_width(10),
                    );
                    Ok(())
                }
                "button" | "submit" => {
                    view.add_child(Button::new(
                        element.attributes.get("value").unwrap_or(&"".to_string()),
                        |_s| {},
                    ));
                    Ok(())
                }
                t => {
                    info!("unsupported input tag type {} found", t);
                    Err(RenderError::UnsupportedInputTypeError {
                        specified_type: t.to_string(),
                    })
                }
            },
            "button" => {
                view.add_child(Button::new(node.inner_text(), |_s| {}));
                Ok(())
            }
            "html" => render_nodes(view, &node.child_nodes, js_runtime),
            "div" | "span" | "p" => {
                let mut child_view = LinearLayout::horizontal();
                match render_nodes(&mut child_view, &node.child_nodes, js_runtime) {
                    Ok(_) => {
                        view.add_child(child_view);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => render_nodes(view, &node.child_nodes, js_runtime),
        },
        NodeType::Text(ref t) => {
            view.add_child(TextView::new(&t.data));
            Ok(())
        }
        _ => Err(RenderError::UnsupportedNodeTypeError),
    }
}

pub struct PageView {
    window: Rc<RefCell<Window>>,

    view: ElementContainer,
    js_runtime: JavaScriptRuntime,
}

impl PageView {
    pub fn new() -> Self {
        let window = Rc::new(RefCell::new(Window {
            name: "".to_string(),
            document: None,
        }));

        let mut js_runtime = JavaScriptRuntime::new();
        js_runtime.set_window(window.clone());

        let mut p = PageView {
            window: window,
            view: LinearLayout::vertical(),
            js_runtime,
        };
        p.init_context();
        p
    }

    pub fn init_context(&mut self) {
        // TODO (enhancement): wrap v8-related process into `js` crate
        let mut scope = self.js_runtime.get_handle_scope();
        let context = scope.get_current_context();

        let global = context.global(&mut scope);

        // register `window` object
        let window = create_object_under(&mut scope, global, "window");
        set_property(
            &mut scope,
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

        // register `document` object
        // TODO
    }

    pub fn render_document(&mut self, node: Node) -> Result<(), RenderError> {
        let top_element = match node.node_type {
            NodeType::Document(ref _document) => {
                assert_eq!(node.child_nodes.len(), 1);
                node.child_nodes.get(0)
            }
            _ => None,
        };
        let result = if let Some(_top_element) = top_element {
            render_node(&mut self.view, _top_element, &mut self.js_runtime)
        } else {
            Ok(())
        };

        let mut window = self.window.borrow_mut();
        window.document = Some(node);

        result
    }
}

impl ViewWrapper for PageView {
    type V = LinearLayout;

    fn with_view<F, R>(&self, f: F) -> ::std::option::Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&self.view))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> ::std::option::Option<R>
    where
        F: ::std::ops::FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut self.view))
    }

    fn into_inner(self) -> ::std::result::Result<Self::V, Self>
    where
        Self::V: ::std::marker::Sized,
    {
        Ok(self.view)
    }
}

impl Clearable for PageView {
    fn clear(&mut self) {
        self.view.clear()
    }
}
