use thiserror::Error;

#[derive(Error, Debug)]
pub enum DOMException {
    #[error("only one element on document allowed.")]
    InvalidDocumentElement,
}
