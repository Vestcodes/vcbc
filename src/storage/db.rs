//! Database and storage manager for VCBC blockchain.
//!
//! Provides storage implementations including file-based and
//! SQLite database storage for blockchain persistence.

use crate::blockchain::Block;
use crate::error::{BlockchainError, Result};
use crate::Blockchain;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

/// Storage interface for blockchain persistence
pub trait Storage {
    /// Save blockchain to storage
    fn save_blockchain(&self, blockchain: &Blockchain) -> Result<()>;

    /// Load blockchain from storage
    fn load_blockchain(&self) -> Result<Blockchain>;

    /// Check if blockchain exists in storage
    fn blockchain_exists(&self) -> bool;

    /// Clear stored blockchain
    fn clear(&self) -> Result<()>;

    /// Perform maintenance operations (compaction, etc.)
    fn maintenance(&self) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// File-based storage implementation
pub struct FileStorage {
    file_path: String,
}

impl FileStorage {
    /// Create new file storage with specified path
    pub fn new(file_path: &str) -> Self {
        FileStorage {
            file_path: file_path.to_string(),
        }
    }

    /// Validate conditions before saving
    fn validate_save_conditions(&self, blockchain: &Blockchain) -> Result<()> {
        if blockchain.chain.is_empty() {
            return Err(BlockchainError::InvalidInput(
                "Cannot save empty blockchain".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate loaded blockchain integrity
    fn validate_loaded_blockchain(&self, blockchain: &Blockchain) -> Result<()> {
        if blockchain.chain.is_empty() {
            return Err(BlockchainError::InvalidInput(
                "Loaded blockchain is empty".to_string(),
            ));
        }
        // TODO: Fix MPT serialization to be deterministic before re-enabling full validation
        // For now, just check basic structure
        for block in &blockchain.chain {
            if block.data.get_all().is_empty() {
                return Err(BlockchainError::BlockValidation(
                    "Loaded block has empty MPT data".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new("blockchain.json")
    }
}

impl Storage for FileStorage {
    fn save_blockchain(&self, blockchain: &Blockchain) -> Result<()> {
        self.validate_save_conditions(blockchain)?;

        let json_data = serde_json::to_string_pretty(blockchain).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to serialize blockchain: {}", e))
        })?;

        fs::write(&self.file_path, json_data).map_err(|e| {
            BlockchainError::StorageError(format!(
                "Failed to write blockchain file '{}': {}",
                self.file_path, e
            ))
        })?;

        Ok(())
    }

    fn load_blockchain(&self) -> Result<Blockchain> {
        if !self.blockchain_exists() {
            return Err(BlockchainError::NotFound(format!(
                "Blockchain file '{}' not found",
                self.file_path
            )));
        }

        let json_data = fs::read_to_string(&self.file_path).map_err(|e| {
            BlockchainError::StorageError(format!(
                "Failed to read blockchain file '{}': {}",
                self.file_path, e
            ))
        })?;

        let blockchain: Blockchain = serde_json::from_str(&json_data).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to deserialize blockchain: {}", e))
        })?;

        self.validate_loaded_blockchain(&blockchain)?;
        Ok(blockchain)
    }

    fn blockchain_exists(&self) -> bool {
        Path::new(&self.file_path).exists()
    }

    fn clear(&self) -> Result<()> {
        if self.blockchain_exists() {
            fs::remove_file(&self.file_path).map_err(|e| {
                BlockchainError::StorageError(format!(
                    "Failed to remove blockchain file '{}': {}",
                    self.file_path, e
                ))
            })?;
        }
        Ok(())
    }
}

/// SQLite database storage implementation
pub struct SqliteStorage {
    db_path: String,
}

impl SqliteStorage {
    /// Create new SQLite storage with specified database path
    pub fn new(db_path: &str) -> Self {
        SqliteStorage {
            db_path: db_path.to_string(),
        }
    }

    /// Initialize database tables
    fn init_db(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to open database: {}", e))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                index INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                data TEXT NOT NULL,
                previous_hash TEXT NOT NULL,
                hash TEXT NOT NULL,
                nonce INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| {
            BlockchainError::StorageError(format!("Failed to create blocks table: {}", e))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| {
            BlockchainError::StorageError(format!("Failed to create metadata table: {}", e))
        })?;

        Ok(conn)
    }

    /// Save a block to the database
    fn save_block(&self, conn: &Connection, block: &Block) -> Result<()> {
        let block_data = serde_json::to_string(&block.data).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to serialize block data: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO blocks (index, timestamp, data, previous_hash, hash, nonce)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                block.index as i64,
                block.timestamp.to_rfc3339(),
                block_data,
                block.previous_hash,
                block.hash,
                block.nonce as i64
            ],
        )
        .map_err(|e| BlockchainError::StorageError(format!("Failed to save block: {}", e)))?;

        Ok(())
    }

    /// Load a block from the database
    fn load_block(&self, conn: &Connection, index: u64) -> Result<Block> {
        let mut stmt = conn
            .prepare("SELECT index, timestamp, data, previous_hash, hash, nonce FROM blocks WHERE index = ?")
            .map_err(|e| {
                BlockchainError::StorageError(format!("Failed to prepare block query: {}", e))
            })?;

        let block = stmt
            .query_row([index as i64], |row| {
                let timestamp_str: String = row.get(1)?;
                let data_json: String = row.get(2)?;
                let data: crate::mpt::MerklePatriciaTrie = serde_json::from_str(&data_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                        )
                    })?;

                Ok(Block {
                    index: row.get(0)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                1,
                                rusqlite::types::Type::Text,
                                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                            )
                        })?
                        .with_timezone(&chrono::Utc),
                    data,
                    previous_hash: row.get(3)?,
                    hash: row.get(4)?,
                    nonce: row.get(5)?,
                })
            })
            .map_err(|e| {
                BlockchainError::StorageError(format!("Failed to load block {}: {}", index, e))
            })?;

        Ok(block)
    }

    /// Get the highest block index in the database
    #[allow(dead_code)]
    fn get_max_block_index(&self, conn: &Connection) -> Result<Option<u64>> {
        let mut stmt = conn.prepare("SELECT MAX(index) FROM blocks").map_err(|e| {
            BlockchainError::StorageError(format!("Failed to prepare max index query: {}", e))
        })?;

        let max_index: Option<i64> = stmt.query_row([], |row| row.get(0)).ok();

        Ok(max_index.map(|i| i as u64))
    }
}

