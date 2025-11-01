//! Text and string generators
//!
//! This module provides generators for various text patterns:
//! - Alphabetic strings (a-z, A-Z)
//! - Alphanumeric strings (a-z, A-Z, 0-9)
//! - Programming identifiers (valid Rust/C/etc identifiers)
//! - Sentences (realistic sentence-like text)
//! - Paragraphs (multiple sentences)
//!
//! All generators use std library only.

use protest::{Generator, GeneratorConfig};
use rand::Rng;

// ============================================================================
// Alphabetic Generator
// ============================================================================

/// Generator for alphabetic strings (only letters a-z, A-Z)
#[derive(Debug, Clone)]
pub struct AlphabeticGenerator {
    min_len: usize,
    max_len: usize,
    lowercase_only: bool,
    uppercase_only: bool,
}

impl AlphabeticGenerator {
    /// Create a new alphabetic generator with length bounds
    pub fn new(min_len: usize, max_len: usize) -> Self {
        Self {
            min_len,
            max_len,
            lowercase_only: false,
            uppercase_only: false,
        }
    }

    /// Generate only lowercase letters
    pub fn lowercase(min_len: usize, max_len: usize) -> Self {
        Self {
            min_len,
            max_len,
            lowercase_only: true,
            uppercase_only: false,
        }
    }

    /// Generate only uppercase letters
    pub fn uppercase(min_len: usize, max_len: usize) -> Self {
        Self {
            min_len,
            max_len,
            lowercase_only: false,
            uppercase_only: true,
        }
    }

