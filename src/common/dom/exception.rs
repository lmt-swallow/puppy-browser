use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DOMException {
    #[error("only one element on document allowed.")]
    InvalidDocumentElement,
}
