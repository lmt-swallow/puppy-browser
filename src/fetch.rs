use num_derive::{self, FromPrimitive};
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

#[derive(Debug, PartialEq)]
pub struct Request {
    pub url: String,
}

impl Request {
    pub fn new(url: String) -> Self {
        Request {
            url: url
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub rtype: ResponseType,
    pub url: String,
    pub status: HTTPStatus,
    pub headers: HeaderMap,
    pub data: Vec<u8>,
}

#[derive(Error, Debug, PartialEq)]
pub enum FetchError {
    #[error("failed to fetch")]
    NetworkError {
        // error: Box<Error>,
        response: Response,
    },
}

// NOTE: Fetch Standard defines a way to handle requests consistently across the web platforms.
// - https://fetch.spec.whatwg.org/#fetching
pub fn fetch(request: Request) -> Result<Response, FetchError> {
    // TODO
    Ok(Response {
        url: request.url.to_string(),
        status: HTTPStatus::OK,
        rtype: ResponseType::Basic,
        headers: HeaderMap::new(),
        data: "<p>Hello World</p><p>Hello World2</p>".as_bytes().to_vec(),
    })
}
