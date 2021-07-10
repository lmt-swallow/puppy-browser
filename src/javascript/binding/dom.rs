//! # DOM API
//!
//! This module includes implementations of a subset of DOM API (https://dom.spec.whatwg.org/).

use std::ffi::c_void;

use super::{set_accessor_to, set_constant_to, set_function_to, set_property_to};
use crate::{
    core::dom::{Node, NodeType},
    javascript::{api::request_rerender, JavaScriptRuntime},
};
use log::error;
use rusty_v8 as v8;

/// This function creates a new `Node` object.
///
/// Here are major standards on this object:
/// - https://dom.spec.whatwg.org/#interface-node
///
/// TODO (enhancement): fix memory leak caused by Box::leak()
fn create_v8_node<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Object> {
    let template = v8::ObjectTemplate::new(scope);

    // set properties
    // TODO (enhancement): add appendChild etc.

    // extend internal field capacity to store node ID (which is used to identify the actual node in PageView)
    template.set_internal_field_count(1);

    // create new node instance
    template.new_instance(scope).unwrap()
}

/// This function creates a new `Element` object.
///
/// Here are major standards on this object:
/// - https://dom.spec.whatwg.org/#interface-element
fn to_v8_element<'s>(
    scope: &mut v8::HandleScope<'s>,
    tag_name: &str,
    attributes: Vec<(String, String)>,
    node_rust: NodeRefTarget,
) -> v8::Local<'s, v8::Object> {
    let node = to_v8_node(scope, node_rust);

    // set properties
    {
        // add `tagName` property
        let tag_name = v8::String::new(scope, tag_name).unwrap();
        set_constant_to(scope, node, "tagName", tag_name.into());
    }
    {
        // set attributes as properties
        for (key, value) in attributes {
            let value = v8::String::new(scope, value.as_str()).unwrap();
            set_constant_to(scope, node, key.as_str(), value.into());
        }
    }
    {
        // add `innerHTML` property
        // TODO (security): the setter might cause dangling pointer from v8 to rust's heap.
        // This is because objects returned by `document.all` have pointers to rust's heap their own internal fields,
        // and they will be alive after setting values to (any node).`innerHTML` and some node are deleted from the heap.
        set_accessor_to(
            scope,
            node,
            "innerHTML",
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  args: v8::PropertyCallbackArguments,
                  mut rv: v8::ReturnValue| {
                let this = args.this();
                let node = to_linked_rust_node(scope, this);

                let ret = v8::String::new(scope, node.inner_html().as_str()).unwrap();
                rv.set(ret.into());
            },
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  value: v8::Local<v8::Value>,
                  args: v8::PropertyCallbackArguments| {
                let this = args.this();
                let node = to_linked_rust_node(scope, this);
                if let Err(e) = node.set_inner_html(value.to_rust_string_lossy(scope)) {
                    error!("failed to set innerHTML; {}", e);
                }
                request_rerender(scope, "setter of innerHTML");
            },
        );
    }

    node
}