    fn generate_char(&self, rng: &mut dyn rand::RngCore) -> char {
        let lowercase = b"abcdefghijklmnopqrstuvwxyz";
        let uppercase = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        if self.lowercase_only {
            lowercase[rng.r#gen_range(0..lowercase.len())] as char
        } else if self.uppercase_only {
            uppercase[rng.r#gen_range(0..uppercase.len())] as char
        } else {
            // Mix of both
            let all = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
            all[rng.r#gen_range(0..all.len())] as char
        }
    }
}

impl Generator<String> for AlphabeticGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let len = rng.r#gen_range(self.min_len..=self.max_len);
        (0..len).map(|_| self.generate_char(rng)).collect()
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try empty string
        if !value.is_empty() && self.min_len == 0 {
            shrinks.push(String::new());
        }

        // Try shrinking to min length
        if value.len() > self.min_len {
            shrinks.push(value.chars().take(self.min_len).collect());
        }

        // Try removing one character at a time
        if value.len() > self.min_len {
            for i in 0..value.len().min(3) {
                let mut chars: Vec<char> = value.chars().collect();
                if !chars.is_empty() {
                    chars.remove(i);
                    let shrunk: String = chars.into_iter().collect();
                    if shrunk.len() >= self.min_len {
                        shrinks.push(shrunk);
                    }
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Alphanumeric Generator
// ============================================================================

/// Generator for alphanumeric strings (letters and digits)
#[derive(Debug, Clone)]
pub struct AlphanumericGenerator {
    min_len: usize,
    max_len: usize,
}

impl AlphanumericGenerator {
    /// Create a new alphanumeric generator with length bounds
    pub fn new(min_len: usize, max_len: usize) -> Self {
        Self { min_len, max_len }
    }

    fn generate_char(&self, rng: &mut dyn rand::RngCore) -> char {
        let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        chars[rng.r#gen_range(0..chars.len())] as char
    }
}

impl Generator<String> for AlphanumericGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let len = rng.r#gen_range(self.min_len..=self.max_len);
        (0..len).map(|_| self.generate_char(rng)).collect()
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        if !value.is_empty() && self.min_len == 0 {
            shrinks.push(String::new());
        }

        if value.len() > self.min_len {
            shrinks.push(value.chars().take(self.min_len).collect());
        }

        if value.len() > self.min_len {
            for i in 0..value.len().min(3) {
                let mut chars: Vec<char> = value.chars().collect();
                if !chars.is_empty() {
                    chars.remove(i);
                    let shrunk: String = chars.into_iter().collect();
                    if shrunk.len() >= self.min_len {
                        shrinks.push(shrunk);
                    }
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Identifier Generator
// ============================================================================

/// Generator for valid programming identifiers (Rust/C style)
///
/// Generates strings that:
/// - Start with a letter or underscore
/// - Contain only letters, digits, and underscores
/// - Are valid identifiers in most programming languages
#[derive(Debug, Clone)]
pub struct IdentifierGenerator {
    min_len: usize,
    max_len: usize,
}

impl IdentifierGenerator {
    /// Create a new identifier generator with length bounds
    pub fn new(min_len: usize, max_len: usize) -> Self {
        Self { min_len, max_len }
    }

    fn generate_first_char(&self, rng: &mut dyn rand::RngCore) -> char {
        // First char must be letter or underscore
        let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        chars[rng.r#gen_range(0..chars.len())] as char
    }

    fn generate_char(&self, rng: &mut dyn rand::RngCore) -> char {
        // Subsequent chars can be letters, digits, or underscore
        let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
        chars[rng.r#gen_range(0..chars.len())] as char
    }
}

impl Generator<String> for IdentifierGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let len = rng.r#gen_range(self.min_len.max(1)..=self.max_len.max(1));
        let mut result = String::with_capacity(len);

        // First character
        result.push(self.generate_first_char(rng));

        // Remaining characters
        for _ in 1..len {
            result.push(self.generate_char(rng));
        }

        result
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try simple identifiers
        if value != "a" && self.min_len <= 1 {
            shrinks.push("a".to_string());
        }

        if value != "x" && self.min_len <= 1 {
            shrinks.push("x".to_string());
        }

        // Try shrinking to min length
        if value.len() > self.min_len {
            shrinks.push(value.chars().take(self.min_len.max(1)).collect());
        }

        // Try removing characters
        if value.len() > self.min_len.max(1) {
            for i in 1..value.len().min(4) {
                let mut chars: Vec<char> = value.chars().collect();
                chars.remove(i);
                let shrunk: String = chars.into_iter().collect();
                if shrunk.len() >= self.min_len.max(1) {
                    shrinks.push(shrunk);
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Sentence Generator
// ============================================================================

/// Generator for sentence-like text
///
/// Generates strings that resemble English sentences:
/// - Start with capital letter
/// - End with period
/// - Contain words separated by spaces
#[derive(Debug, Clone)]
pub struct SentenceGenerator {
    min_words: usize,
    max_words: usize,
}

impl SentenceGenerator {
    /// Create a new sentence generator
    pub fn new(min_words: usize, max_words: usize) -> Self {
        Self {
            min_words,
            max_words,
        }
    }

    fn generate_word(&self, rng: &mut dyn rand::RngCore, is_first: bool) -> String {
        let len = rng.r#gen_range(2..8);
        let mut word = String::with_capacity(len);

        for i in 0..len {
            let c = if i == 0 && is_first {
                // First letter of first word is capital
                (b'A' + rng.r#gen_range(0..26)) as char
            } else {
                // Other letters are lowercase
                (b'a' + rng.r#gen_range(0..26)) as char
            };
            word.push(c);
        }

        word
    }
}

impl Generator<String> for SentenceGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let num_words = rng.r#gen_range(self.min_words.max(1)..=self.max_words.max(1));
        let mut sentence = String::new();

        for i in 0..num_words {
            if i > 0 {
                sentence.push(' ');
            }
            sentence.push_str(&self.generate_word(rng, i == 0));
        }

        sentence.push('.');
        sentence
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try minimal sentence
        if value != "A." && self.min_words <= 1 {
            shrinks.push("A.".to_string());
        }

        // Try removing words
        let words: Vec<&str> = value.trim_end_matches('.').split_whitespace().collect();
        if words.len() > self.min_words {
            // Take first min_words
            let shrunk_words = &words[..self.min_words.max(1)];
            let mut shrunk = shrunk_words.join(" ");
            // Capitalize first letter if needed
            if let Some(first_char) = shrunk.chars().next()
                && first_char.is_lowercase()
            {
                let capitalized = first_char.to_uppercase().collect::<String>() + &shrunk[1..];
                shrunk = capitalized;
            }
            shrunk.push('.');
            shrinks.push(shrunk);
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Paragraph Generator
// ============================================================================

/// Generator for paragraph-like text
///
/// Generates multiple sentences separated by spaces
#[derive(Debug, Clone)]
pub struct ParagraphGenerator {
    min_sentences: usize,
    max_sentences: usize,
}

impl ParagraphGenerator {
    /// Create a new paragraph generator
    pub fn new(min_sentences: usize, max_sentences: usize) -> Self {
        Self {
            min_sentences,
            max_sentences,
        }
    }
}

impl Generator<String> for ParagraphGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> String {
        let num_sentences = rng.r#gen_range(self.min_sentences.max(1)..=self.max_sentences.max(1));
        let sentence_gen = SentenceGenerator::new(3, 10);

        let sentences: Vec<String> = (0..num_sentences)
            .map(|_| sentence_gen.generate(rng, config))
            .collect();

        sentences.join(" ")
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try minimal paragraph
        if value != "A." && self.min_sentences <= 1 {
            shrinks.push("A.".to_string());
        }

        // Try removing sentences
        let sentences: Vec<&str> = value.split(". ").collect();
        if sentences.len() > self.min_sentences {
            let shrunk_sentences = &sentences[..self.min_sentences.max(1)];
            let mut shrunk = shrunk_sentences.join(". ");
            if !shrunk.ends_with('.') {
                shrunk.push('.');
            }
            shrinks.push(shrunk);
        }

        Box::new(shrinks.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_alphabetic_generator() {
        let generator = AlphabeticGenerator::new(5, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let text = generator.generate(&mut rng, &config);
            assert!(text.len() >= 5 && text.len() <= 10);
            assert!(text.chars().all(|c| c.is_alphabetic()));
        }
    }

    #[test]
    fn test_alphabetic_lowercase_only() {
        let generator = AlphabeticGenerator::lowercase(5, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let text = generator.generate(&mut rng, &config);
            assert!(text.chars().all(|c| c.is_lowercase()));
        }
    }

    #[test]
    fn test_alphanumeric_generator() {
        let generator = AlphanumericGenerator::new(5, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let text = generator.generate(&mut rng, &config);
            assert!(text.len() >= 5 && text.len() <= 10);
            assert!(text.chars().all(|c| c.is_alphanumeric()));
        }
    }

    #[test]
    fn test_identifier_generator() {
        let generator = IdentifierGenerator::new(3, 15);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let id = generator.generate(&mut rng, &config);
            assert!(id.len() >= 3 && id.len() <= 15);

            // First char must be letter or underscore
            let first = id.chars().next().unwrap();
            assert!(first.is_alphabetic() || first == '_');

            // All chars must be alphanumeric or underscore
            assert!(id.chars().all(|c| c.is_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn test_sentence_generator() {
        let generator = SentenceGenerator::new(3, 8);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let sentence = generator.generate(&mut rng, &config);

            // Should end with period
            assert!(sentence.ends_with('.'));

            // Should start with capital letter
            assert!(sentence.chars().next().unwrap().is_uppercase());

            // Should have words
            let words: Vec<&str> = sentence.trim_end_matches('.').split_whitespace().collect();
            assert!(words.len() >= 3 && words.len() <= 8);
        }
    }

    #[test]
    fn test_paragraph_generator() {
        let generator = ParagraphGenerator::new(2, 5);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..5 {
            let paragraph = generator.generate(&mut rng, &config);

            // Should contain multiple periods
            let period_count = paragraph.matches('.').count();
            assert!((2..=5).contains(&period_count));
        }
    }
}
