//! This module includes some implementations on window concepts.

/// `Window` interface.
/// Here is a list of major WebIDL definition related to the interface:
/// - https://html.spec.whatwg.org/multipage/window-object.html#the-window-object
#[derive(Debug, PartialEq)]
pub struct Window {
    pub name: String,
}
