use super::Node;

/// `Document` interface.
/// Here is a list of major WebIDL definition related to the interface:
/// - https://dom.spec.whatwg.org/#interface-document
/// - https://html.spec.whatwg.org/multipage/dom.html#the-document-object
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
