use std::ffi::c_void;

use super::{set_function_to, set_property, set_property_with_accessor, set_readonly_constant};
use crate::{
    common::{
        dom::{
            element::{AttrMap, Element},
            Node,
        },
        html::parse_without_normalziation,
    },
    javascript::{binding::request_rerender, JavaScriptRuntime},
};
use log::{error, info, trace};
use rusty_v8 as v8;

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

fn get_node_from_internal_ref<'s>(node_v8: v8::Local<v8::Value>) -> &mut NodeRefTarget {
    let node = unsafe { v8::Local::<v8::External>::cast(node_v8) };
    let node = node.value() as *mut NodeRefTarget;
    unsafe { &mut *node }
}

fn to_v8_node<'s>(scope: &mut v8::HandleScope<'s>, node: &Box<Node>) -> v8::Local<v8::Object> {}

fn to_rust_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node: v8::Local<v8::Object>,
) -> Result<&'s Box<Node>, ()> {
    // get innerHTML
    let inner_html = {
        let key = v8::String::new(scope, "innerHTML");
        let key = v8::Private::for_api(scope, key);
        node.get_private(scope, key.into())
            .unwrap_or(v8::String::new(scope, "").unwrap().into())
            .to_rust_string_lossy(scope)
    };

    // get tagName
    let tag_name = {
        let key = v8::String::new(scope, "tagName").unwrap();
        node.get(scope, key.into())
            .unwrap()
            .to_rust_string_lossy(scope)
    };

    info!(
        "appendChild: tag_name={}, inner_html={}",
        tag_name, inner_html
    );

    // parse the value of innerHTML
    let children = match parse_without_normalziation(inner_html.as_bytes().to_vec()) {
        Ok(children) => children,
        Err(e) => {
            error!("failed to parse new HTML; {}", e);
            // TODO (enhancement): throw Error appropriately
            return Err(());
        }
    };

    Ok(&Element::new(tag_name, AttrMap::new(), children))
}

/// This function creates a new `Node` object.
///
/// Here are major standards on this object:
/// - https://dom.spec.whatwg.org/#interface-node
///
/// TODO (enhancement): fix memory leak caused by Box::leak()
fn create_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_ref: Option<Box<Node>>,
) -> v8::Local<'s, v8::Object> {
    let template = v8::ObjectTemplate::new(scope);
    // set properties
    {
        // add `appendChild` function
        // standard: https://dom.spec.whatwg.org/#dom-node-appendchild
        // NOTE for readers: the following implementation assumes `target_node.appendChild(node_to_add)` called.
        let cb = v8::FunctionTemplate::new(
            scope,
            move |scope: &mut v8::HandleScope,
                  args: v8::FunctionCallbackArguments,
                  mut retval: v8::ReturnValue| {
                let node_to_add_v8 = {
                    let node_to_add = args.get(0);
                    // validate node_to_add
                    if !node_to_add.is_object() {
                        // TODO (enhancement): throw Error appropriately
                        return;
                    }
                    node_to_add.to_object(scope).unwrap()
                };

                // check target_node exists in current DOM tree
                let target_node_ref = args.this().get_internal_field(scope, 0).unwrap();
                if target_node_ref.is_undefined() {
                    // => This node is not linked with to the actual node.
                    // we need to update target_node's innerHTML!

                    let n = to_rust_node(scope, args.this());
                    if n.is_err() {
                        return;
                    }
                    n.unwrap();
                } else {
                    // => This node is linked with to a node.
                    // we need to update the rust-side tree
                    let target_node = get_node_from_internal_ref(target_node_ref);
                    let node_to_add_rust = {
                        let n = to_rust_node(scope, node_to_add_v8);
                        if n.is_err() {
                            // TODO (enhancement): throw Error appropriately
                            return;
                        }
                        n.unwrap()
                    };

                    // insert node to the target
                    target_node.append_child(*node_to_add_rust);
                    let added_node = target_node.children.last_mut().unwrap();

                    // set (a ref for node_to_add_v8) to node_to_add_rust
                    set_node_internal_ref(scope, added_node, node_to_add_v8);

                    // all set! gonna return back :-)
                    retval.set(node_to_add_v8.into());
                    request_rerender(scope, "appendChild()");
                }
            },
        );

        template.set(
            v8::String::new(scope, "appendChild").unwrap().into(),
            cb.into(),
        );
    }

    // extend internal field capacity to store node ID (which is used to identify the actual node in PageView)
    template.set_internal_field_count(1);

    // create new node instance
    let node = template.new_instance(scope).unwrap();

    // set a reference to Node into the internal field
    let node_ref_v8: v8::Local<v8::Value> = if let Some(mut node_ref) = node_ref {
        let boxed_ref = Box::new(&mut node_ref);
        let v8_ext = v8::External::new(
            scope,
            Box::leak(boxed_ref) as *mut NodeRefTarget as *mut c_void,
        );
        v8_ext.into()
    } else {
        v8::undefined(scope).into()
    };
    node.set_internal_field(0, node_ref_v8);

    // all set :-)
    node
}

