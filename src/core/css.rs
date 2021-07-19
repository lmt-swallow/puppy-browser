//! This module includes some implementations on Cascade Style Sheets (CSS).

use super::dom::{Node, NodeType};
use combine::{
    choice,
    error::StreamError,
    error::StringStreamError,
    many, many1, optional,
    parser::char::{self, letter, newline, space},
    sep_by, sep_end_by, ParseError, Parser, Stream,
};
use thiserror::Error;

/// `Stylesheet` represents a single stylesheet.
/// It consists of multiple rules, which are called "rule-list" in the standard (https://www.w3.org/TR/css-syntax-3/).
#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Stylesheet { rules: rules }
    }
}

/// `Rule` represents a single CSS rule.
/// - *at-rule* such as `@font-face (...)`. it is defined at https://www.w3.org/TR/css-syntax-3/#at-rule
/// - *qualified rule* such as `h1 { .... }`. it is defined at https://www.w3.org/TR/css-syntax-3/#qualified-rule
#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        self.selectors.iter().any(|s| s.matches(n))
    }
}

/// `Selector` represents a sequence of simple selectors separated by combinators.
/// `div > p`, for instance, is a sequence two simple selectors; "div" and "p" is concatenated with combinator ">".
///
/// However, handling this type of sequences is too hard & boring.
/// puppy does not support the combination of selectors. In puppy, slector is just a simple slector, not a sequence of simple selectors.
/// TODO (enhancement): make this compliant to the standard!
pub type Selector = SimpleSelector;

/// `SimpleSelector` represents a simple selector defined in the following standard:
/// https://www.w3.org/TR/selectors-3/#selector-syntax
#[derive(Debug, PartialEq)]
pub enum SimpleSelector {
    UniversalSelector,
    TypeSelector {
        tag_name: String,
    },
    AttributeSelector {
        tag_name: String,
        op: AttributeSelectorOp,
        attribute: String,
        value: String,
    },
    ClassSelector {
        class_name: String,
    },
    // TODO (enhancement): support multiple attribute selectors like `a[href=bar][ping=foo]`
    // TODO (enhancement): support more attribute selectors
}

impl SimpleSelector {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        match self {
            SimpleSelector::UniversalSelector => true,
            SimpleSelector::TypeSelector { tag_name } => match n.node_type {
                NodeType::Element(ref e) => e.tag_name.as_str() == tag_name,
                _ => false,
            },
            SimpleSelector::AttributeSelector {
                tag_name,
                op,
                attribute,
                value,
            } => match n.node_type {
                NodeType::Element(ref e) => {
                    e.tag_name.as_str() == tag_name
                        && match op {
                            AttributeSelectorOp::Eq => e.attributes.get(attribute) == Some(value),
                            AttributeSelectorOp::Contain => e
                                .attributes
                                .get(attribute)
                                .map(|value| {
                                    value
                                        .split_ascii_whitespace()
                                        .find(|v| v == value)
                                        .is_some()
                                })
                                .unwrap_or(false),
                        }
                }
                _ => false,
            },
            SimpleSelector::ClassSelector { class_name } => match n.node_type {
                NodeType::Element(ref e) => e.attributes.get("class") == Some(class_name),
                _ => false,
            },
        }
    }
}

/// `AttributeSelectorOp` is an operator which is allowed to use.
/// See https://www.w3.org/TR/selectors-3/#attribute-selectors to check the full list of available operators.
#[derive(Debug, PartialEq)]
pub enum AttributeSelectorOp {
    Eq, // =
    Contain, // ~=
        // TODO (enhancement): support more attribute selectors
}

/// `Declaration` represents a CSS declaration defined at [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/#declaration)
///
/// Declarations are further categorized into the followings:
/// - descriptors, which are mostly used in "at-rules" like `@foo (bar: piyo)` https://www.w3.org/Style/CSS/all-descriptors.en.html
/// - properties, which are mostly used in "qualified rules" like `.foo {bar: piyo}` https://www.w3.org/Style/CSS/all-descriptors.en.html
///
/// For simplicity, we handle two types of declarations together.
#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: CSSValue,
    // TODO (enhancement): add a field for `!important`
}

/// `CSSValue` represents some of *component value types* defined at [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/#component-types).
#[derive(Debug, PartialEq, Clone)]
pub enum CSSValue {
    Keyword(String),
    Length((usize, Unit)),
}

/// `Unit` describes *a relative length unit* defined at [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/#lengths)
#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Em,
    // TODO (enhancement): add more units here from the definition.
}

/// `CSSParseError` describes an error occured during CSS parsing.
#[derive(Error, Debug, PartialEq)]
pub enum CSSParseError {
    #[error("failed to parse; {0}")]
    InvalidResourceError(StringStreamError),
}

