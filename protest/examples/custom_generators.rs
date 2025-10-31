//! Custom generator examples
//!
//! This example demonstrates how to create custom generators for complex data types,
//! including domain-specific generators, composite generators, and generators with
//! custom shrinking strategies.

use protest::{
    Generator, GeneratorConfig, Property, PropertyError, PropertyTestBuilder, TestConfig, check,
    check_with_config,
};
use rand::RngCore;
use std::collections::{HashMap, HashSet};

// Example 1: Simple custom generator for a domain-specific type
#[derive(Debug, Clone, PartialEq)]
struct EmailAddress {
    local: String,
    domain: String,
}

impl EmailAddress {
    fn new(local: String, domain: String) -> Self {
        Self { local, domain }
    }

    fn as_string(&self) -> String {
        format!("{}@{}", self.local, self.domain)
    }

    fn is_valid(&self) -> bool {
        !self.local.is_empty()
            && !self.domain.is_empty()
            && !self.local.contains('@')
            && !self.domain.contains('@')
            && self.domain.contains('.')
    }
}

struct EmailGenerator {
    local_chars: Vec<char>,
    domain_parts: Vec<String>,
    tlds: Vec<String>,
}

impl EmailGenerator {
    fn new() -> Self {
        Self {
            local_chars: "abcdefghijklmnopqrstuvwxyz0123456789._-".chars().collect(),
            domain_parts: vec![
                "gmail".to_string(),
                "yahoo".to_string(),
                "hotmail".to_string(),
                "example".to_string(),
                "test".to_string(),
                "company".to_string(),
            ],
            tlds: vec![
                "com".to_string(),
                "org".to_string(),
                "net".to_string(),
                "edu".to_string(),
                "gov".to_string(),
            ],
        }
    }
}

impl Generator<EmailAddress> for EmailGenerator {
    fn generate(&self, rng: &mut dyn RngCore, _config: &GeneratorConfig) -> EmailAddress {
        // Generate local part (1-20 characters)
        let local_len = (rng.next_u32() % 20) + 1;
        let local: String = (0..local_len)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % self.local_chars.len();
                self.local_chars[idx]
            })
            .collect();

        // Generate domain part
        let domain_idx = (rng.next_u32() as usize) % self.domain_parts.len();
        let tld_idx = (rng.next_u32() as usize) % self.tlds.len();
        let domain = format!("{}.{}", self.domain_parts[domain_idx], self.tlds[tld_idx]);

        EmailAddress::new(local, domain)
    }

    fn shrink(&self, value: &EmailAddress) -> Box<dyn Iterator<Item = EmailAddress>> {
        let mut shrinks = Vec::new();

        // Shrink local part
        if value.local.len() > 1 {
            let shorter_local = value.local[..value.local.len() - 1].to_string();
            if !shorter_local.is_empty() {
                shrinks.push(EmailAddress::new(shorter_local, value.domain.clone()));
            }
        }

        // Try simpler local parts
        if value.local != "a" {
            shrinks.push(EmailAddress::new("a".to_string(), value.domain.clone()));
        }

        // Try simpler domain
        if value.domain != "test.com" {
            shrinks.push(EmailAddress::new(
                value.local.clone(),
                "test.com".to_string(),
            ));
        }

        Box::new(shrinks.into_iter())
    }
}

fn example_1_custom_email_generator() {
    println!("=== Example 1: Custom Email Generator ===");

    struct EmailValidityProperty;
    impl Property<EmailAddress> for EmailValidityProperty {
        type Output = ();

        fn test(&self, email: EmailAddress) -> Result<Self::Output, PropertyError> {
            if email.is_valid() {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Invalid email: {}",
                    email.as_string()
                )))
            }
        }
    }

    match check(EmailGenerator::new(), EmailValidityProperty) {
        Ok(success) => {
            println!(
                "✓ Email validity property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Email: {}", failure.original_input.as_string());
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk to: {}", shrunk.as_string());
            }
        }
    }
}

// Example 2: Composite generator for complex structures
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u32,
    email: EmailAddress,
    age: u8,
    preferences: HashMap<String, String>,
    tags: HashSet<String>,
}

struct UserGenerator {
    email_gen: EmailGenerator,
    preference_keys: Vec<String>,
    preference_values: Vec<String>,
    available_tags: Vec<String>,
}

