//! This module defines some interfaces related to `Document` interface.

use crate::core::dom::NodeType;

use super::Node;

/// `Document` is a kind of `Node`. Here is a list of major WebIDL definition related to the interface:
/// - https://dom.spec.whatwg.org/#interface-document
/// - https://html.spec.whatwg.org/multipage/dom.html#the-document-object
///
/// In the standard, `Document` interface inherits `Node` objects,
/// indicating that it's one of the options to include the following `Document` struct in `super::NodeType`.
/// However, if you do so, our implementation will be more complex. Thie is because the behaviour of `Document` and `Element` is much different!
#[derive(Debug, PartialEq)]
pub struct Document {
    pub url: String,
    pub document_uri: String,
    pub document_element: Box<Node>,
}

impl Document {
    pub fn new(url: String, document_uri: String, document_element: Box<Node>) -> Document {
        Document {
            url: url,
            document_uri: document_uri,
            document_element: document_element,
        }
    }

    pub fn collect_tag_inners(&self, tag_name: &str) -> Vec<String> {
        fn intl(node: &Box<Node>, tag_name: &str) -> Vec<String> {
            if let NodeType::Element(ref element) = node.node_type {
                if element.tag_name.as_str() == tag_name {
                    return vec![node.inner_text()];
                }
            }

            node.children
                .iter()
                .map(|child| intl(child, tag_name))
                .collect::<Vec<Vec<String>>>()
                .into_iter()
                .flatten()
                .collect()
        }
        intl(&self.document_element, tag_name)
    }

    pub fn get_script_inners(&self) -> Vec<String> {
        self.collect_tag_inners("script")
    }

    pub fn get_style_inners(&self) -> Vec<String> {
        self.collect_tag_inners("style")
    }
}

#[cfg(test)]
mod tests {
    use crate::dom::{AttrMap, Document, Element};

    #[test]
    fn test_valid_new() {
        let url = "http://example.com";
        let document = Document::new(
            url.to_string(),
            url.to_string(),
            Element::new("p".to_string(), AttrMap::new(), vec![]),
        );

        assert_eq!(document.url, url.to_string());
        assert_eq!(document.document_uri, url.to_string());
        assert_eq!(
            document.document_element,
            Element::new("p".to_string(), AttrMap::new(), vec![])
        );
    }
}