// [NOTE] Specification on CSS parsing https://www.w3.org/TR/css-syntax-3/#parsing-overview
//
// The specification defines parsing algorithm of HTML, which takes input stream as argument and emits DOM.
// It consists of the following two stages:
// 1. tokenization stage
// 2. tree construction stage
// The first one, tokenization stage, generates tokens from input stream.
// The latter one, tree construction stage, constructs a DOM while handling scripts inside <script> tags.

// This functions parses `response` as CSS in non-standard manner.
pub fn parse(raw: String) -> Result<Stylesheet, CSSParseError> {
    rules()
        .parse(raw.as_str())
        .map(|(rules, _)| Stylesheet::new(rules))
        .map_err(|e| CSSParseError::InvalidResourceError(e))
}

fn whitespaces<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many::<String, _, _>(space().or(newline()))
}

fn rules<Input>() -> impl Parser<Input, Output = Vec<Rule>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (whitespaces(), many(rule().skip(whitespaces()))).map(|(_, rules)| rules)
}

fn rule<Input>() -> impl Parser<Input, Output = Rule>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        selectors().skip(whitespaces()),
        char::char('{').skip(whitespaces()),
        declarations().skip(whitespaces()),
        char::char('}'),
    )
        .map(|(selectors, _, declarations, _)| Rule {
            selectors: selectors,
            declarations,
        })
}

fn selectors<Input>() -> impl Parser<Input, Output = Vec<Selector>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by(
        selector().skip(whitespaces()),
        char::char(',').skip(whitespaces()),
    )
}

fn selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    simple_selector()
}

fn simple_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let universal_selector = char::char('*').map(|_| SimpleSelector::UniversalSelector);
    let class_selector =
        (char::char('.'), many1(letter())).map(|(_, class_name)| SimpleSelector::ClassSelector {
            class_name: class_name,
        });
    let type_or_attribute_selector = (
        many1(letter()).skip(whitespaces()),
        optional((
            char::char('[').skip(whitespaces()),
            many1(letter()),
            choice((char::string("="), char::string("~="))),
            many1(letter()),
            char::char(']'),
        )),
    )
        .and_then(|(tag_name, opts)| match opts {
            Some((_, attribute, op, value, _)) => {
                let op = match op {
                    "=" => AttributeSelectorOp::Eq,
                    "~=" => AttributeSelectorOp::Contain,
                    _ => {
                        return Err(<Input::Error as combine::error::ParseError<
                            char,
                            Input::Range,
                            Input::Position,
                        >>::StreamError::message_static_message(
                            "invalid attribute selector op",
                        ))
                    }
                };
                Ok(SimpleSelector::AttributeSelector {
                    tag_name: tag_name,
                    attribute: attribute,
                    op: op,
                    value: value,
                })
            }
            None => Ok(SimpleSelector::TypeSelector { tag_name: tag_name }),
        });

    choice((
        universal_selector,
        class_selector,
        type_or_attribute_selector,
    ))
}

fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by(
        declaration().skip(whitespaces()),
        char::char(';').skip(whitespaces()),
    )
}

fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1(letter()).skip(whitespaces()),
        char::char(':').skip(whitespaces()),
        css_value(),
    )
        .map(|(k, _, v)| Declaration { name: k, value: v })
}

