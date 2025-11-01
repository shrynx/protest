//! Domain-specific generators
//!
//! This module provides generators for common domain-specific data types:
//! - Hexadecimal strings
//! - Base64 encoded strings
//! - File system paths
//! - UUIDs (v4)
//!
//! All generators use std library only (no external dependencies).

use protest::{Generator, GeneratorConfig};
use rand::Rng;
use std::path::{MAIN_SEPARATOR, PathBuf};

// ============================================================================
// Hex Generator
// ============================================================================

/// Generator for hexadecimal strings
///
/// Generates strings containing only hex characters (0-9, a-f)
#[derive(Debug, Clone)]
pub struct HexGenerator {
    min_len: usize,
    max_len: usize,
    uppercase: bool,
}

impl HexGenerator {
    /// Create a new hex generator with length bounds
    pub fn new(min_len: usize, max_len: usize) -> Self {
        Self {
            min_len,
            max_len,
            uppercase: false,
        }
    }

    /// Create a hex generator with uppercase letters (A-F instead of a-f)
    pub fn uppercase(min_len: usize, max_len: usize) -> Self {
        Self {
            min_len,
            max_len,
            uppercase: true,
        }
    }

    fn generate_hex_char(&self, rng: &mut dyn rand::RngCore) -> char {
        let hex_chars = if self.uppercase {
            b"0123456789ABCDEF"
        } else {
            b"0123456789abcdef"
        };
        hex_chars[rng.r#gen_range(0..hex_chars.len())] as char
    }
}

impl Generator<String> for HexGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let len = rng.r#gen_range(self.min_len..=self.max_len);
        (0..len).map(|_| self.generate_hex_char(rng)).collect()
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try empty string
        if !value.is_empty() && self.min_len == 0 {
            shrinks.push(String::new());
        }

        // Try all zeros
        if *value != "0".repeat(value.len()) && value.len() >= self.min_len {
            shrinks.push("0".repeat(value.len()));
        }

        // Try min length
        if value.len() > self.min_len {
            shrinks.push(value.chars().take(self.min_len).collect());
        }

