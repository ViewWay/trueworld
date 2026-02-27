// Error types for TrueWorld
//
// This module defines the core error types used throughout the TrueWorld project.

use std::io;
use thiserror::Error;

/// Core error type for TrueWorld
///
/// This enum represents all possible errors that can occur in the TrueWorld system.
/// Each variant provides context about the specific error condition.
#[derive(Error, Debug)]
pub enum Error {
    /// Network-related errors (connection issues, timeouts, etc.)
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization/deserialization errors (JSON, binary formats, etc.)
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// I/O errors (file operations, streams, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// AI model-related errors (loading, inference, etc.)
    #[error("AI model error: {0}")]
    AiModel(String),

    /// Invalid input provided by user or from another component
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Resource not found (entity, file, data, etc.)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission or authorization denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Operation timed out
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Unknown or unexpected errors
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// Generic error wrapper for external error types
    #[error("{0}")]
    Boxed(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Result type alias for convenience
///
/// Usage:
/// ```rust
/// use trueworld_core::Result;
///
/// fn do_something() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        // Test Network error display
        let err = Error::Network("Connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: Connection refused");

        // Test Serialization error display
        let err = Error::Serialization("Failed to parse JSON".to_string());
        assert_eq!(err.to_string(), "Serialization error: Failed to parse JSON");

        // Test Io error display
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        assert_eq!(err.to_string(), "I/O error: file not found");

        // Test AiModel error display
        let err = Error::AiModel("Model load failed".to_string());
        assert_eq!(err.to_string(), "AI model error: Model load failed");

        // Test InvalidInput error display
        let err = Error::InvalidInput("Negative value not allowed".to_string());
        assert_eq!(err.to_string(), "Invalid input: Negative value not allowed");

        // Test NotFound error display
        let err = Error::NotFound("Player not found".to_string());
        assert_eq!(err.to_string(), "Not found: Player not found");

        // Test PermissionDenied error display
        let err = Error::PermissionDenied("Access denied".to_string());
        assert_eq!(err.to_string(), "Permission denied: Access denied");

        // Test Timeout error display
        let err = Error::Timeout("Request timed out after 30s".to_string());
        assert_eq!(err.to_string(), "Timeout: Request timed out after 30s");

        // Test Unknown error display
        let err = Error::Unknown("Unexpected condition".to_string());
        assert_eq!(err.to_string(), "Unknown error: Unexpected condition");
    }

    #[test]
    fn test_error_debug() {
        let err = Error::Network("Connection refused".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Network"));
        assert!(debug_str.contains("Connection refused"));
    }

    #[test]
    fn test_result_alias() {
        // Test Ok variant
        let result: Result<String> = Ok("success".to_string());
        assert!(result.is_ok());

        // Test Err variant
        let result: Result<String> = Err(Error::NotFound("Item".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().contains("access denied"));
    }
}
