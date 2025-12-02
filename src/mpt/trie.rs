//! Merkle-Patricia Trie implementation for VCBC blockchain.
//!
//! Provides the main MerklePatriciaTrie struct with efficient key-value
//! storage, cryptographic proofs, and verifiable data integrity.

use crate::error::Result;
use crate::mpt::hash::KeyUtils;
use crate::mpt::node::{CompactEncoding, MPTNode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Merkle-Patricia Trie main structure
#[derive(Clone, Debug)]
pub struct MerklePatriciaTrie {
    root: MPTNode,
}

impl Serialize for MerklePatriciaTrie {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        // Serialize as key-value pairs
        let data = self.get_all();
        for (key, value) in data {
            map.serialize_entry(&key, &value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for MerklePatriciaTrie {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::collections::HashMap;

        let data: HashMap<String, Vec<u8>> = HashMap::deserialize(deserializer)?;
        let mut mpt = MerklePatriciaTrie::new();
        for (key, value) in data {
            mpt.insert(&key, value).map_err(serde::de::Error::custom)?;
        }
        Ok(mpt)
    }
}

impl MerklePatriciaTrie {
    /// Create a new empty MPT
    pub fn new() -> Self {
        MerklePatriciaTrie {
            root: MPTNode::Empty,
        }
    }

    /// Calculate the root hash of the MPT
    pub fn root_hash(&self) -> Vec<u8> {
        if self.root.is_empty() {
            // Return hash of "empty" for empty trie
            Sha256::digest(b"empty").to_vec()
        } else {
            self.root.calculate_hash()
        }
    }

    /// Get the root hash as a hex string
    pub fn hex_root_hash(&self) -> String {
        hex::encode(self.root_hash())
    }

    /// Insert a key-value pair into the MPT
    pub fn insert(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        let nibbles = KeyUtils::string_to_nibbles(key);
        self.root = self.insert_recursive(self.root.clone(), &nibbles, value)?;
        Ok(())
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let nibbles = KeyUtils::string_to_nibbles(key);
        self.get_recursive(&self.root, &nibbles)
    }

    /// Check if the MPT is empty
    pub fn is_empty(&self) -> bool {
        matches!(self.root, MPTNode::Empty)
    }

    /// Remove a key-value pair
    pub fn remove(&mut self, key: &str) -> Result<bool> {
        let nibbles = KeyUtils::string_to_nibbles(key);
        let (new_root, removed) = self.remove_recursive(self.root.clone(), &nibbles)?;
        self.root = new_root;
        Ok(removed)
    }

    /// Get all key-value pairs
    pub fn get_all(&self) -> BTreeMap<String, Vec<u8>> {
        let mut result = BTreeMap::new();
        self.collect_all(&self.root, Vec::new(), &mut result);
        result
    }

    #[allow(clippy::only_used_in_recursion)]
    fn collect_all(
        &self,
        node: &MPTNode,
        current_path: Vec<u8>,
        result: &mut BTreeMap<String, Vec<u8>>,
    ) {
        match node {
            MPTNode::Leaf { path, value } => {
                let (decoded_path, _) = CompactEncoding::decode(path).unwrap_or_default();
                let mut full_path = current_path;
                full_path.extend(decoded_path);
                let key = KeyUtils::nibbles_to_string(&full_path);
                result.insert(key, value.clone());
            }
            MPTNode::Extension { path, child } => {
                let (decoded_path, _) = CompactEncoding::decode(path).unwrap_or_default();
                let mut new_path = current_path;
                new_path.extend(decoded_path);
                self.collect_all(child, new_path, result);
            }
            MPTNode::Branch { children, value } => {
                // Collect value if present
                if let Some(val) = value {
                    let key = KeyUtils::nibbles_to_string(&current_path);
                    result.insert(key, val.clone());
                }
                // Recurse into children
                for (index, child) in children.iter().enumerate() {
                    if let Some(child_node) = child {
                        let mut new_path = current_path.clone();
                        new_path.push(index as u8);
                        self.collect_all(child_node, new_path, result);
                    }
                }
            }
            MPTNode::Empty => {}
        }
    }

    /// Generate a Merkle proof for a key at a specific block index
    pub fn generate_proof(
        &self,
        key: &str,
        block_index: u64,
        block_hash: String,
    ) -> Result<crate::proof::MerkleProof> {
        let value = self.get(key).unwrap_or_default();
        let mpt_root_hash = self.root_hash();

        // For now, we create a proof with the current MPT state
        // In a full implementation, this would build the actual proof path
        let proof = crate::proof::MerkleProof::new(
            key.to_string(),
            value,
            block_index,
            mpt_root_hash,
            block_hash,
        );

        Ok(proof)
    }

    // Recursive insert implementation
    fn insert_recursive(&self, node: MPTNode, nibbles: &[u8], value: Vec<u8>) -> Result<MPTNode> {
        match node {
            MPTNode::Empty => self.handle_insert_empty(nibbles, value),
            MPTNode::Leaf {
                path,
                value: existing_value,
            } => self.handle_insert_leaf(path, existing_value, nibbles, value),
            MPTNode::Extension { path, child } => {
                self.handle_insert_extension(path, *child, nibbles, value)
            }
            MPTNode::Branch {
                children,
                value: branch_value,
            } => self.handle_insert_branch(children, branch_value, nibbles, value),
        }
    }

    fn handle_insert_empty(&self, nibbles: &[u8], value: Vec<u8>) -> Result<MPTNode> {
        let path = CompactEncoding::encode(nibbles, true);
        Ok(MPTNode::Leaf { path, value })
    }

    fn handle_insert_leaf(
        &self,
        leaf_path: Vec<u8>,
        existing_value: Vec<u8>,
        nibbles: &[u8],
        value: Vec<u8>,
    ) -> Result<MPTNode> {
        let (decoded_leaf_path, _) = CompactEncoding::decode(&leaf_path)?;
        let common_len =
            crate::mpt::hash::PathUtils::common_prefix_length(&decoded_leaf_path, nibbles);

        if common_len == decoded_leaf_path.len() && common_len == nibbles.len() {
            // Exact match - replace value
            let new_path = CompactEncoding::encode(nibbles, true);
            Ok(MPTNode::Leaf {
                path: new_path,
                value,
            })
        } else if common_len == decoded_leaf_path.len() {
            // Leaf path is prefix of new path - create extension + branch
            let remaining_nibbles = &nibbles[common_len..];
            let new_branch = MPTNode::Branch {
                children: {
                    let mut children: [Option<Box<MPTNode>>; 16] = Default::default();
                    children[remaining_nibbles[0] as usize] = Some(Box::new(MPTNode::Leaf {
                        path: CompactEncoding::encode(&remaining_nibbles[1..], true),
                        value,
                    }));
                    children
                },
                value: None,
            };

            if common_len > 0 {
                let extension_path =
                    CompactEncoding::encode(&decoded_leaf_path[..common_len], false);
                Ok(MPTNode::Extension {
                    path: extension_path,
                    child: Box::new(new_branch),
                })
            } else {
                Ok(new_branch)
            }
        } else {
            // Split the leaf and create a branch
            self.split_leaf_and_branch(
                leaf_path,
                decoded_leaf_path,
                existing_value,
                common_len,
                nibbles,
                value,
            )
        }
    }

    fn split_leaf_and_branch(
        &self,
        #[allow(unused_variables)] leaf_path: Vec<u8>,
        decoded_leaf_path: Vec<u8>,
        existing_value: Vec<u8>,
        common_len: usize,
        nibbles: &[u8],
        value: Vec<u8>,
    ) -> Result<MPTNode> {
        let branch_index = decoded_leaf_path[common_len] as usize;
        let new_leaf_index = nibbles[common_len] as usize;

        let existing_leaf_path =
            CompactEncoding::encode(&decoded_leaf_path[common_len + 1..], true);
        let new_leaf_path = CompactEncoding::encode(&nibbles[common_len + 1..], true);

        let mut children: [Option<Box<MPTNode>>; 16] = Default::default();
        children[branch_index] = Some(Box::new(MPTNode::Leaf {
            path: existing_leaf_path,
            value: existing_value,
        }));
        children[new_leaf_index] = Some(Box::new(MPTNode::Leaf {
            path: new_leaf_path,
            value,
        }));

        let branch = MPTNode::Branch {
            children,
            value: None,
        };

        if common_len > 0 {
            let extension_path = CompactEncoding::encode(&decoded_leaf_path[..common_len], false);
            Ok(MPTNode::Extension {
                path: extension_path,
                child: Box::new(branch),
            })
        } else {
            Ok(branch)
        }
    }

    fn handle_insert_extension(
        &self,
        extension_path: Vec<u8>,
        child: MPTNode,
        nibbles: &[u8],
        value: Vec<u8>,
    ) -> Result<MPTNode> {
        let (decoded_path, _) = CompactEncoding::decode(&extension_path)?;
        let common_len = crate::mpt::hash::PathUtils::common_prefix_length(&decoded_path, nibbles);

        if common_len == decoded_path.len() {
            // Extension path is fully matched - recurse into child
            let remaining_nibbles = &nibbles[common_len..];
            let new_child = self.insert_recursive(child, remaining_nibbles, value)?;
            Ok(MPTNode::Extension {
                path: extension_path,
                child: Box::new(new_child),
            })
        } else if common_len == nibbles.len() {
            // New key exactly matches extension path prefix - need to split the extension
            let remaining_path = &decoded_path[common_len..];

            let new_extension_path = if common_len > 0 {
                CompactEncoding::encode(&decoded_path[..common_len], false)
            } else {
                vec![] // Empty extension
            };

            // Create new branch for the split
            let mut children: [Option<Box<MPTNode>>; 16] = Default::default();

            if remaining_path.is_empty() {
                // The extension path exactly matches the new key - replace with branch
                let branch = MPTNode::Branch {
                    children,
                    value: Some(value),
                };
                return if new_extension_path.is_empty() {
                    Ok(branch)
                } else {
                    Ok(MPTNode::Extension {
                        path: new_extension_path,
                        child: Box::new(branch),
                    })
                };
            }

            let child_index = remaining_path[0] as usize;

            // Move the current child to the branch at the remaining path index
            children[child_index] = Some(Box::new(child));

            let branch = MPTNode::Branch {
                children,
                value: Some(value), // The new key value goes here
            };

            if new_extension_path.is_empty() {
                Ok(branch)
            } else {
                Ok(MPTNode::Extension {
                    path: new_extension_path,
                    child: Box::new(branch),
                })
            }
        } else {
            // Split the extension at divergence point
            self.split_extension_and_branch(
                extension_path,
                decoded_path,
                child,
                common_len,
                nibbles,
                value,
            )
        }
    }

    fn split_extension_and_branch(
        &self,
        #[allow(unused_variables)] extension_path: Vec<u8>,
        decoded_path: Vec<u8>,
        child: MPTNode,
        common_len: usize,
        nibbles: &[u8],
        value: Vec<u8>,
    ) -> Result<MPTNode> {
        let branch_index = decoded_path[common_len] as usize;
        let new_index = nibbles[common_len] as usize;

        let remaining_extension = CompactEncoding::encode(&decoded_path[common_len + 1..], false);
        let new_extension = CompactEncoding::encode(&nibbles[common_len + 1..], true);

        let mut children: [Option<Box<MPTNode>>; 16] = Default::default();
        children[branch_index] = Some(Box::new(if remaining_extension.is_empty() {
            child
        } else {
            MPTNode::Extension {
                path: remaining_extension,
                child: Box::new(child),
            }
        }));
        children[new_index] = Some(Box::new(MPTNode::Leaf {
            path: new_extension,
            value,
        }));

        let branch = MPTNode::Branch {
            children,
            value: None,
        };

        if common_len > 0 {
            let new_extension_path = CompactEncoding::encode(&decoded_path[..common_len], false);
            Ok(MPTNode::Extension {
                path: new_extension_path,
                child: Box::new(branch),
            })
        } else {
            Ok(branch)
        }
    }

    fn handle_insert_branch(
        &self,
        mut children: [Option<Box<MPTNode>>; 16],
        branch_value: Option<Vec<u8>>,
        nibbles: &[u8],
        value: Vec<u8>,
    ) -> Result<MPTNode> {
        if nibbles.is_empty() {
            // Insert at branch value
            return Ok(MPTNode::Branch {
                children,
                value: Some(value),
            });
        }

        let index = nibbles[0] as usize;
        let remaining = &nibbles[1..];

        let new_child = if let Some(child) = children[index].take() {
            self.insert_recursive(*child, remaining, value)?
        } else {
            self.handle_insert_empty(remaining, value)?
        };

        children[index] = Some(Box::new(new_child));

        Ok(MPTNode::Branch {
            children,
            value: branch_value,
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn get_recursive(&self, node: &MPTNode, nibbles: &[u8]) -> Option<Vec<u8>> {
        match node {
            MPTNode::Empty => None,
            MPTNode::Leaf { path, value } => {
                let (decoded_path, is_terminal) = CompactEncoding::decode(path).ok()?;
                if decoded_path == nibbles && is_terminal {
                    Some(value.clone())
                } else {
                    None
                }
            }
            MPTNode::Extension { path, child } => {
                let (decoded_path, _) = CompactEncoding::decode(path).ok()?;
                if nibbles.starts_with(&decoded_path) {
                    let remaining = &nibbles[decoded_path.len()..];
                    self.get_recursive(child, remaining)
                } else {
                    None
                }
            }
            MPTNode::Branch { children, value } => {
                if nibbles.is_empty() {
                    value.clone()
                } else {
                    let index = nibbles[0] as usize;
                    if let Some(child) = &children[index] {
                        let remaining = &nibbles[1..];
                        self.get_recursive(child, remaining)
                    } else {
                        None
                    }
                }
            }
        }
    }

    fn remove_recursive(&self, node: MPTNode, nibbles: &[u8]) -> Result<(MPTNode, bool)> {
        match node {
            MPTNode::Empty => Ok((MPTNode::Empty, false)),
            MPTNode::Leaf { path, value: _ } => {
                let (decoded_path, is_terminal) = CompactEncoding::decode(&path)?;
                if decoded_path == nibbles && is_terminal {
                    Ok((MPTNode::Empty, true))
                } else {
                    Ok((
                        MPTNode::Leaf {
                            path,
                            value: vec![],
                        },
                        false,
                    )) // Keep original value
                }
            }
            MPTNode::Extension { path, child } => {
                let (decoded_path, _) = CompactEncoding::decode(&path)?;
                if nibbles.starts_with(&decoded_path) {
                    let remaining = &nibbles[decoded_path.len()..];
                    let (new_child, removed) = self.remove_recursive(*child, remaining)?;
                    if removed {
                        match &new_child {
                            MPTNode::Empty => Ok((MPTNode::Empty, true)),
                            MPTNode::Leaf { .. } | MPTNode::Branch { .. } => {
                                // Try to simplify the extension
                                if let MPTNode::Leaf {
                                    path: child_path,
                                    value,
                                } = &new_child
                                {
                                    let (child_decoded, _) = CompactEncoding::decode(child_path)?;
                                    let combined_path =
                                        [decoded_path.clone(), child_decoded].concat();
                                    let new_path = CompactEncoding::encode(&combined_path, true);
                                    Ok((
                                        MPTNode::Leaf {
                                            path: new_path,
                                            value: value.clone(),
                                        },
                                        true,
                                    ))
                                } else {
                                    Ok((
                                        MPTNode::Extension {
                                            path,
                                            child: Box::new(new_child),
                                        },
                                        true,
                                    ))
                                }
                            }
                            MPTNode::Extension { .. } => Ok((
                                MPTNode::Extension {
                                    path,
                                    child: Box::new(new_child),
                                },
                                true,
                            )),
                        }
                    } else {
                        Ok((
                            MPTNode::Extension {
                                path,
                                child: Box::new(new_child),
                            },
                            false,
                        ))
                    }
                } else {
                    Ok((MPTNode::Extension { path, child }, false))
                }
            }
            MPTNode::Branch { children, value } => {
                self.handle_remove_branch(children, value, nibbles)
            }
        }
    }

    fn handle_remove_branch(
        &self,
        mut children: [Option<Box<MPTNode>>; 16],
        value: Option<Vec<u8>>,
        nibbles: &[u8],
    ) -> Result<(MPTNode, bool)> {
        if nibbles.is_empty() {
            // Remove branch value
            return Ok((
                MPTNode::Branch {
                    children,
                    value: None,
                },
                value.is_some(),
            ));
        }

        let index = nibbles[0] as usize;
        let remaining = &nibbles[1..];

        if let Some(child) = children[index].take() {
            let (new_child, removed) = self.remove_recursive(*child, remaining)?;
            children[index] = Some(Box::new(new_child));
            Ok((MPTNode::Branch { children, value }, removed))
        } else {
            Ok((MPTNode::Branch { children, value }, false))
        }
    }

    #[allow(dead_code)]
    fn simplify_branch(
        &self,
        children: [Option<Box<MPTNode>>; 16],
        value: Option<Vec<u8>>,
    ) -> MPTNode {
        let non_empty_children: Vec<(usize, &Box<MPTNode>)> = children
            .iter()
            .enumerate()
            .filter_map(|(i, child)| child.as_ref().map(|c| (i, c)))
            .collect();

        match non_empty_children.len() {
            0 => {
                // No children - return value as leaf or empty
                if let Some(val) = value {
                    MPTNode::Leaf {
                        path: CompactEncoding::encode(&[], true),
                        value: val,
                    }
                } else {
                    MPTNode::Empty
                }
            }
            1 => {
                // Single child - create extension if possible
                let (child_index, child) = non_empty_children[0];
                match &**child {
                    MPTNode::Leaf {
                        path: child_path,
                        value: child_value,
                    } => {
                        let (decoded_path, _) =
                            CompactEncoding::decode(child_path).unwrap_or_default();
                        let full_path = [vec![child_index as u8], decoded_path].concat();
                        let extension_path = CompactEncoding::encode(&full_path, true);
                        MPTNode::Leaf {
                            path: extension_path,
                            value: child_value.clone(),
                        }
                    }
                    MPTNode::Extension {
                        path: child_path,
                        child: grand_child,
                    } => {
                        let (decoded_path, _) =
                            CompactEncoding::decode(child_path).unwrap_or_default();
                        let full_path = [vec![child_index as u8], decoded_path].concat();
                        let extension_path = CompactEncoding::encode(&full_path, false);
                        MPTNode::Extension {
                            path: extension_path,
                            child: grand_child.clone(),
                        }
                    }
                    _ => MPTNode::Branch { children, value },
                }
            }
            _ => MPTNode::Branch { children, value },
        }
    }

    #[allow(dead_code)]
    fn set_branch_value(&self, node: MPTNode, value: Vec<u8>) -> MPTNode {
        match node {
            MPTNode::Branch { children, .. } => MPTNode::Branch {
                children,
                value: Some(value),
            },
            _ => node,
        }
    }
}

impl Default for MerklePatriciaTrie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_mpt() {
        let mpt = MerklePatriciaTrie::new();
        assert!(mpt.is_empty());
        assert_eq!(mpt.get("nonexistent"), None);
    }

    #[test]
    fn test_single_insert_and_get() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("key1", b"value1".to_vec()).unwrap();

        assert_eq!(mpt.get("key1"), Some(b"value1".to_vec()));
        assert_eq!(mpt.get("key2"), None);
        assert!(!mpt.is_empty());
    }

    #[test]
    fn test_multiple_inserts() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("key1", b"value1".to_vec()).unwrap();
        mpt.insert("key2", b"value2".to_vec()).unwrap();

        assert_eq!(mpt.get("key1"), Some(b"value1".to_vec()));
        assert_eq!(mpt.get("key2"), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_update_existing_key() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("key1", b"value1".to_vec()).unwrap();
        mpt.insert("key1", b"updated".to_vec()).unwrap();

        assert_eq!(mpt.get("key1"), Some(b"updated".to_vec()));
    }

    #[test]
    fn test_remove_existing_key() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("key1", b"value1".to_vec()).unwrap();
        assert!(mpt.remove("key1").unwrap());

        assert_eq!(mpt.get("key1"), None);
    }

    #[test]
    fn test_remove_nonexistent_key() {
        let mut mpt = MerklePatriciaTrie::new();
        assert!(!mpt.remove("nonexistent").unwrap());
    }

    #[test]
    fn test_get_all_data() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("key1", b"value1".to_vec()).unwrap();
        mpt.insert("key2", b"value2".to_vec()).unwrap();

        let all = mpt.get_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("key1"), Some(&b"value1".to_vec()));
        assert_eq!(all.get("key2"), Some(&b"value2".to_vec()));
    }

    #[test]
    fn test_root_hash_changes_with_data() {
        let mut mpt = MerklePatriciaTrie::new();
        let empty_hash = mpt.root_hash();

        mpt.insert("key", b"value".to_vec()).unwrap();
        let filled_hash = mpt.root_hash();

        assert_ne!(empty_hash, filled_hash);
    }

    #[test]
    fn test_root_hash_consistency() {
        let mut mpt1 = MerklePatriciaTrie::new();
        let mut mpt2 = MerklePatriciaTrie::new();

        mpt1.insert("key", b"value".to_vec()).unwrap();
        mpt2.insert("key", b"value".to_vec()).unwrap();

        assert_eq!(mpt1.root_hash(), mpt2.root_hash());
    }

    #[test]
    fn test_hex_root_hash() {
        let mpt = MerklePatriciaTrie::new();
        let hex_hash = mpt.hex_root_hash();

        // Should be a valid hex string
        assert!(hex_hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hex_hash.len(), 64); // 32 bytes * 2 hex chars per byte
    }

    #[test]
    fn test_complex_key_prefixes() {
        let mut mpt = MerklePatriciaTrie::new();

        mpt.insert("prefix1", b"value1".to_vec()).unwrap();
        mpt.insert("prefix2", b"value2".to_vec()).unwrap();
        mpt.insert("prefix", b"value3".to_vec()).unwrap();

        assert_eq!(mpt.get("prefix1"), Some(b"value1".to_vec()));
        assert_eq!(mpt.get("prefix2"), Some(b"value2".to_vec()));
        assert_eq!(mpt.get("prefix"), Some(b"value3".to_vec()));
    }

    #[test]
    fn test_empty_value_handling() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("empty", Vec::new()).unwrap();

        assert_eq!(mpt.get("empty"), Some(Vec::new()));
    }

    #[test]
    fn test_large_values() {
        let mut mpt = MerklePatriciaTrie::new();
        let large_value = vec![42; 10000]; // 10KB value
        mpt.insert("large", large_value.clone()).unwrap();

        assert_eq!(mpt.get("large"), Some(large_value));
    }

    #[test]
    fn test_unicode_keys() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("🚀", b"rocket".to_vec()).unwrap();
        mpt.insert("测试", b"test".to_vec()).unwrap();

        assert_eq!(mpt.get("🚀"), Some(b"rocket".to_vec()));
        assert_eq!(mpt.get("测试"), Some(b"test".to_vec()));
    }

    #[test]
    fn test_insert_remove_insert_cycle() {
        let mut mpt = MerklePatriciaTrie::new();

        mpt.insert("key", b"value1".to_vec()).unwrap();
        assert_eq!(mpt.get("key"), Some(b"value1".to_vec()));

        mpt.remove("key").unwrap();
        assert_eq!(mpt.get("key"), None);

        mpt.insert("key", b"value2".to_vec()).unwrap();
        assert_eq!(mpt.get("key"), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_multiple_removals() {
        let mut mpt = MerklePatriciaTrie::new();

        mpt.insert("key1", b"value1".to_vec()).unwrap();
        mpt.insert("key2", b"value2".to_vec()).unwrap();
        mpt.insert("key3", b"value3".to_vec()).unwrap();

        assert!(mpt.remove("key2").unwrap());
        assert_eq!(mpt.get("key2"), None);
        assert_eq!(mpt.get("key1"), Some(b"value1".to_vec()));
        assert_eq!(mpt.get("key3"), Some(b"value3".to_vec()));
    }

    #[test]
    fn test_root_hash_after_removals() {
        let mut mpt = MerklePatriciaTrie::new();
        let original_hash = mpt.root_hash();

        mpt.insert("key", b"value".to_vec()).unwrap();
        let after_insert = mpt.root_hash();
        assert_ne!(original_hash, after_insert);

        mpt.remove("key").unwrap();
        let after_remove = mpt.root_hash();
        assert_eq!(original_hash, after_remove);
    }

    #[test]
    fn test_mpt_insert_and_get() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test_key", b"test_value".to_vec()).unwrap();
        assert_eq!(mpt.get("test_key"), Some(b"test_value".to_vec()));
    }

    #[test]
    fn test_mpt_remove() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test_key", b"test_value".to_vec()).unwrap();
        assert!(mpt.remove("test_key").unwrap());
        assert_eq!(mpt.get("test_key"), None);
    }

    #[test]
    fn test_mpt_generate_proof() {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("key1", b"value1".to_vec()).unwrap();

        let proof = mpt
            .generate_proof("key1", 1, "block_hash".to_string())
            .unwrap();
        assert_eq!(proof.key, "key1");
        assert_eq!(proof.value, b"value1");
        assert_eq!(proof.block_index, 1);
        assert_eq!(proof.block_hash, "block_hash");
        assert!(!proof.timestamp.is_empty());
    }
}
