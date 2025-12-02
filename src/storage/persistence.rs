//! Persistence layer utilities for VCBC blockchain.
//!
//! Provides serialization, deserialization, and persistence
//! utilities for blockchain data and configuration.

use crate::error::{BlockchainError, Result};
use std::fs;
use std::path::Path;

/// Persistence layer for blockchain data
pub struct PersistenceLayer;

impl PersistenceLayer {
    /// Save data to JSON file with atomic write
    pub fn save_json_atomic<T: serde::Serialize>(data: &T, file_path: &str) -> Result<()> {
        let json_data = serde_json::to_string_pretty(data).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to serialize data: {}", e))
        })?;

        // Write to temporary file first
        let temp_path = format!("{}.tmp", file_path);
        fs::write(&temp_path, &json_data).map_err(|e| {
            BlockchainError::StorageError(format!(
                "Failed to write temp file '{}': {}",
                temp_path, e
            ))
        })?;

        // Atomic move to final location
        fs::rename(&temp_path, file_path).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to rename file '{}': {}", file_path, e))
        })?;

        Ok(())
    }

    /// Load data from JSON file
    pub fn load_json<T: serde::de::DeserializeOwned>(file_path: &str) -> Result<T> {
        if !Path::new(file_path).exists() {
            return Err(BlockchainError::NotFound(format!(
                "File '{}' not found",
                file_path
            )));
        }

        let json_data = fs::read_to_string(file_path).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to read file '{}': {}", file_path, e))
        })?;

        serde_json::from_str(&json_data).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to deserialize data: {}", e))
        })
    }

    /// Check if file exists and is readable
    pub fn file_exists_and_readable(file_path: &str) -> bool {
        let path = Path::new(file_path);
        path.exists() && path.is_file()
    }

    /// Create backup of existing file
    pub fn create_backup(file_path: &str) -> Result<String> {
        if !Self::file_exists_and_readable(file_path) {
            return Err(BlockchainError::NotFound(format!(
                "File '{}' not found or not readable",
                file_path
            )));
        }

        let backup_path = format!("{}.backup", file_path);
        fs::copy(file_path, &backup_path).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to create backup: {}", e))
        })?;

        Ok(backup_path)
    }

    /// Restore from backup file
    pub fn restore_from_backup(file_path: &str) -> Result<()> {
        let backup_path = format!("{}.backup", file_path);

        if !Self::file_exists_and_readable(&backup_path) {
            return Err(BlockchainError::NotFound(format!(
                "Backup file '{}' not found",
                backup_path
            )));
        }

        fs::copy(&backup_path, file_path).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to restore from backup: {}", e))
        })?;

        Ok(())
    }

    /// Validate JSON file structure
    pub fn validate_json_file<T: serde::de::DeserializeOwned>(file_path: &str) -> Result<()> {
        // Try to load and deserialize the file
        let _: T = Self::load_json(file_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_save_and_load_json() {
        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let file_path = "test_data.json";

        // Save data
        PersistenceLayer::save_json_atomic(&test_data, file_path).unwrap();

        // Load data
        let loaded_data: TestData = PersistenceLayer::load_json(file_path).unwrap();

        assert_eq!(test_data, loaded_data);

        // Cleanup
        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_file_exists_and_readable() {
        let file_path = "nonexistent.json";
        assert!(!PersistenceLayer::file_exists_and_readable(file_path));

        // Create a test file
        fs::write(file_path, "test").unwrap();
        assert!(PersistenceLayer::file_exists_and_readable(file_path));

        // Cleanup
        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_create_backup() {
        let file_path = "test_backup.json";
        let test_content = "test content";

        // Create test file
        fs::write(file_path, test_content).unwrap();

        // Create backup
        let backup_path = PersistenceLayer::create_backup(file_path).unwrap();
        assert_eq!(backup_path, "test_backup.json.backup");

        // Verify backup content
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, test_content);

        // Cleanup
        fs::remove_file(file_path).unwrap();
        fs::remove_file(&backup_path).unwrap();
    }

    #[test]
    fn test_restore_from_backup() {
        let file_path = "test_restore.json";
        let backup_path = "test_restore.json.backup";
        let test_content = "backup content";

        // Create backup file
        fs::write(backup_path, test_content).unwrap();

        // Restore from backup
        PersistenceLayer::restore_from_backup(file_path).unwrap();

        // Verify restored content
        let restored_content = fs::read_to_string(file_path).unwrap();
        assert_eq!(restored_content, test_content);

        // Cleanup
        fs::remove_file(file_path).unwrap();
        fs::remove_file(backup_path).unwrap();
    }

    #[test]
    fn test_validate_json_file() {
        let file_path = "test_validate.json";
        let test_data = TestData {
            name: "validate".to_string(),
            value: 123,
        };

        // Save valid JSON
        PersistenceLayer::save_json_atomic(&test_data, file_path).unwrap();

        // Validate should succeed
        assert!(PersistenceLayer::validate_json_file::<TestData>(file_path).is_ok());

        // Create invalid JSON
        fs::write(file_path, "invalid json").unwrap();

        // Validate should fail
        assert!(PersistenceLayer::validate_json_file::<TestData>(file_path).is_err());

        // Cleanup
        fs::remove_file(file_path).unwrap();
    }
}