impl UserGenerator {
    fn new() -> Self {
        Self {
            email_gen: EmailGenerator::new(),
            preference_keys: vec![
                "theme".to_string(),
                "language".to_string(),
                "timezone".to_string(),
                "notifications".to_string(),
            ],
            preference_values: vec![
                "dark".to_string(),
                "light".to_string(),
                "en".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "UTC".to_string(),
                "EST".to_string(),
                "enabled".to_string(),
                "disabled".to_string(),
            ],
            available_tags: vec![
                "premium".to_string(),
                "beta".to_string(),
                "admin".to_string(),
                "verified".to_string(),
                "new".to_string(),
            ],
        }
    }
}

impl Generator<User> for UserGenerator {
    fn generate(&self, rng: &mut dyn RngCore, config: &GeneratorConfig) -> User {
        let id = rng.next_u32();
        let email = self.email_gen.generate(rng, config);
        let age = (rng.next_u32() % 100) as u8 + 18; // 18-117 years old

        // Generate preferences (0-4 preferences)
        let pref_count = rng.next_u32() % 5;
        let mut preferences = HashMap::new();
        for _ in 0..pref_count {
            let key_idx = (rng.next_u32() as usize) % self.preference_keys.len();
            let val_idx = (rng.next_u32() as usize) % self.preference_values.len();
            preferences.insert(
                self.preference_keys[key_idx].clone(),
                self.preference_values[val_idx].clone(),
            );
        }

        // Generate tags (0-3 tags)
        let tag_count = rng.next_u32() % 4;
        let mut tags = HashSet::new();
        for _ in 0..tag_count {
            let tag_idx = (rng.next_u32() as usize) % self.available_tags.len();
            tags.insert(self.available_tags[tag_idx].clone());
        }

        User {
            id,
            email,
            age,
            preferences,
            tags,
        }
    }

    fn shrink(&self, value: &User) -> Box<dyn Iterator<Item = User>> {
        let mut shrinks = Vec::new();

        // Shrink ID towards 0
        if value.id > 0 {
            shrinks.push(User {
                id: value.id / 2,
                ..value.clone()
            });
            shrinks.push(User {
                id: 0,
                ..value.clone()
            });
        }

        // Shrink age towards minimum
        if value.age > 18 {
            shrinks.push(User {
                age: 18,
                ..value.clone()
            });
        }

        // Shrink preferences by removing entries
        if !value.preferences.is_empty() {
            let mut smaller_prefs = value.preferences.clone();
            if let Some(key) = smaller_prefs.keys().next().cloned() {
                smaller_prefs.remove(&key);
                shrinks.push(User {
                    preferences: smaller_prefs,
                    ..value.clone()
                });
            }
        }

        // Shrink tags by removing entries
        if !value.tags.is_empty() {
            let mut smaller_tags = value.tags.clone();
            if let Some(tag) = smaller_tags.iter().next().cloned() {
                smaller_tags.remove(&tag);
                shrinks.push(User {
                    tags: smaller_tags,
                    ..value.clone()
                });
            }
        }

        Box::new(shrinks.into_iter())
    }
}

fn example_2_composite_user_generator() {
    println!("\n=== Example 2: Composite User Generator ===");

    struct UserConsistencyProperty;
    impl Property<User> for UserConsistencyProperty {
        type Output = ();

        fn test(&self, user: User) -> Result<Self::Output, PropertyError> {
            // Property 1: Email should be valid
            if !user.email.is_valid() {
                return Err(PropertyError::property_failed(format!(
                    "User has invalid email: {}",
                    user.email.as_string()
                )));
            }

            // Property 2: Age should be reasonable
            if user.age < 18 || user.age > 120 {
                return Err(PropertyError::property_failed(format!(
                    "User age {} is unreasonable",
                    user.age
                )));
            }

            // Property 3: Premium users should be verified
            if user.tags.contains("premium") && !user.tags.contains("verified") {
                return Err(PropertyError::property_failed(
                    "Premium users must be verified".to_string(),
                ));
            }

            Ok(())
        }
    }

    let config = TestConfig {
        iterations: 50,
        seed: Some(999),
        ..TestConfig::default()
    };

    match check_with_config(UserGenerator::new(), UserConsistencyProperty, config) {
        Ok(success) => {
            println!(
                "✓ User consistency property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  User ID: {}", failure.original_input.id);
            println!("  Email: {}", failure.original_input.email.as_string());
            println!("  Age: {}", failure.original_input.age);
            println!("  Tags: {:?}", failure.original_input.tags);
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk user ID: {}", shrunk.id);
                println!("  Shrunk tags: {:?}", shrunk.tags);
            }
        }
    }
}

// Example 3: Generator with weighted choices
#[derive(Debug, Clone, PartialEq, Eq)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LogEntry {
    level: LogLevel,
    message: String,
    timestamp: u64,
    module: String,
}

