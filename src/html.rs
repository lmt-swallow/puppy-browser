use crate::dom::Node;
use crate::source::Source;

// https://html.spec.whatwg.org/multipage/parsing.html#parsing
fn parse(source: Source) -> Node {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{AttrMap, Document, Element, Text};
    use crate::source::Source;

    #[test]
    fn test_parse_single__without_nest() {
        let url = "http://example.com";
        let s = Source {
            from_url: url.to_string(),
            body: "<p>Hello World</p>".to_string(),
        };
        let got = parse(s);
        let expected = Document::new(
            url.to_string(),
            url.to_string(),
            vec![Element::new(
                "p".to_string(),
                AttrMap::new(),
                vec![Text::new("Hello World".to_string())],
            )],
        )
        .unwrap();
        assert_eq!(got, expected)
    }

    #[test]
    fn test_parse_two_without_nest() {
        let url = "http://example.com";
        let s = Source {
            from_url: url.to_string(),
            body: "<p>Hello World (1)</p><p>Hello World (2)</p>".to_string(),
        };
        let got = parse(s);
        let expected = Document::new(
            url.to_string(),
            url.to_string(),
            vec![
                Element::new(
                    "p".to_string(),
                    AttrMap::new(),
                    vec![Text::new("Hello World (1)".to_string())],
                ),
                Element::new(
                    "p".to_string(),
                    AttrMap::new(),
                    vec![Text::new("Hello World (2)".to_string())],
                ),
            ],
        )
        .unwrap();
        assert_eq!(got, expected)
    }

    #[test]
    fn test_parse_with_nest() {
        let url = "http://example.com";
        let s = Source {
            from_url: url.to_string(),
            body: "<div><p>nested (1)</p><p>nested (2)</p></div>".to_string(),
        };
        let got = parse(s);
        let expected = Document::new(
            url.to_string(),
            url.to_string(),
            vec![Element::new(
                "div".to_string(),
                AttrMap::new(),
                vec![
                    Element::new(
                        "p".to_string(),
                        AttrMap::new(),
                        vec![Text::new("nested (1)".to_string())],
                    ),
                    Element::new(
                        "p".to_string(),
                        AttrMap::new(),
                        vec![Text::new("nested (2)".to_string())],
                    ),
                ],
            )],
        )
        .unwrap();
        assert_eq!(got, expected)
    }
}
