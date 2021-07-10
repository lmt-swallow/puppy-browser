//! This module includes some implementations on HTML.

use crate::dom::{AttrMap, Document, Element, Node, Text};
use crate::fetch::Response;
#[allow(unused_imports)]
use combine::EasyParser;
use combine::{
    attempt,
    error::{StreamError, StringStreamError},
    many,
    parser::char::{newline, space},
};
use combine::{between, many1, parser, sep_by, Parser, Stream};
use combine::{choice, error::ParseError};
use combine::{
    parser::char::{char, letter},
    satisfy,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HTMLParseError {
    #[error("failed to parse; {0}")]
    InvalidResourceError(StringStreamError),
}

// [NOTE] Specification on HTML parsing: https://html.spec.whatwg.org/multipage/parsing.html#parsing
//
// The specification defines parsing algorithm of HTML, which takes input stream as argument and emits DOM.
// It consists of the following two stages:
// 1. tokenization stage
// 2. tree construction stage
// The first one, tokenization stage, generates tokens from input stream.
// The latter one, tree construction stage, constructs a DOM while handling scripts inside <script> tags.
//
// This implementation omits details of those two stages for simplicity.
// Please check the following if you'd like to know about the parsing process more deeply:
// - html5ever crate by Serve project https://github.com/servo/html5ever
// - HTMLDocumentParser, HTMLTokenizer, HTMLTreeBuilder of Chromium (src/third_party/blink/renderer/core/html/parser/*)

/// This functions parses `response` as HTML in non-standard manner.
pub fn parse(response: Response) -> Result<Document, HTMLParseError> {
    // NOTE: Here we assume the resource is HTML and encoded by UTF-8.
    // We should determine character encoding as follows:
    // https://html.spec.whatwg.org/multipage/parsing.html#the-input-byte-streama
    let nodes = parse_without_normalziation(response.data);
    match nodes {
        Ok(nodes) => {
            let document_element = if nodes.len() == 1 {
                nodes.into_iter().nth(0).unwrap()
            } else {
                Element::new("html".to_string(), AttrMap::new(), nodes)
            };
            Ok(Document::new(
                response.url.to_string(),
                response.url.to_string(),
                document_element,
            ))
        }
        Err(e) => Err(e),
    }
}

pub fn parse_without_normalziation(data: Vec<u8>) -> Result<Vec<Box<Node>>, HTMLParseError> {
    // NOTE: Here we assume the resource is HTML and encoded by UTF-8.
    // We should determine character encoding as follows:
    // https://html.spec.whatwg.org/multipage/parsing.html#the-input-byte-streama
    let body = String::from_utf8(data).unwrap();

    nodes()
        .parse(&body as &str)
        .map(|(nodes, _)| nodes)
        .map_err(|e| HTMLParseError::InvalidResourceError(e))
}

// `nodes_` (and `nodes`) tries to parse input as Element or Text.
fn nodes_<Input>() -> impl Parser<Input, Output = Vec<Box<Node>>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(many(choice((attempt(element()), attempt(text())))))
}

/// `text` consumes input until `<` comes.
fn text<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c: char| c != '<')).map(|t| Text::new(t))
}

/// `element` consumes `<tag_name attr_name="attr_value" ...>(children)</tag_name>`.
fn element<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (open_tag(), nodes(), close_tag()).and_then(
        |((open_tag_name, attributes), children, close_tag_name)| {
            if open_tag_name == close_tag_name {
                Ok(Element::new(open_tag_name, attributes, children))
            } else {
                Err(<Input::Error as combine::error::ParseError<
                    char,
                    Input::Range,
                    Input::Position,
                >>::StreamError::message_static_message(
                    "tag name of open tag and close tag mismatched",
                ))
            }
        },
    )
}

/// `open_tag` consumes `<tag_name attr_name="attr_value" ...>`.
fn open_tag<Input>() -> impl Parser<Input, Output = (String, AttrMap)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let open_tag_name = many1::<String, _, _>(letter());
    let open_tag_content = (
        open_tag_name,
        many::<String, _, _>(space().or(newline())),
        attributes(),
    )
        .map(|v: (String, _, AttrMap)| (v.0, v.2));
    between(char('<'), char('>'), open_tag_content)
}

/// close_tag consumes `</tag_name>`.
fn close_tag<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let close_tag_name = many1::<String, _, _>(letter());
    let close_tag_content = (char('/'), close_tag_name).map(|v| v.1);
    between(char('<'), char('>'), close_tag_content)
}

/// `attribute` consumes `name="value"`.
fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let attribute_name = many1::<String, _, _>(letter());
    let attribute_inner_value = many1::<String, _, _>(satisfy(|c: char| c != '"'));
    let attribute_value = between(char('"'), char('"'), attribute_inner_value);
    (
        attribute_name,
        many::<String, _, _>(space().or(newline())),
        char('='),
        many::<String, _, _>(space().or(newline())),
        attribute_value,
    )
        .map(|v| (v.0, v.4))
}

