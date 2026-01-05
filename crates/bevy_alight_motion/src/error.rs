//! Error types for bevy_alight_motion.

use thiserror::Error;

/// Errors that can occur when working with AM files.
#[derive(Debug, Error)]
pub enum AmError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// XML parsing error.
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    /// ZIP error.
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Invalid file format.
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Invalid value.
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

impl From<quick_xml::DeError> for AmError {
    fn from(e: quick_xml::DeError) -> Self {
        AmError::XmlParse(e.to_string())
    }
}