fn css_value<Input>() -> impl Parser<Input, Output = CSSValue>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let keyword = many1(letter()).map(|s| CSSValue::Keyword(s));
    let length = (
        many1(char::digit()).map(|s: String| s.parse::<usize>().unwrap()),
        char::string("em"),
    )
        .map(|(num, _unit)| CSSValue::Length((num, Unit::Em)));
    choice((keyword, length))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stylesheet() {
        assert_eq!(
            parse("test [foo=bar] { aa: bb; cc: 1em } rule { ee: dd;  }".to_string()),
            Ok(Stylesheet::new(vec![
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![
                        Declaration {
                            name: "aa".to_string(),
                            value: CSSValue::Keyword("bb".to_string())
                        },
                        Declaration {
                            name: "cc".to_string(),
                            value: CSSValue::Length((1, Unit::Em)),
                        }
                    ]
                },
                Rule {
                    selectors: vec![SimpleSelector::TypeSelector {
                        tag_name: "rule".to_string(),
                    }],
                    declarations: vec![Declaration {
                        name: "ee".to_string(),
                        value: CSSValue::Keyword("dd".to_string())
                    }]
                },
            ]))
        );
    }

    #[test]
    fn test_rule() {
        assert_eq!(
            rule().parse("test [foo=bar] {}"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![]
                },
                ""
            ))
        );

        assert_eq!(
            rule().parse("test [foo=bar], testtest[piyo~=guoo] {}"),
            Ok((
                Rule {
                    selectors: vec![
                        SimpleSelector::AttributeSelector {
                            tag_name: "test".to_string(),
                            attribute: "foo".to_string(),
                            op: AttributeSelectorOp::Eq,
                            value: "bar".to_string()
                        },
                        SimpleSelector::AttributeSelector {
                            tag_name: "testtest".to_string(),
                            attribute: "piyo".to_string(),
                            op: AttributeSelectorOp::Contain,
                            value: "guoo".to_string()
                        }
                    ],
                    declarations: vec![]
                },
                ""
            ))
        );

        assert_eq!(
            rule().parse("test [foo=bar] { aa: bb; cc: 1em }"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![
                        Declaration {
                            name: "aa".to_string(),
                            value: CSSValue::Keyword("bb".to_string())
                        },
                        Declaration {
                            name: "cc".to_string(),
                            value: CSSValue::Length((1, Unit::Em)),
                        }
                    ]
                },
                ""
            ))
        );
    }

    #[test]
    fn test_selectors() {
        assert_eq!(
            selectors().parse("test [aa=bb], piyo[cc~=dd] {"),
            Ok((
                vec![
                    SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "aa".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bb".to_string()
                    },
                    SimpleSelector::AttributeSelector {
                        tag_name: "piyo".to_string(),
                        attribute: "cc".to_string(),
                        op: AttributeSelectorOp::Contain,
                        value: "dd".to_string()
                    }
                ],
                "{"
            ))
        );
    }

    #[test]
    fn test_declarations() {
        assert_eq!(
            declarations().parse("foo: bar; piyo: guoo }"),
            Ok((
                vec![
                    Declaration {
                        name: "foo".to_string(),
                        value: CSSValue::Keyword("bar".to_string())
                    },
                    Declaration {
                        name: "piyo".to_string(),
                        value: CSSValue::Keyword("guoo".to_string())
                    }
                ],
                "}"
            ))
        );

        assert_eq!(
            declarations().parse("foo: bar; piyo: 1em; }"),
            Ok((
                vec![
                    Declaration {
                        name: "foo".to_string(),
                        value: CSSValue::Keyword("bar".to_string())
                    },
                    Declaration {
                        name: "piyo".to_string(),
                        value: CSSValue::Length((1, Unit::Em))
                    }
                ],
                "}"
            ))
        );
    }

    #[test]
    fn test_selector() {
        assert_eq!(
            selector().parse("test [foo=bar], "),
            Ok((
                SimpleSelector::AttributeSelector {
                    tag_name: "test".to_string(),
                    attribute: "foo".to_string(),
                    op: AttributeSelectorOp::Eq,
                    value: "bar".to_string()
                },
                ", "
            ))
        );
    }

    #[test]
    fn test_simple_selector() {
        assert_eq!(
            simple_selector().parse("* {"),
            Ok((SimpleSelector::UniversalSelector, " {"))
        );

        assert_eq!(
            simple_selector().parse("test{"),
            Ok((
                SimpleSelector::TypeSelector {
                    tag_name: "test".to_string(),
                },
                "{"
            ))
        );

        assert_eq!(
            simple_selector().parse("test [foo=bar] "),
            Ok((
                SimpleSelector::AttributeSelector {
                    tag_name: "test".to_string(),
                    attribute: "foo".to_string(),
                    op: AttributeSelectorOp::Eq,
                    value: "bar".to_string()
                },
                " "
            ))
        );

        assert_eq!(
            simple_selector().parse("test[foo~=bar]{"),
            Ok((
                SimpleSelector::AttributeSelector {
                    tag_name: "test".to_string(),
                    attribute: "foo".to_string(),
                    op: AttributeSelectorOp::Contain,
                    value: "bar".to_string()
                },
                "{"
            ))
        );
    }

    #[test]
    fn test_declaration() {
        assert_eq!(
            declaration().parse("keykey:1em"),
            Ok((
                Declaration {
                    name: "keykey".to_string(),
                    value: CSSValue::Length((1, Unit::Em))
                },
                ""
            ))
        );

        assert_eq!(
            declaration().parse("keyabc : 3em "),
            Ok((
                Declaration {
                    name: "keyabc".to_string(),
                    value: CSSValue::Length((3, Unit::Em))
                },
                " "
            ))
        );

        assert_eq!(
            declaration().parse("keyhello : piyo "),
            Ok((
                Declaration {
                    name: "keyhello".to_string(),
                    value: CSSValue::Keyword("piyo".to_string()),
                },
                " "
            ))
        );

        assert!(declaration().parse("aaaaa").is_err())
    }

    #[test]
    fn test_css_value() {
        let expected = css_value().parse("1em");
        assert_eq!(expected, Ok((CSSValue::Length((1, Unit::Em)), "")))
    }
}