/// This function creates a new `Document` object.
///
/// Here are major standards on this object:
/// - https://dom.spec.whatwg.org/#interface-document
/// - https://html.spec.whatwg.org/multipage/dom.html#the-document-object
fn create_document_object<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Object> {
    let document = create_v8_node(scope);

    // set properties
    {
        // add `all` property (too old though!)
        // standard: https://dom.spec.whatwg.org/#dom-document-createelement
        set_accessor_to(
            scope,
            document,
            "all",
            |scope: &mut v8::HandleScope,
             _key: v8::Local<v8::Name>,
             _args: v8::PropertyCallbackArguments,
             mut rv: v8::ReturnValue| {
                // get puppy's document object
                let document = match JavaScriptRuntime::document(scope) {
                    Some(_document) => _document,
                    None => {
                        error!("failed to get document reference; document is None");
                        return;
                    }
                };
                let mut document = document.borrow_mut();

                // get all nodes
                let document_element = &mut document.document_element;

                let mut f = |n: &mut Box<Node>| -> Option<v8::Local<v8::Value>> {
                    let (tag_name, attributes) = match n.node_type {
                        NodeType::Element(ref e) => (e.tag_name.clone(), e.attributes()),
                        _ => return None,
                    };
                    Some(to_v8_element(scope, tag_name.as_str(), attributes, n).into())
                };

                let all: Vec<v8::Local<v8::Value>> = map_mut(document_element, &mut f)
                    .into_iter()
                    .filter_map(|n| n)
                    .collect();
                let all = v8::Array::new_with_elements(scope, all.as_slice());

                // all set!
                rv.set(all.into());
            },
            |_scope: &mut v8::HandleScope,
             _key: v8::Local<v8::Name>,
             _value: v8::Local<v8::Value>,
             _args: v8::PropertyCallbackArguments| {},
        );
    }
    {
        // `getElementById` property
        set_function_to(
            scope,
            document,
            "getElementById",
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             mut retval: v8::ReturnValue| {
                let id = args
                    .get(0)
                    .to_string(scope)
                    .unwrap()
                    .to_rust_string_lossy(scope);
                // get puppy's document object
                let document = match JavaScriptRuntime::document(scope) {
                    Some(_document) => _document,
                    None => {
                        error!("failed to get document reference; document is None");
                        return;
                    }
                };
                let mut document = document.borrow_mut();

                // get all nodes
                let document_element = &mut document.document_element;

                let mut f = |n: &mut Box<Node>| -> Option<v8::Local<v8::Value>> {
                    let (tag_name, attributes) = match n.node_type {
                        NodeType::Element(ref e) => {
                            if e.id().map(|eid| eid.to_string() == id).unwrap_or(false) {
                                (e.tag_name.clone(), e.attributes())
                            } else {
                                return None;
                            }
                        }
                        _ => return None,
                    };
                    Some(to_v8_element(scope, tag_name.as_str(), attributes, n).into())
                };

                let element: v8::Local<v8::Value> = map_mut(document_element, &mut f)
                    .into_iter()
                    .find_map(|n| n)
                    .unwrap_or(v8::undefined(scope).into());

                // all set!
                retval.set(element.into());
            },
        );
    }

    document
}

/// This function sets `document` object into `global`.
pub fn initialize_dom<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) {
    let document = create_document_object(scope);
    set_property_to(scope, global, "document", document.into());
}

// utilities
// =========

type NodeRefTarget<'a> = &'a mut Box<Node>;

fn set_node_internal_ref<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_rust: NodeRefTarget,
    node_v8: v8::Local<v8::Object>,
) {
    let boxed_ref = Box::new(node_rust);
    let addr = Box::leak(boxed_ref) as *mut NodeRefTarget as *mut c_void;
    let v8_ext = v8::External::new(scope, addr);
    let target_node_ref_v8: v8::Local<v8::Value> = v8_ext.into();
    node_v8.set_internal_field(0, target_node_ref_v8);
}

fn to_linked_rust_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_v8: v8::Local<v8::Object>,
) -> &'s mut NodeRefTarget<'s> {
    let node_v8 = node_v8.get_internal_field(scope, 0).unwrap();
    let node = unsafe { v8::Local::<v8::External>::cast(node_v8) };
    let node = node.value() as *mut NodeRefTarget;
    unsafe { &mut *node }
}

fn to_v8_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_rust: NodeRefTarget,
) -> v8::Local<'s, v8::Object> {
    // create new node instance
    let node_v8 = create_v8_node(scope);

    // set a reference to Node into the internal field
    set_node_internal_ref(scope, node_rust, node_v8);

    // all set :-)
    node_v8
}

fn map_mut<T, F>(node: NodeRefTarget, f: &mut F) -> Vec<T>
where
    F: FnMut(&mut Box<Node>) -> T,
{
    let mut v: Vec<T> = vec![];

    for child in &mut node.children {
        v.push(f(child));
        v.extend(map_mut(child, f));
    }

    v.push(f(node));
    v
}