struct LogEntryGenerator {
    modules: Vec<String>,
    message_templates: Vec<String>,
}

impl LogEntryGenerator {
    fn new() -> Self {
        Self {
            modules: vec![
                "auth".to_string(),
                "database".to_string(),
                "api".to_string(),
                "cache".to_string(),
                "scheduler".to_string(),
            ],
            message_templates: vec![
                "Operation completed successfully".to_string(),
                "Connection established".to_string(),
                "Request processed".to_string(),
                "Cache miss occurred".to_string(),
                "Validation failed".to_string(),
                "Timeout exceeded".to_string(),
                "Resource not found".to_string(),
                "Permission denied".to_string(),
            ],
        }
    }

    fn generate_log_level(&self, rng: &mut dyn RngCore) -> LogLevel {
        // Weighted distribution: Info and Debug are more common
        let weight = rng.next_u32() % 100;
        match weight {
            0..=40 => LogLevel::Info,
            41..=70 => LogLevel::Debug,
            71..=85 => LogLevel::Warning,
            86..=95 => LogLevel::Error,
            _ => LogLevel::Critical,
        }
    }
}

impl Generator<LogEntry> for LogEntryGenerator {
    fn generate(&self, rng: &mut dyn RngCore, _config: &GeneratorConfig) -> LogEntry {
        let level = self.generate_log_level(rng);

        let msg_idx = (rng.next_u32() as usize) % self.message_templates.len();
        let message = self.message_templates[msg_idx].clone();

        let timestamp = rng.next_u64() % 1_000_000_000; // Reasonable timestamp range

        let mod_idx = (rng.next_u32() as usize) % self.modules.len();
        let module = self.modules[mod_idx].clone();

        LogEntry {
            level,
            message,
            timestamp,
            module,
        }
    }

    fn shrink(&self, value: &LogEntry) -> Box<dyn Iterator<Item = LogEntry>> {
        let mut shrinks = Vec::new();

        // Shrink to simpler log level
        match value.level {
            LogLevel::Critical => {
                shrinks.push(LogEntry {
                    level: LogLevel::Error,
                    ..value.clone()
                });
            }
            LogLevel::Error => {
                shrinks.push(LogEntry {
                    level: LogLevel::Warning,
                    ..value.clone()
                });
            }
            LogLevel::Warning => {
                shrinks.push(LogEntry {
                    level: LogLevel::Info,
                    ..value.clone()
                });
            }
            _ => {}
        }

        // Shrink timestamp towards 0
        if value.timestamp > 0 {
            shrinks.push(LogEntry {
                timestamp: value.timestamp / 2,
                ..value.clone()
            });
        }

        // Shrink to simpler module
        if value.module != "api" {
            shrinks.push(LogEntry {
                module: "api".to_string(),
                ..value.clone()
            });
        }

        Box::new(shrinks.into_iter())
    }
}

fn example_3_weighted_log_generator() {
    println!("\n=== Example 3: Weighted Log Entry Generator ===");

    struct LogLevelDistributionProperty;
    impl Property<Vec<LogEntry>> for LogLevelDistributionProperty {
        type Output = ();

        fn test(&self, logs: Vec<LogEntry>) -> Result<Self::Output, PropertyError> {
            if logs.is_empty() {
                return Ok(());
            }

            let critical_count = logs
                .iter()
                .filter(|log| matches!(log.level, LogLevel::Critical))
                .count();
            let total_count = logs.len();

            // Property: Critical logs should be rare (less than 10% of total)
            let critical_ratio = critical_count as f64 / total_count as f64;
            if critical_ratio > 0.1 {
                return Err(PropertyError::property_failed(format!(
                    "Too many critical logs: {:.1}% (expected < 10%)",
                    critical_ratio * 100.0
                )));
            }

            // Property: All timestamps should be reasonable
            for log in &logs {
                if log.timestamp > 2_000_000_000 {
                    return Err(PropertyError::property_failed(format!(
                        "Unreasonable timestamp: {}",
                        log.timestamp
                    )));
                }
            }

            Ok(())
        }
    }

    let log_vec_generator =
        protest::primitives::VecGenerator::new(LogEntryGenerator::new(), 10, 100);

    match check(log_vec_generator, LogLevelDistributionProperty) {
        Ok(success) => {
            println!(
                "✓ Log distribution property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Log count: {}", failure.original_input.len());
            let critical_count = failure
                .original_input
                .iter()
                .filter(|log| matches!(log.level, LogLevel::Critical))
                .count();
            println!("  Critical logs: {}", critical_count);
        }
    }
}