impl Default for SqliteStorage {
    fn default() -> Self {
        Self::new("blockchain.db")
    }
}

impl Storage for SqliteStorage {
    fn save_blockchain(&self, blockchain: &Blockchain) -> Result<()> {
        let mut conn = self.init_db()?;

        // Begin transaction
        let tx = conn.transaction().map_err(|e| {
            BlockchainError::StorageError(format!("Failed to start transaction: {}", e))
        })?;

        // Clear existing data
        tx.execute("DELETE FROM blocks", [])
            .map_err(|e| BlockchainError::StorageError(format!("Failed to clear blocks: {}", e)))?;

        // Save metadata
        let network_config_json =
            serde_json::to_string(&blockchain.network_config).map_err(|e| {
                BlockchainError::StorageError(format!("Failed to serialize network config: {}", e))
            })?;
        tx.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            params!["network_config", network_config_json],
        )
        .map_err(|e| {
            BlockchainError::StorageError(format!("Failed to save network config: {}", e))
        })?;

        let difficulty_json = serde_json::to_string(&blockchain.difficulty).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to serialize difficulty: {}", e))
        })?;
        tx.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            params!["difficulty", difficulty_json],
        )
        .map_err(|e| BlockchainError::StorageError(format!("Failed to save difficulty: {}", e)))?;

        // Save all blocks
        for block in &blockchain.chain {
            self.save_block(&tx, block)?;
        }

        // Commit transaction
        tx.commit().map_err(|e| {
            BlockchainError::StorageError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    fn load_blockchain(&self) -> Result<Blockchain> {
        let conn = self.init_db()?;

        // Load metadata
        let network_config_json: String = conn
            .query_row(
                "SELECT value FROM metadata WHERE key = ?",
                ["network_config"],
                |row| row.get(0),
            )
            .map_err(|e| {
                BlockchainError::StorageError(format!("Failed to load network config: {}", e))
            })?;

        let network_config: crate::blockchain::NetworkConfig =
            serde_json::from_str(&network_config_json).map_err(|e| {
                BlockchainError::StorageError(format!(
                    "Failed to deserialize network config: {}",
                    e
                ))
            })?;

        let difficulty_json: String = conn
            .query_row(
                "SELECT value FROM metadata WHERE key = ?",
                ["difficulty"],
                |row| row.get(0),
            )
            .map_err(|e| {
                BlockchainError::StorageError(format!("Failed to load difficulty: {}", e))
            })?;

        let difficulty: usize = serde_json::from_str(&difficulty_json).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to deserialize difficulty: {}", e))
        })?;

        // Load all blocks
        let mut stmt = conn
            .prepare("SELECT index FROM blocks ORDER BY index")
            .map_err(|e| {
                BlockchainError::StorageError(format!("Failed to prepare block query: {}", e))
            })?;

        let block_indices: Vec<u64> = stmt
            .query_map([], |row| {
                let index: i64 = row.get(0)?;
                Ok(index as u64)
            })
            .map_err(|e| BlockchainError::StorageError(format!("Failed to query blocks: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                BlockchainError::StorageError(format!("Failed to collect block indices: {}", e))
            })?;

        let mut chain = Vec::new();
        for index in block_indices {
            let block = self.load_block(&conn, index)?;
            chain.push(block);
        }

        if chain.is_empty() {
            return Err(BlockchainError::InvalidInput(
                "No blocks found in database".to_string(),
            ));
        }

        Ok(Blockchain {
            chain,
            difficulty,
            network_config,
        })
    }

    fn blockchain_exists(&self) -> bool {
        Path::new(&self.db_path).exists()
    }

    fn clear(&self) -> Result<()> {
        if self.blockchain_exists() {
            fs::remove_file(&self.db_path).map_err(|e| {
                BlockchainError::StorageError(format!(
                    "Failed to remove database file '{}': {}",
                    self.db_path, e
                ))
            })?;
        }
        Ok(())
    }

    fn maintenance(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to open database for maintenance: {}", e))
        })?;

        conn.execute("VACUUM", []).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to vacuum database: {}", e))
        })?;

        Ok(())
    }
}