        // Try removing characters
        if value.len() > self.min_len {
            for i in 0..value.len().min(3) {
                let mut chars: Vec<char> = value.chars().collect();
                chars.remove(i);
                let shrunk: String = chars.into_iter().collect();
                if shrunk.len() >= self.min_len {
                    shrinks.push(shrunk);
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Base64 Generator
// ============================================================================

/// Generator for Base64 encoded strings
///
/// Generates valid Base64 strings (standard encoding with padding)
#[derive(Debug, Clone)]
pub struct Base64Generator {
    min_bytes: usize,
    max_bytes: usize,
}

impl Base64Generator {
    /// Create a new Base64 generator
    ///
    /// The min_bytes and max_bytes refer to the number of random bytes to encode,
    /// not the length of the resulting Base64 string (which will be ~33% longer)
    pub fn new(min_bytes: usize, max_bytes: usize) -> Self {
        Self {
            min_bytes,
            max_bytes,
        }
    }

    fn encode_base64(bytes: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut result = String::new();
        let mut i = 0;

        // Process 3 bytes at a time
        while i + 2 < bytes.len() {
            let b1 = bytes[i];
            let b2 = bytes[i + 1];
            let b3 = bytes[i + 2];

            result.push(CHARS[(b1 >> 2) as usize] as char);
            result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
            result.push(CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char);
            result.push(CHARS[(b3 & 0x3f) as usize] as char);

            i += 3;
        }

        // Handle remaining bytes with padding
        if i < bytes.len() {
            let b1 = bytes[i];
            result.push(CHARS[(b1 >> 2) as usize] as char);

            if i + 1 < bytes.len() {
                let b2 = bytes[i + 1];
                result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
                result.push(CHARS[((b2 & 0x0f) << 2) as usize] as char);
                result.push('=');
            } else {
                result.push(CHARS[((b1 & 0x03) << 4) as usize] as char);
                result.push_str("==");
            }
        }

        result
    }
}

impl Generator<String> for Base64Generator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let num_bytes = rng.r#gen_range(self.min_bytes..=self.max_bytes);
        let bytes: Vec<u8> = (0..num_bytes).map(|_| rng.r#gen()).collect();
        Self::encode_base64(&bytes)
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try empty (0 bytes)
        if !value.is_empty() && self.min_bytes == 0 {
            shrinks.push(String::new());
        }

        // Try min bytes
        if !value.is_empty() && self.min_bytes > 0 {
            let bytes: Vec<u8> = (0..self.min_bytes).map(|_| 0u8).collect();
            shrinks.push(Self::encode_base64(&bytes));
        }

        // Try all zeros
        // Estimate byte count from base64 length
        let approx_bytes = (value.len() * 3) / 4;
        if approx_bytes >= self.min_bytes {
            let bytes = vec![0u8; approx_bytes];
            let encoded = Self::encode_base64(&bytes);
            if encoded != *value {
                shrinks.push(encoded);
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Path Generator
// ============================================================================

/// Generator for file system paths
///
/// Generates valid PathBuf instances with configurable depth and component lengths
#[derive(Debug, Clone)]
pub struct PathGenerator {
    min_depth: usize,
    max_depth: usize,
    absolute: bool,
}

impl PathGenerator {
    /// Create a new path generator
    ///
    /// depth refers to the number of path components (e.g., "a/b/c" has depth 3)
    pub fn new(min_depth: usize, max_depth: usize) -> Self {
        Self {
            min_depth,
            max_depth,
            absolute: false,
        }
    }

    /// Create a generator for absolute paths (starting with /)
    pub fn absolute(min_depth: usize, max_depth: usize) -> Self {
        Self {
            min_depth,
            max_depth,
            absolute: true,
        }
    }

    fn generate_component(&self, rng: &mut dyn rand::RngCore) -> String {
        let len = rng.r#gen_range(1..=12);
        let mut component = String::with_capacity(len);

        // First char: letter or underscore
        let first_chars = b"abcdefghijklmnopqrstuvwxyz_";
        component.push(first_chars[rng.r#gen_range(0..first_chars.len())] as char);

        // Remaining chars: letters, digits, underscore, hyphen, dot
        let chars = b"abcdefghijklmnopqrstuvwxyz0123456789_-.";
        for _ in 1..len {
            component.push(chars[rng.r#gen_range(0..chars.len())] as char);
        }

        component
    }
}

impl Generator<PathBuf> for PathGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> PathBuf {
        let depth = rng.r#gen_range(self.min_depth.max(1)..=self.max_depth.max(1));
        let mut path = PathBuf::new();

        if self.absolute {
            // Use platform-appropriate root
            // Unix: "/", Windows: "C:\", others: "/"
            #[cfg(target_family = "unix")]
            {
                path.push(MAIN_SEPARATOR.to_string());
            }
            #[cfg(target_family = "windows")]
            {
                path.push(format!("C:{}", MAIN_SEPARATOR));
            }
            #[cfg(not(any(target_family = "unix", target_family = "windows")))]
            {
                path.push(MAIN_SEPARATOR.to_string());
            }
        }

        for _ in 0..depth {
            path.push(self.generate_component(rng));
        }

        path
    }

    fn shrink(&self, value: &PathBuf) -> Box<dyn Iterator<Item = PathBuf>> {
        let mut shrinks = Vec::new();

        let components: Vec<_> = value.components().collect();
        let depth = components.len();

        // Try min depth
        if depth > self.min_depth {
            let mut path = PathBuf::new();
            if self.absolute {
                path.push("/");
            }
            for component in components.iter().take(self.min_depth.max(1)) {
                path.push(component);
            }
            shrinks.push(path);
        }

        // Try removing one component at a time
        if depth > self.min_depth {
            for i in (if self.absolute { 1 } else { 0 })..components.len().min(4) {
                let mut path = PathBuf::new();
                if self.absolute {
                    path.push("/");
                }
                for (j, component) in components.iter().enumerate() {
                    if j != i {
                        path.push(component);
                    }
                }
                if path.components().count() >= self.min_depth {
                    shrinks.push(path);
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// UUID v4 Generator
// ============================================================================

/// Generator for UUID v4 (random UUIDs)
///
/// Generates valid UUID v4 strings in the standard format:
/// xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
///
/// Where x is any hex digit and y is one of 8, 9, a, or b.
/// Implementation uses std library only (no uuid crate dependency).
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV4Generator;

impl UuidV4Generator {
    /// Create a new UUID v4 generator
    pub fn new() -> Self {
        Self
    }

    fn format_uuid(bytes: [u8; 16]) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15]
        )
    }
}

impl Generator<String> for UuidV4Generator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);

        // Set version to 4 (bits 12-15 of time_hi_and_version)
        bytes[6] = (bytes[6] & 0x0f) | 0x40;

        // Set variant to RFC4122 (bits 6-7 of clock_seq_hi_and_reserved)
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Self::format_uuid(bytes)
    }

    fn shrink(&self, _value: &String) -> Box<dyn Iterator<Item = String>> {
        // UUIDs don't shrink meaningfully - they're random identifiers
        // Return a few fixed UUIDs as minimal examples
        let shrinks = vec![
            "00000000-0000-4000-8000-000000000000".to_string(),
            "00000000-0000-4000-8000-000000000001".to_string(),
        ];
        Box::new(shrinks.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_hex_generator() {
        let generator = HexGenerator::new(8, 16);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let hex = generator.generate(&mut rng, &config);
            assert!(hex.len() >= 8 && hex.len() <= 16);
            assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
            assert!(hex.chars().all(|c| !c.is_uppercase()));
        }
    }

    #[test]
    fn test_hex_generator_uppercase() {
        let generator = HexGenerator::uppercase(8, 16);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let hex = generator.generate(&mut rng, &config);
            assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
            // Check that if there are letters, they're uppercase
            for c in hex.chars() {
                if c.is_alphabetic() {
                    assert!(c.is_uppercase());
                }
            }
        }
    }

    #[test]
    fn test_base64_generator() {
        let generator = Base64Generator::new(6, 32);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let b64 = generator.generate(&mut rng, &config);
            assert!(!b64.is_empty());

            // Check valid Base64 characters
            for c in b64.chars() {
                assert!(
                    c.is_alphanumeric() || c == '+' || c == '/' || c == '=',
                    "Invalid Base64 char: {}",
                    c
                );
            }

            // Check padding is only at the end
            if b64.contains('=') {
                assert!(b64.ends_with('=') || b64.ends_with("=="));
            }
        }
    }

    #[test]
    fn test_base64_encode() {
        // Test known Base64 encodings
        assert_eq!(Base64Generator::encode_base64(b"hello"), "aGVsbG8=");
        assert_eq!(
            Base64Generator::encode_base64(b"hello world"),
            "aGVsbG8gd29ybGQ="
        );
        assert_eq!(Base64Generator::encode_base64(b"a"), "YQ==");
        assert_eq!(Base64Generator::encode_base64(b"ab"), "YWI=");
        assert_eq!(Base64Generator::encode_base64(b"abc"), "YWJj");
    }

    #[test]
    fn test_path_generator() {
        let generator = PathGenerator::new(1, 4);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let path = generator.generate(&mut rng, &config);
            let components: Vec<_> = path.components().collect();
            assert!(!components.is_empty() && components.len() <= 4);

            // Check each component is valid
            for component in path.iter() {
                let s = component.to_string_lossy();
                assert!(!s.is_empty());
                assert!(!s.contains('/'));
            }
        }
    }

    #[test]
    fn test_path_generator_absolute() {
        let generator = PathGenerator::absolute(2, 4);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let path = generator.generate(&mut rng, &config);
            assert!(path.is_absolute());
        }
    }

    #[test]
    fn test_uuid_v4_generator() {
        let generator = UuidV4Generator::new();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let uuid = generator.generate(&mut rng, &config);

            // Check format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
            let parts: Vec<&str> = uuid.split('-').collect();
            assert_eq!(parts.len(), 5);
            assert_eq!(parts[0].len(), 8);
            assert_eq!(parts[1].len(), 4);
            assert_eq!(parts[2].len(), 4);
            assert_eq!(parts[3].len(), 4);
            assert_eq!(parts[4].len(), 12);

            // Check version is 4
            assert_eq!(parts[2].chars().next().unwrap(), '4');

            // Check variant is 8, 9, a, or b
            let variant = parts[3].chars().next().unwrap();
            assert!(
                variant == '8' || variant == '9' || variant == 'a' || variant == 'b',
                "Invalid variant: {}",
                variant
            );

            // Check all characters are hex
            for c in uuid.chars() {
                assert!(c.is_ascii_hexdigit() || c == '-');
            }
        }
    }

    #[test]
    fn test_uuid_v4_uniqueness() {
        // Generate multiple UUIDs and ensure they're unique
        let generator = UuidV4Generator::new();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let mut uuids = std::collections::HashSet::new();
        for _ in 0..100 {
            let uuid = generator.generate(&mut rng, &config);
            assert!(uuids.insert(uuid), "Generated duplicate UUID");
        }
    }
}
