use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError { expected: String, actual: String },

    #[error("Unknown type: {0}")]
    UnknownType(String),

    #[error("Unknown enum variant: {enum_name}::{variant}")]
    UnknownEnumVariant { enum_name: String, variant: String },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid socket address: {0}")]
    InvalidSocket(String),

    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),

    #[error("Include file not found: {0}")]
    IncludeNotFound(String),

    #[error("Duplicate key: {0}")]
    DuplicateKey(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}
