//! This module includes some implementations on Fetch.

use crate::url::{ParseError, Url};
use log::{error, info};
use num_derive::{self, FromPrimitive};
use reqwest;
use std::fs;
use std::{collections::HashMap, fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, PartialEq, FromPrimitive)]
pub enum HTTPStatus {
    OK = 200,
}

#[derive(Debug, PartialEq)]
pub enum ResponseType {
    Basic,
    CORS,
    Default,
    Error,
    Opaque,
    OpaqueRedirect,
}

impl fmt::Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseType::Basic => f.write_str("basic"),
            ResponseType::CORS => f.write_str("cors"),
            ResponseType::Default => f.write_str("default"),
            ResponseType::Error => f.write_str("error"),
            ResponseType::Opaque => f.write_str("opaque"),
            ResponseType::OpaqueRedirect => f.write_str("opaqueredirect"),
        }
    }
}

impl FromStr for ResponseType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "basic" => Ok(ResponseType::Basic),
            "cors" => Ok(ResponseType::CORS),
            "default" => Ok(ResponseType::Default),
            "error" => Ok(ResponseType::Error),
            "opaque" => Ok(ResponseType::Opaque),
            "opaqueredirect" => Ok(ResponseType::OpaqueRedirect),
            _ => Err("invalid response type"),
        }
    }
}

pub type HeaderMap = HashMap<String, String>;

/// `Request` is an interface defined at [Fetch Standard](https://fetch.spec.whatwg.org/#request-class).
/// This structure will be used both in internal processing and in JS engine.
#[derive(Debug, PartialEq)]
pub struct Request {
    pub url: String,
}

impl Request {
    pub fn new(url: String) -> Self {
        Request { url: url }
    }
}

// `Response` is an interface defined at [Fetch Standard](https://fetch.spec.whatwg.org/#response-class).
/// This structure will be used both in internal processing and in JS engine.
#[derive(Debug, PartialEq)]
pub struct Response {
    pub rtype: ResponseType,
    pub url: Url,
    pub status: HTTPStatus,
    pub headers: HeaderMap,
    pub data: Vec<u8>,
}

#[derive(Error, Debug, PartialEq)]
pub enum FetchError {
    #[error("failed to fetch because of something")]
    NetworkError { response: Option<Response> },

    #[error("failed to fetch because given url is invalid")]
    URLParseError {
        error: ParseError,
        response: Option<Response>,
    },

    #[error("failed to fetch because scheme {scheme:?} is not supported")]
    URLSchemeUnsupportedError {
        scheme: String,
        response: Option<Response>,
    },
}

// NOTE: Fetch Standard defines a way to handle requests consistently across the web platforms.
// - https://fetch.spec.whatwg.org/#fetching
//
// The name of the specification may remind you of `fetch()` function in JS,
// but it includes wider definitions (such as the behaviour of navigation requests).

pub fn fetch(request: Request) -> Result<Response, FetchError> {
    match Url::parse(request.url.as_str()) {
        Ok(u) => {
            // Err(FetchError::NetworkError { response: None })
            match u.scheme() {
                "file" => {
                    info!("[file:] local resource at {} is requested.", u.path());
                    match fs::read(u.path()) {
                        Ok(content) => Ok(Response {
                            url: u,
                            status: HTTPStatus::OK,
                            rtype: ResponseType::Basic,
                            headers: HeaderMap::new(),
                            data: content,
                        }),
                        Err(_e) => Err(FetchError::NetworkError { response: None }),
                    }
                }
                "http" | "https" => {
                    info!(
                        "[http(s):] remote resource at {} is requested.",
                        u.to_string()
                    );
                    match reqwest::blocking::get(u.to_string()).and_then(|resp| resp.bytes()) {
                        Ok(content) => Ok(Response {
                            url: u,
                            status: HTTPStatus::OK,
                            rtype: ResponseType::Basic,
                            headers: HeaderMap::new(),
                            data: content.to_vec(),
                        }),
                        Err(_e) => Err(FetchError::NetworkError { response: None }),
                    }
                }
                unsupported_scheme => {
                    // TODO (enhancement): set appropriate response
                    Err(FetchError::URLSchemeUnsupportedError {
                        scheme: unsupported_scheme.to_string(),
                        response: None,
                    })
                }
            }
        }
        Err(e) => {
            // TODO (enhancement): set appropriate response
            Err(FetchError::URLParseError {
                error: e,
                response: None,
            })
        }
    }
}
