//! This module includes some implementations on URL concepts.

/// URL Standard: https://url.spec.whatwg.org/
/// Here we use `rust-url` crate instead of implementing by ourselves.
pub use url::{ParseError, Url};