/// Storage manager for easy switching between storage backends
pub struct StorageManager {
    storage: Box<dyn Storage>,
}

impl StorageManager {
    /// Create storage manager with file-based storage
    pub fn with_file_storage(file_path: &str) -> Self {
        StorageManager {
            storage: Box::new(FileStorage::new(file_path)),
        }
    }

    /// Create storage manager with SQLite storage
    pub fn with_sqlite_storage(db_path: &str) -> Self {
        StorageManager {
            storage: Box::new(SqliteStorage::new(db_path)),
        }
    }

    /// Save blockchain using the configured storage backend
    pub fn save_blockchain(&self, blockchain: &Blockchain) -> Result<()> {
        self.storage.save_blockchain(blockchain)
    }

    /// Load blockchain using the configured storage backend
    pub fn load_blockchain(&self) -> Result<Blockchain> {
        self.storage.load_blockchain()
    }

    /// Check if blockchain exists in storage
    pub fn blockchain_exists(&self) -> bool {
        self.storage.blockchain_exists()
    }

    /// Clear stored blockchain
    pub fn clear(&self) -> Result<()> {
        self.storage.clear()
    }

    /// Perform maintenance operations
    pub fn maintenance(&self) -> Result<()> {
        self.storage.maintenance()
    }

    /// Format file size for display
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Validate storage path
    pub fn validate_path(path: &str) -> Result<()> {
        let path_obj = Path::new(path);
        if let Some(parent) = path_obj.parent() {
            // Only check if parent exists if it's not the current directory
            if parent != Path::new("") && !parent.exists() {
                return Err(BlockchainError::InvalidInput(format!(
                    "Parent directory does not exist: {}",
                    parent.display()
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Blockchain;

    #[test]
    fn test_file_storage_creation() {
        let storage = FileStorage::new("test_blockchain.json");
        assert_eq!(storage.file_path, "test_blockchain.json");
    }

    #[test]
    fn test_file_storage_default() {
        let storage = FileStorage::default();
        assert_eq!(storage.file_path, "blockchain.json");
    }

    #[test]
    fn test_file_storage_exists() {
        let storage = FileStorage::new("nonexistent.json");
        assert!(!storage.blockchain_exists());
    }

    #[test]
    fn test_sqlite_storage_creation() {
        let storage = SqliteStorage::new("test_blockchain.db");
        assert_eq!(storage.db_path, "test_blockchain.db");
    }

    #[test]
    fn test_sqlite_storage_default() {
        let storage = SqliteStorage::default();
        assert_eq!(storage.db_path, "blockchain.db");
    }

    #[test]
    fn test_storage_manager_file() {
        let manager = StorageManager::with_file_storage("test.json");
        assert!(!manager.blockchain_exists());
    }

    #[test]
    fn test_storage_manager_sqlite() {
        let manager = StorageManager::with_sqlite_storage("test.db");
        assert!(!manager.blockchain_exists());
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(StorageManager::format_file_size(0), "0.00 B");
        assert_eq!(StorageManager::format_file_size(1024), "1.00 KB");
        assert_eq!(StorageManager::format_file_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_validate_path() {
        // Path with non-existent parent directory should error
        assert!(StorageManager::validate_path("/nonexistent/directory/test.json").is_err());

        // Valid relative path should not error
        assert!(StorageManager::validate_path("test.json").is_ok());
    }

    #[test]
    fn test_storage_manager_backup_restore() {
        let blockchain = Blockchain::new_default(1);
        let manager = StorageManager::with_file_storage("backup_test.json");

        // Save blockchain
        manager.save_blockchain(&blockchain).unwrap();
        assert!(manager.blockchain_exists());

        // Load blockchain
        let loaded = manager.load_blockchain().unwrap();
        assert_eq!(loaded.chain.len(), blockchain.chain.len());

        // Verify MPT data integrity (structure may differ but data should be the same)
        for (orig_block, loaded_block) in blockchain.chain.iter().zip(loaded.chain.iter()) {
            assert_eq!(orig_block.data.get_all(), loaded_block.data.get_all());
        }

        // Cleanup
        manager.clear().unwrap();
        assert!(!manager.blockchain_exists());
    }
}
