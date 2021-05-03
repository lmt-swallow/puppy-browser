use super::{DOMException, Node, NodeType};

/// `Document` interface.
/// Here is a list of major WebIDL definition related to the interface:
/// - https://dom.spec.whatwg.org/#interface-document
/// - https://html.spec.whatwg.org/multipage/dom.html#the-document-object
#[derive(Debug, PartialEq)]
pub struct Document {
    pub url: String,
    pub document_uri: String,
}

impl Document {
    pub fn new(
        url: String,
        document_uri: String,
        children: Vec<Node>,
    ) -> Result<Node, DOMException> {
        if children.len() != 1 {
            Err(DOMException::InvalidDocumentElement)
        } else {
            Ok(Node {
                node_type: NodeType::Document(Document {
                    url: url,
                    document_uri: document_uri,
                }),
                children,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dom::{AttrMap, Document, Element, NodeType};

    #[test]
    fn test_valid_new() {
        let url = "http://example.com";
        if let Ok(document) = Document::new(url.to_string(), url.to_string(), vec![]) {
            match document.node_type {
                NodeType::Document(props) => {
                    assert_eq!(props.url, url.to_string());
                    assert_eq!(props.document_uri, url.to_string());
                }
                _ => {
                    assert!(false);
                }
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_invalid_new() {
        let url = "http://example.com";
        assert!(Document::new(
            url.to_string(),
            url.to_string(),
            vec![
                Element::new("p".to_string(), AttrMap::new(), vec![]),
                Element::new("p".to_string(), AttrMap::new(), vec![]),
            ],
        )
        .is_err());

        assert!(Document::new(url.to_string(), url.to_string(), vec![],).is_err());
    }
}
