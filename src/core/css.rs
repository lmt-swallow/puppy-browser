use super::fetch::Response;

use combine::{
    choice,
    error::StreamError,
    error::StringStreamError,
    many, many1, optional,
    parser::char::{self, letter, spaces},
    sep_by, sep_end_by, ParseError, Parser, Stream,
};
use thiserror::Error;

/// `Stylesheet` represents a single stylesheet.
/// It consists of multiple rules, which are called "rule-list" in the standard (https://www.w3.org/TR/css-syntax-3/).
#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    rules: Vec<Rule>,
}

/// `Rule` represents a single CSS rule.
/// - *at-rule* such as `@font-face (...)`. it is defined at https://www.w3.org/TR/css-syntax-3/#at-rule
/// - *qualified rule* such as `h1 { .... }`. it is defined at https://www.w3.org/TR/css-syntax-3/#qualified-rule
#[derive(Debug, PartialEq)]
pub struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
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
    // TODO (enhancement): support multiple attribute selectors like `a[href=bar][ping=foo]`
    // TODO (enhancement): support more attribute selectors
}

/// `AttributeSelectorOp` is an operator which is allowed to use.
/// See https://www.w3.org/TR/selectors-3/#attribute-selectors to check the full list of available operators.
#[derive(Debug, PartialEq)]
pub enum AttributeSelectorOp {
    Eq, // =
    Contain, // ~=
        // TODO (enhancement): support more attribute selectors
}

/// `Declaration` represents a CSS declaration defined in the following standard:
/// - https://www.w3.org/TR/css-syntax-3/#declaration
///
/// Declarations are further categorized into the followings:
/// - descriptors, which are mostly used in "at-rules" like `@foo (bar: piyo)` https://www.w3.org/Style/CSS/all-descriptors.en.html
/// - properties, which are mostly used in "qualified rules" like `.foo {bar: piyo}` https://www.w3.org/Style/CSS/all-descriptors.en.html
///
/// For simplicity, we handle two types of declarations together.
#[derive(Debug, PartialEq)]
pub struct Declaration {
    name: String,
    value: CSSValue,
    // TODO (enhancement): add a field for `!important`
}

/// `CSSValue` represents some of component value types.
/// See the following to check the definition of component value types:
/// - https://www.w3.org/TR/css-values-3/#component-types
#[derive(Debug, PartialEq)]
pub enum CSSValue {
    Keyword(String),
    Length((usize, Unit)),
}

#[derive(Debug, PartialEq)]
pub enum Unit {
    Em,
    // TODO (enhancement): add more units here from https://www.w3.org/TR/css-values-3/#lengths
}

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
pub fn parse(response: Response) -> Result<Stylesheet, CSSParseError> {
    let raw = String::from_utf8(response.data).unwrap();
    let result = rules().parse(raw.as_str());
    result
        .map(|(stylesheet, _)| Stylesheet { rules: stylesheet })
        .map_err(|e| CSSParseError::InvalidResourceError(e))
}

// `rules_` (and `rules`) tries to parse input as Element or Text.
fn rules<Input>() -> impl Parser<Input, Output = Vec<Rule>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many(rule())
}

fn rule<Input>() -> impl Parser<Input, Output = Rule>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        selectors().skip(spaces()),
        char::char('{').skip(spaces()),
        declarations().skip(spaces()),
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
    sep_by(selector().skip(spaces()), char::char(',').skip(spaces()))
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
    let type_or_attribute_selector = (
        many1(letter()).skip(spaces()),
        optional((
            char::char('[').skip(spaces()),
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

    choice((universal_selector, type_or_attribute_selector))
}

fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by(declaration().skip(spaces()), char::char(';').skip(spaces()))
}

fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1(letter()).skip(spaces()),
        char::char(':').skip(spaces()),
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
