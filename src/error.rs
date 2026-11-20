use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid tag: {0}")]
    InvalidTag(String),

    #[error("invalid length for tag {tag}: {length}")]
    InvalidLength { tag: String, length: usize },

    #[error("duplicate tag: {0}")]
    DuplicateTag(String),

    #[error("missing required tag: {0}")]
    MissingTag(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    #[error("CRC mismatch: expected {expected}, got {actual}")]
    CrcMismatch { expected: String, actual: String },

    #[error("QR rendering failed: {0}")]
    Qr(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