/// `attributes` consumes `name1="value1" name2="value2" ... name="value"`
fn attributes<Input>() -> impl Parser<Input, Output = AttrMap>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by::<Vec<(String, String)>, _, _, _>(
        attribute(),
        many::<String, _, _>(space().or(newline())),
    )
    .map(|attrs: Vec<(String, String)>| {
        let m: AttrMap = attrs.into_iter().collect();
        m
    })
}

parser! {
    fn nodes[Input]()(Input) -> Vec<Box<Node>>
    where [Input: Stream<Token = char>]
    {
        nodes_()
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;
    use crate::fetch::{HeaderMap, Response};
    use crate::{
        dom::{AttrMap, Document, Element, Text},
        fetch::{HTTPStatus, ResponseType},
    };

    // parsing tests of attributes
    #[test]
    fn test_parse_attribute() {
        assert_eq!(
            attribute().easy_parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );

        assert_eq!(
            attribute().easy_parse("test = \"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        )
    }

    #[test]
    fn test_parse_attributes() {
        let mut expected_map = AttrMap::new();
        expected_map.insert("test".to_string(), "foobar".to_string());
        assert_eq!(
            attributes().easy_parse("test=\"foobar\""),
            Ok((expected_map, ""))
        );

        assert_eq!(attributes().easy_parse(""), Ok((AttrMap::new(), "")))
    }

    // parsing tests of open tags
    #[test]
    fn test_parse_open_tag_without_attributes() {}

    #[test]
    fn test_parse_open_tag() {
        {
            assert_eq!(
                open_tag().easy_parse("<p>aaaa"),
                Ok((("p".to_string(), AttrMap::new()), "aaaa"))
            );
        }
        {
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            assert_eq!(
                open_tag().easy_parse("<p id=\"test\">"),
                Ok((("p".to_string(), attributes), ""))
            )
        }

        {
            let result = open_tag().easy_parse("<p id=\"test\" class=\"sample\">");
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            attributes.insert("class".to_string(), "sample".to_string());
            assert_eq!(result, Ok((("p".to_string(), attributes), "")));
        }

        {
            assert!(open_tag().easy_parse("<p id>").is_err());
        }
    }

    // parsing tests of close tags
    #[test]
    fn test_parse_close_tag() {
        let result = close_tag().easy_parse("</p>");
        assert_eq!(result, Ok(("p".to_string(), "")))
    }

    // parsing tests of an element
    #[test]
    fn test_parse_element() {
        assert_eq!(
            element().easy_parse("<p></p>"),
            Ok((Element::new("p".to_string(), AttrMap::new(), vec![]), ""))
        );

        assert_eq!(
            element().easy_parse("<p>Hello World</p>"),
            Ok((
                Element::new(
                    "p".to_string(),
                    AttrMap::new(),
                    vec![Text::new("Hello World".to_string())]
                ),
                ""
            ))
        );

        assert!(element().easy_parse("<p>Hello World</div>").is_err());
    }

    // parsing tests of a tag
    #[test]
    fn test_parse_text() {
        {
            assert_eq!(
                text().easy_parse("Hello World"),
                Ok((Text::new("Hello World".to_string()), ""))
            );
        }
        {
            assert_eq!(
                text().easy_parse("Hello World<"),
                Ok((Text::new("Hello World".to_string()), "<"))
            );
        }
    }

    // parsing tests of documents
    #[test]
    fn test_parse_single_without_nest() {
        let url = "http://example.com";
        let s = Response {
            url: Url::parse(url).unwrap(),
            status: HTTPStatus::OK,
            rtype: ResponseType::Basic,
            headers: HeaderMap::new(),
            data: "<p>Hello World</p>".as_bytes().to_vec(),
        };
        let got = parse(s);
        let expected = Ok(Document::new(
            Url::parse(url).unwrap().to_string(),
            Url::parse(url).unwrap().to_string(),
            Element::new(
                "p".to_string(),
                AttrMap::new(),
                vec![Text::new("Hello World".to_string())],
            ),
        ));
        assert_eq!(got, expected)
    }

    #[test]
    fn test_parse_two_without_nest() {
        let url = "http://example.com";
        let s = Response {
            url: Url::parse(url).unwrap(),
            status: HTTPStatus::OK,
            rtype: ResponseType::Basic,
            headers: HeaderMap::new(),
            data: "<p>Hello World (1)</p><p>Hello World (2)</p>"
                .as_bytes()
                .to_vec(),
        };
        let expected = Ok(Document::new(
            Url::parse(url).unwrap().to_string(),
            Url::parse(url).unwrap().to_string(),
            Element::new(
                "html".to_string(),
                AttrMap::new(),
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
            ),
        ));
        assert_eq!(parse(s), expected)
    }

    #[test]
    fn test_parse_with_nest() {
        let url = "http://example.com";
        let s = Response {
            url: Url::parse(url).unwrap(),
            status: HTTPStatus::OK,
            rtype: ResponseType::Basic,
            headers: HeaderMap::new(),
            data: "<div><p>nested (1)</p><p>nested (2)</p></div>"
                .as_bytes()
                .to_vec(),
        };
        let expected = Ok(Document::new(
            Url::parse(url).unwrap().to_string(),
            Url::parse(url).unwrap().to_string(),
            Element::new(
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
            ),
        ));
        assert_eq!(parse(s), expected)
    }
}
