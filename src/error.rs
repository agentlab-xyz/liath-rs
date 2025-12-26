//! Error types for Liath database operations

use thiserror::Error;

/// Main error type for Liath operations
#[derive(Error, Debug)]
pub enum LiathError {
    /// Namespace not found
    #[error("Namespace '{0}' not found")]
    NamespaceNotFound(String),

    /// Namespace already exists
    #[error("Namespace '{0}' already exists")]
    NamespaceExists(String),

    /// Key not found in namespace
    #[error("Key not found in namespace '{0}'")]
    KeyNotFound(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] anyhow::Error),

    /// Unauthorized operation
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Embedding generation error
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Vector search error
    #[error("Vector search error: {0}")]
    VectorSearch(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Agent error
    #[error("Agent error: {0}")]
    Agent(String),

    /// Conversation not found
    #[error("Conversation '{0}' not found")]
    ConversationNotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<serde_json::Error> for LiathError {
    fn from(err: serde_json::Error) -> Self {
        LiathError::Serialization(err.to_string())
    }
}

/// Result type alias for Liath operations
pub type LiathResult<T> = Result<T, LiathError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = LiathError::NamespaceNotFound("test".to_string());
        assert_eq!(err.to_string(), "Namespace 'test' not found");

        let err = LiathError::Unauthorized("admin access required".to_string());
        assert_eq!(err.to_string(), "Unauthorized: admin access required");
    }
}
