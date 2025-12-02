//! Custom error types for the blockchain implementation.
//!
//! Provides specific error variants for different failure modes,
//! enabling better error handling and debugging.

use std::fmt;

/// Custom error type for blockchain operations
#[derive(Debug, Clone, PartialEq)]
pub enum BlockchainError {
    /// Errors related to block validation
    BlockValidation(String),

    /// Errors related to MPT operations
    MptOperation(String),

    /// Errors related to proof generation/verification
    ProofError(String),

    /// Errors related to network communication
    NetworkError(String),

    /// Errors related to storage operations
    StorageError(String),

    /// Errors related to node operations
    NodeError(String),

    /// Invalid input parameters
    InvalidInput(String),

    /// Resource not found
    NotFound(String),

    /// Network mismatch between nodes
    NetworkMismatch { expected: String, received: String },

    /// Genesis block hash mismatch
    GenesisMismatch { expected: String, received: String },

    /// Protocol version mismatch
    ProtocolMismatch { expected: String, received: String },

    /// Internal consistency errors
    InternalError(String),
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockchainError::BlockValidation(msg) => write!(f, "Block validation error: {}", msg),
            BlockchainError::MptOperation(msg) => write!(f, "MPT operation error: {}", msg),
            BlockchainError::ProofError(msg) => write!(f, "Proof error: {}", msg),
            BlockchainError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            BlockchainError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            BlockchainError::NodeError(msg) => write!(f, "Node error: {}", msg),
            BlockchainError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            BlockchainError::NotFound(msg) => write!(f, "Not found: {}", msg),
            BlockchainError::NetworkMismatch { expected, received } => {
                write!(
                    f,
                    "Network mismatch: expected '{}', received '{}'",
                    expected, received
                )
            }
            BlockchainError::GenesisMismatch { expected, received } => {
                write!(
                    f,
                    "Genesis mismatch: expected '{}', received '{}'",
                    expected, received
                )
            }
            BlockchainError::ProtocolMismatch { expected, received } => {
                write!(
                    f,
                    "Protocol mismatch: expected '{}', received '{}'",
                    expected, received
                )
            }
            BlockchainError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for BlockchainError {}

/// Type alias for Results using BlockchainError
pub type Result<T> = std::result::Result<T, BlockchainError>;

// Conversion from common error types
impl From<std::io::Error> for BlockchainError {
    fn from(err: std::io::Error) -> Self {
        BlockchainError::StorageError(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for BlockchainError {
    fn from(err: serde_json::Error) -> Self {
        BlockchainError::StorageError(format!("Serialization error: {}", err))
    }
}

impl From<rusqlite::Error> for BlockchainError {
    fn from(err: rusqlite::Error) -> Self {
        BlockchainError::StorageError(format!("Database error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_error_display() {
        let error = BlockchainError::BlockValidation("Invalid hash".to_string());
        assert!(format!("{}", error).contains("Block validation error"));
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let blockchain_error: BlockchainError = io_error.into();
        match blockchain_error {
            BlockchainError::StorageError(_) => {}
            _ => panic!("Expected StorageError"),
        }
    }

    #[test]
    fn test_all_error_variants_display() {
        // Test all error variants have proper display formatting
        assert!(
            format!("{}", BlockchainError::BlockValidation("test".to_string()))
                .contains("Block validation error")
        );
        assert!(
            format!("{}", BlockchainError::MptOperation("test".to_string()))
                .contains("MPT operation error")
        );
        assert!(
            format!("{}", BlockchainError::ProofError("test".to_string())).contains("Proof error")
        );
        assert!(
            format!("{}", BlockchainError::NetworkError("test".to_string()))
                .contains("Network error")
        );
        assert!(
            format!("{}", BlockchainError::StorageError("test".to_string()))
                .contains("Storage error")
        );
        assert!(
            format!("{}", BlockchainError::NodeError("test".to_string())).contains("Node error")
        );
        assert!(
            format!("{}", BlockchainError::InvalidInput("test".to_string()))
                .contains("Invalid input")
        );
        assert!(format!("{}", BlockchainError::NotFound("test".to_string())).contains("Not found"));
        assert!(
            format!("{}", BlockchainError::InternalError("test".to_string()))
                .contains("Internal error")
        );
    }

    #[test]
    fn test_network_mismatch_display() {
        let error = BlockchainError::NetworkMismatch {
            expected: "vc-mainnet".to_string(),
            received: "vc-testnet".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Network mismatch"));
        assert!(display.contains("vc-mainnet"));
        assert!(display.contains("vc-testnet"));
    }

    #[test]
    fn test_genesis_mismatch_display() {
        let error = BlockchainError::GenesisMismatch {
            expected: "hash123".to_string(),
            received: "hash456".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Genesis mismatch"));
        assert!(display.contains("hash123"));
        assert!(display.contains("hash456"));
    }

    #[test]
    fn test_protocol_mismatch_display() {
        let error = BlockchainError::ProtocolMismatch {
            expected: "v1.0".to_string(),
            received: "v2.0".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Protocol mismatch"));
        assert!(display.contains("v1.0"));
        assert!(display.contains("v2.0"));
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = BlockchainError::InvalidInput("test message".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidInput"));
        assert!(debug_str.contains("test message"));
    }

    #[test]
    fn test_error_clone() {
        let error = BlockchainError::BlockValidation("test".to_string());
        let cloned = error.clone();
        assert!(matches!(cloned, BlockchainError::BlockValidation(_)));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(ErrorKind::PermissionDenied, "permission denied");
        let blockchain_error: BlockchainError = io_error.into();
        match blockchain_error {
            BlockchainError::StorageError(msg) => {
                assert!(msg.contains("IO error"));
                assert!(msg.contains("permission denied"));
            }
            _ => panic!("Expected StorageError"),
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let blockchain_error: BlockchainError = json_error.into();
        match blockchain_error {
            BlockchainError::StorageError(msg) => {
                assert!(msg.contains("Serialization error"));
            }
            _ => panic!("Expected StorageError"),
        }
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = BlockchainError::InvalidInput("test".to_string());
        // Test that it implements std::error::Error
        let _error_trait: &dyn std::error::Error = &error;
        // Test that Display works
        let _display = format!("{}", error);
    }

    #[test]
    fn test_result_type_alias() {
        let result: Result<i32> = Ok(42);
        assert_eq!(result, Ok(42));

        let error_result: Result<i32> = Err(BlockchainError::InvalidInput("test".to_string()));
        assert!(error_result.is_err());
    }
}
