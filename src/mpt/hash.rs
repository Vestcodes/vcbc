//! Hashing and key utilities for MPT operations.
//!
//! Provides key conversion utilities and path manipulation functions
//! for efficient trie operations.

/// Key conversion utilities
pub struct KeyUtils;

impl KeyUtils {
    /// Convert string key to nibbles (4-bit chunks)
    pub fn string_to_nibbles(key: &str) -> Vec<u8> {
        let mut nibbles = Vec::new();
        for byte in key.as_bytes() {
            nibbles.push(byte >> 4); // High 4 bits
            nibbles.push(byte & 0x0F); // Low 4 bits
        }
        nibbles
    }

    /// Convert nibbles back to string
    pub fn nibbles_to_string(nibbles: &[u8]) -> String {
        let mut bytes = Vec::new();
        for chunk in nibbles.chunks(2) {
            if chunk.len() == 2 {
                let byte = (chunk[0] << 4) | chunk[1];
                bytes.push(byte);
            }
        }
        String::from_utf8_lossy(&bytes).to_string()
    }
}

/// Path manipulation utilities
pub struct PathUtils;

impl PathUtils {
    /// Find the length of the common prefix between two paths
    pub fn common_prefix_length(path1: &[u8], path2: &[u8]) -> usize {
        path1
            .iter()
            .zip(path2.iter())
            .take_while(|(a, b)| a == b)
            .count()
    }

    /// Split a path at the given index
    pub fn split_at(path: &[u8], index: usize) -> (&[u8], &[u8]) {
        (&path[..index], &path[index..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_nibbles() {
        let nibbles = KeyUtils::string_to_nibbles("a");
        // 'a' is 97 in ASCII, which is 01100001 in binary
        // High 4 bits: 0110 = 6, Low 4 bits: 0001 = 1
        assert_eq!(nibbles, vec![6, 1]);

        let nibbles = KeyUtils::string_to_nibbles("ab");
        // 'a' -> [6, 1], 'b' -> [6, 2]
        assert_eq!(nibbles, vec![6, 1, 6, 2]);
    }

    #[test]
    fn test_nibbles_to_string() {
        let nibbles = vec![6, 1];
        let string = KeyUtils::nibbles_to_string(&nibbles);
        assert_eq!(string, "a");

        let nibbles = vec![6, 1, 6, 2];
        let string = KeyUtils::nibbles_to_string(&nibbles);
        assert_eq!(string, "ab");
    }

    #[test]
    fn test_key_utils() {
        // Test roundtrip conversion
        let test_strings = vec!["", "a", "hello", "world", "🚀"];

        for original in test_strings {
            let nibbles = KeyUtils::string_to_nibbles(original);
            let reconstructed = KeyUtils::nibbles_to_string(&nibbles);
            assert_eq!(original, reconstructed);
        }
    }

    #[test]
    fn test_common_prefix_length() {
        assert_eq!(PathUtils::common_prefix_length(&[1, 2, 3], &[1, 2, 3]), 3);
        assert_eq!(PathUtils::common_prefix_length(&[1, 2, 3], &[1, 2, 4]), 2);
        assert_eq!(PathUtils::common_prefix_length(&[1, 2, 3], &[4, 5, 6]), 0);
        assert_eq!(PathUtils::common_prefix_length(&[], &[1, 2, 3]), 0);
        assert_eq!(PathUtils::common_prefix_length(&[1, 2, 3], &[]), 0);
    }

    #[test]
    fn test_split_at() {
        let path = &[1, 2, 3, 4, 5];

        let (left, right) = PathUtils::split_at(path, 0);
        assert_eq!(left, &[] as &[u8]);
        assert_eq!(right, &[1, 2, 3, 4, 5]);

        let (left, right) = PathUtils::split_at(path, 2);
        assert_eq!(left, &[1, 2]);
        assert_eq!(right, &[3, 4, 5]);

        let (left, right) = PathUtils::split_at(path, 5);
        assert_eq!(left, &[1, 2, 3, 4, 5]);
        assert_eq!(right, &[] as &[u8]);
    }
}