// Example 4: Generator with configuration-dependent behavior
struct ConfigurableStringGenerator {
    charset: Vec<char>,
    min_length: usize,
    max_length: usize,
}

impl ConfigurableStringGenerator {
    fn ascii_only() -> Self {
        Self {
            charset: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                .chars()
                .collect(),
            min_length: 1,
            max_length: 50,
        }
    }

    fn with_special_chars() -> Self {
        Self {
            charset: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?".chars().collect(),
            min_length: 1,
            max_length: 50,
        }
    }

    #[allow(dead_code)]
    fn unicode_friendly() -> Self {
        Self {
            charset: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789αβγδεζηθικλμνξοπρστυφχψω🚀🎉🔥💡".chars().collect(),
            min_length: 1,
            max_length: 30,
        }
    }
}

impl Generator<String> for ConfigurableStringGenerator {
    fn generate(&self, rng: &mut dyn RngCore, config: &GeneratorConfig) -> String {
        // Use config.size_hint to influence string length
        let adjusted_max = (self.max_length * config.size_hint / 10).max(self.min_length);
        let length =
            self.min_length + (rng.next_u32() as usize % (adjusted_max - self.min_length + 1));

        (0..length)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % self.charset.len();
                self.charset[idx]
            })
            .collect()
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Shrink length
        if value.len() > self.min_length {
            shrinks.push(value[..value.len() - 1].to_string());
            if value.len() > self.min_length + 1 {
                shrinks.push(value[..self.min_length].to_string());
            }
        }

        // Try simpler characters
        if value.chars().any(|c| !c.is_ascii_alphanumeric()) {
            let simplified: String = value
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { 'a' })
                .collect();
            if simplified != *value {
                shrinks.push(simplified);
            }
        }

        Box::new(shrinks.into_iter())
    }
}

fn example_4_configurable_generator() {
    println!("\n=== Example 4: Configurable String Generator ===");

    struct StringCharsetProperty {
        allow_special: bool,
        allow_unicode: bool,
    }

    impl Property<String> for StringCharsetProperty {
        type Output = ();

        fn test(&self, input: String) -> Result<Self::Output, PropertyError> {
            for ch in input.chars() {
                if !self.allow_unicode && !ch.is_ascii() {
                    return Err(PropertyError::property_failed(format!(
                        "Non-ASCII character '{}' not allowed",
                        ch
                    )));
                }

                if !self.allow_special && !ch.is_alphanumeric() && ch != ' ' {
                    return Err(PropertyError::property_failed(format!(
                        "Special character '{}' not allowed",
                        ch
                    )));
                }
            }

            Ok(())
        }
    }

    // Test with ASCII-only generator
    println!("  Testing ASCII-only strings:");
    let result1 = PropertyTestBuilder::new()
        .iterations(30)
        // Note: generator_config method doesn't exist, using default
        .run(
            ConfigurableStringGenerator::ascii_only(),
            StringCharsetProperty {
                allow_special: false,
                allow_unicode: false,
            },
        );

    match result1 {
        Ok(success) => {
            println!(
                "    ✓ ASCII-only property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("    ✗ Property failed: {}", failure.error);
            println!("    String: {:?}", failure.original_input);
        }
    }

    // Test with special characters
    println!("  Testing strings with special characters:");
    let result2 = PropertyTestBuilder::new()
        .iterations(30)
        // Note: generator_config method doesn't exist, using default
        .run(
            ConfigurableStringGenerator::with_special_chars(),
            StringCharsetProperty {
                allow_special: true,
                allow_unicode: false,
            },
        );

    match result2 {
        Ok(success) => {
            println!(
                "    ✓ Special characters property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("    ✗ Property failed: {}", failure.error);
            println!("    String: {:?}", failure.original_input);
        }
    }
}

fn main() {
    println!("Protest Library - Custom Generator Examples");
    println!("==========================================");

    example_1_custom_email_generator();
    example_2_composite_user_generator();
    example_3_weighted_log_generator();
    example_4_configurable_generator();

    println!("\n=== Summary ===");
    println!("These custom generator examples demonstrate:");
    println!("• Creating domain-specific generators (EmailAddress)");
    println!("• Building composite generators for complex types (User)");
    println!("• Implementing weighted random choices (LogLevel)");
    println!("• Making generators configurable and context-aware");
    println!("• Custom shrinking strategies for better debugging");
    println!("• Integration with Protest's property testing framework");
    println!("\nCustom generators allow you to:");
    println!("• Model your domain accurately");
    println!("• Control the distribution of generated values");
    println!("• Implement domain-specific shrinking logic");
    println!("• Create reusable generators for common patterns");
}