/// This function creates a new `Element` object.
///
/// Here are major standards on this object:
/// - https://dom.spec.whatwg.org/#interface-element
fn create_element<'s>(
    scope: &mut v8::HandleScope<'s>,
    tag_name: &str,
    node_ref: Option<Box<Node>>,
) -> v8::Local<'s, v8::Object> {
    let node = create_node(scope, node_ref);

    // set properties
    {
        // add `tagName` property
        let tag_name = v8::String::new(scope, tag_name).unwrap();
        set_readonly_constant(scope, node, "tagName", tag_name.into());
    }
    {
        // add `innerHTML` property
        set_property_with_accessor(
            scope,
            node,
            "innerHTML",
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  args: v8::PropertyCallbackArguments,
                  mut rv: v8::ReturnValue| {
                let this = args.this();
                let node = this.get_internal_field(scope, 0).unwrap();

                if node.is_undefined() {
                    // => This node is not linked with to the actual node.
                    let private_key = v8::String::new(scope, "innerHTML");
                    let private_key = v8::Private::for_api(scope, private_key);
                    rv.set(
                        args.this()
                            .get_private(scope, private_key.into())
                            .unwrap_or(v8::String::new(scope, "").unwrap().into()),
                    );
                } else {
                    // => This node is linked with to a node.
                    let node = unsafe { v8::Local::<v8::External>::cast(node) };
                    let node = node.value() as *mut NodeRefTarget;

                    // TODO: get values from original DOM.
                }
            },
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  value: v8::Local<v8::Value>,
                  args: v8::PropertyCallbackArguments| {
                let this = args.this();
                let node = this.get_internal_field(scope, 0).unwrap();

                if node.is_undefined() {
                    // => This node is not linked with to the actual node.
                    let private_key = v8::String::new(scope, "innerHTML");
                    let private_key = v8::Private::for_api(scope, private_key);
                    args.this().set_private(scope, private_key.into(), value);
                } else {
                    // => This node is linked with to a node.
                    let node = unsafe { v8::Local::<v8::External>::cast(node) };
                    let node = node.value() as *mut NodeRefTarget;

                    // TODO: set values of original DOM.
                    request_rerender(scope, "setter of innerHTML");
                }
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
    let document = create_node(scope, None);

    // set properties
    {
        // add `createElement` function
        // standard: https://dom.spec.whatwg.org/#dom-document-createelement
        set_function_to(
            scope,
            document,
            "createElement",
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             mut retval: v8::ReturnValue| {
                let tag_name = args
                    .get(0)
                    .to_string(scope)
                    .unwrap()
                    .to_rust_string_lossy(scope);
                trace!("createElement called with: {}", tag_name);
                retval.set(create_element(scope, tag_name.as_str(), None).into());
            },
        );
    }
    {
        // add `getElementById` function
        // standard: https://dom.spec.whatwg.org/#dom-document-createelement
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
                trace!("getElementById called with: {}", id);

                // get puppy's document object
                let document = match JavaScriptRuntime::document(scope) {
                    Some(_document) => _document,
                    None => {
                        error!("failed to get document reference; document is None");
                        return;
                    }
                };
                let mut document = document.borrow_mut();
                match document.get_element_by_id(id) {
                    Some(node_rust) => {
                        retval.set();
                    }
                    None => {
                        retval.set(v8::undefined(scope).into());
                    }
                }
            },
        );
    }

    document
}

pub fn initialize_dom<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) {
    let document = create_document_object(scope);
    set_property(scope, global, "document", document.into());
}
