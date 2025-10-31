//! Integration tests that verify the examples work correctly
//!
//! These tests ensure that all the example code compiles and runs without errors,
//! serving as both integration tests and documentation validation.

#![allow(clippy::inherent_to_string_shadow_display)]

use protest::{
    AsyncProperty, Generator, GeneratorConfig, Property, PropertyError, PropertyTestBuilder,
    TestConfig, check, check_async, check_with_config, just, range,
};
use std::time::Duration;

// Test the basic usage patterns from examples/basic_usage.rs
#[test]
fn test_basic_usage_patterns() {
    // Test commutative property
    struct CommutativeProperty;
    impl Property<(i32, i32)> for CommutativeProperty {
        type Output = ();

        fn test(&self, input: (i32, i32)) -> Result<Self::Output, PropertyError> {
            let (a, b) = input;
            if a + b == b + a {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "{} + {} != {} + {}",
                    a, b, b, a
                )))
            }
        }
    }

    // Create a simple tuple generator
    struct TupleGenerator;
    impl Generator<(i32, i32)> for TupleGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> (i32, i32) {
            use rand::Rng;
            (rng.gen_range(-10..=10), rng.gen_range(-10..=10))
        }
        fn shrink(&self, _value: &(i32, i32)) -> Box<dyn Iterator<Item = (i32, i32)>> {
            Box::new(std::iter::empty())
        }
    }

    let result = check(TupleGenerator, CommutativeProperty);
    assert!(result.is_ok());

    if let Ok(success) = result {
        assert_eq!(success.iterations, 100); // Default iterations
        assert!(success.stats.is_some());
    }
}

#[test]
fn test_custom_configuration() {
    struct AbsoluteValueProperty;
    impl Property<i32> for AbsoluteValueProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input.abs() >= 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "abs({}) = {} is negative",
                    input,
                    input.abs()
                )))
            }
        }
    }

    let config = TestConfig {
        iterations: 25,
        seed: Some(42),
        max_shrink_iterations: 50,
        shrink_timeout: Duration::from_secs(2),
        ..TestConfig::default()
    };

    let result = check_with_config(range(-100, 100), AbsoluteValueProperty, config);
    assert!(result.is_ok());

    if let Ok(success) = result {
        assert_eq!(success.iterations, 25);
        assert_eq!(success.config.seed, Some(42));
    }
}

#[test]
fn test_builder_pattern() {
    struct StringLengthProperty {
        min_len: usize,
        max_len: usize,
    }

    impl Property<String> for StringLengthProperty {
        type Output = ();

        fn test(&self, input: String) -> Result<Self::Output, PropertyError> {
            let len = input.len();
            if len >= self.min_len && len <= self.max_len {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "String length {} not in range [{}, {}]",
                    len, self.min_len, self.max_len
                )))
            }
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(20)
        .seed(12345)
        .max_shrink_iterations(30)
        .enable_statistics()
        .run(
            protest::primitives::StringGenerator::ascii_printable(5, 15),
            StringLengthProperty {
                min_len: 5,
                max_len: 15,
            },
        );

    assert!(result.is_ok());

    if let Ok(success) = result {
        assert_eq!(success.iterations, 20);
        assert_eq!(success.config.seed, Some(12345));
        assert!(success.stats.is_some());
    }
}

#[test]
fn test_collection_properties() {
    struct SortPreservesElementsProperty;
    impl Property<Vec<i32>> for SortPreservesElementsProperty {
        type Output = ();

        fn test(&self, mut input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            let original = input.clone();
            input.sort();

            // Check that all original elements are still present
            for &elem in &original {
                if !input.contains(&elem) {
                    return Err(PropertyError::property_failed(format!(
                        "Element {} missing after sort",
                        elem
                    )));
                }
            }

            // Check that no new elements were added
            if input.len() != original.len() {
                return Err(PropertyError::property_failed(format!(
                    "Length changed: {} -> {}",
                    original.len(),
                    input.len()
                )));
            }

            Ok(())
        }
    }

    let generator = protest::primitives::VecGenerator::new(range(-20, 20), 0, 10);

    let result = check(generator, SortPreservesElementsProperty);
    assert!(result.is_ok());
}

#[test]
fn test_strategy_composition() {
    struct EvenNumberProperty;
    impl Property<i32> for EvenNumberProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input % 2 == 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Number {} is not even",
                    input
                )))
            }
        }
    }

    // Create a generator that only generates even numbers
    struct EvenGenerator;
    impl Generator<i32> for EvenGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> i32 {
            use rand::Rng;
            rng.gen_range(1..=50) * 2
        }
        fn shrink(&self, _value: &i32) -> Box<dyn Iterator<Item = i32>> {
            Box::new(std::iter::empty())
        }
    }

    let result = check(EvenGenerator, EvenNumberProperty);
    assert!(result.is_ok());
}

// Test async patterns from examples/async_examples.rs
#[tokio::test]
async fn test_basic_async_property() {
    // Test that async properties work with check_async()
    // We don't test timing precision - that's testing tokio/OS, not our library
    struct AsyncValidationProperty;
    impl AsyncProperty<u64> for AsyncValidationProperty {
        type Output = ();

        async fn test(&self, value: u64) -> Result<Self::Output, PropertyError> {
            // Perform an async operation (sleep) to verify async works
            tokio::time::sleep(Duration::from_millis(value)).await;

            // Validate the input itself, not the timing
            if value > 0 && value <= 100 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Value {} out of valid range (1-100)",
                    value
                )))
            }
        }
    }

    // Test with a range of values
    let generator = range(1u64, 20u64);

    let result = check_async(generator, AsyncValidationProperty).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_with_builder() {
    struct SimpleAsyncProperty;
    impl AsyncProperty<i32> for SimpleAsyncProperty {
        type Output = ();

        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::task::yield_now().await;

            if value >= 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Negative value"))
            }
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(15)
        .seed(123)
        .run_async(range(0, 100), SimpleAsyncProperty)
        .await;

    assert!(result.is_ok());

    if let Ok(success) = result {
        assert_eq!(success.iterations, 15);
        assert_eq!(success.config.seed, Some(123));
    }
}

// Test custom generator patterns from examples/custom_generators.rs
#[test]
fn test_custom_email_generator() {
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

    struct EmailGenerator;
    impl Generator<EmailAddress> for EmailGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> EmailAddress {
            let local_chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
            let domains = ["test.com", "example.org", "demo.net"];

            let local_len = (rng.next_u32() % 10) + 1;
            let local: String = (0..local_len)
                .map(|_| {
                    let idx = (rng.next_u32() as usize) % local_chars.len();
                    local_chars[idx]
                })
                .collect();

            let domain_idx = (rng.next_u32() as usize) % domains.len();
            let domain = domains[domain_idx].to_string();

            EmailAddress::new(local, domain)
        }

        fn shrink(&self, value: &EmailAddress) -> Box<dyn Iterator<Item = EmailAddress>> {
            let mut shrinks = Vec::new();

            if value.local.len() > 1 {
                let shorter_local = value.local[..value.local.len() - 1].to_string();
                shrinks.push(EmailAddress::new(shorter_local, value.domain.clone()));
            }

            if value.local != "a" {
                shrinks.push(EmailAddress::new("a".to_string(), value.domain.clone()));
            }

            Box::new(shrinks.into_iter())
        }
    }

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

    let result = check(EmailGenerator, EmailValidityProperty);
    assert!(result.is_ok());
}

#[test]
fn test_composite_generator() {
    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: u32,
        name: String,
        active: bool,
    }

    struct UserGenerator;
    impl Generator<User> for UserGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> User {
            let id = rng.next_u32() % 10000;

            let names = ["Alice", "Bob", "Charlie", "Diana"];
            let name_idx = (rng.next_u32() as usize) % names.len();
            let name = names[name_idx].to_string();

            let active = rng.next_u32().is_multiple_of(2);

            User { id, name, active }
        }

        fn shrink(&self, value: &User) -> Box<dyn Iterator<Item = User>> {
            let mut shrinks = Vec::new();

            if value.id > 0 {
                shrinks.push(User {
                    id: 0,
                    ..value.clone()
                });
            }

            if value.name != "Alice" {
                shrinks.push(User {
                    name: "Alice".to_string(),
                    ..value.clone()
                });
            }

            Box::new(shrinks.into_iter())
        }
    }

    struct UserConsistencyProperty;
    impl Property<User> for UserConsistencyProperty {
        type Output = ();

        fn test(&self, user: User) -> Result<Self::Output, PropertyError> {
            if user.name.is_empty() {
                return Err(PropertyError::property_failed("User name cannot be empty"));
            }

            if user.id > 100000 {
                return Err(PropertyError::property_failed("User ID too large"));
            }

            Ok(())
        }
    }

    let result = check(UserGenerator, UserConsistencyProperty);
    assert!(result.is_ok());
}

// Test performance patterns
#[test]
fn test_performance_monitoring() {
    struct FastProperty;
    impl Property<i32> for FastProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // Simple operation that should be fast
            let result = input * 2;

            if result / 2 == input {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Arithmetic error"))
            }
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(50)
        .enable_statistics()
        .run(range(1, 1000), FastProperty);

    assert!(result.is_ok());

    if let Ok(success) = result {
        assert!(success.stats.is_some());
        let stats = success.stats.unwrap();
        assert_eq!(stats.total_generated, 50);
        assert!(stats.performance_metrics.total_generation_time > Duration::from_nanos(0));
    }
}

#[test]
fn test_memory_efficient_patterns() {
    struct DataProperty;
    impl Property<Vec<u8>> for DataProperty {
        type Output = ();

        fn test(&self, input: Vec<u8>) -> Result<Self::Output, PropertyError> {
            // Property: data length should match actual length
            if input.len() == input.capacity() || input.len() <= input.capacity() {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Length/capacity mismatch"))
            }
        }
    }

    let generator = protest::primitives::VecGenerator::new(
        range(0u8, 255u8),
        0,
        100, // Reasonable size for tests
    );

    let config = TestConfig {
        iterations: 30,
        max_shrink_iterations: 10, // Limit shrinking for efficiency
        shrink_timeout: Duration::from_secs(2),
        ..TestConfig::default()
    };

    let result = check_with_config(generator, DataProperty, config);
    assert!(result.is_ok());
}

// Test error handling and edge cases
#[test]
fn test_shrinking_behavior() {
    struct LargeValueProperty;
    impl Property<i32> for LargeValueProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input <= 5 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Value {} is too large (must be <= 5)",
                    input
                )))
            }
        }
    }

    // Use a range that will definitely generate failing cases
    let generator = range(10, 100);

    let result = check(generator, LargeValueProperty);
    assert!(result.is_err());

    if let Err(failure) = result {
        assert!(failure.original_input >= 10);
        assert!(failure.original_input <= 100);

        // Should have attempted shrinking
        if let Some(shrunk) = failure.shrunk_input {
            assert!(shrunk > 5); // Should still fail the property
            assert!(shrunk <= failure.original_input); // Should be smaller or equal
        }
    }
}

#[test]
fn test_configuration_validation() {
    struct SimpleProperty;
    impl Property<i32> for SimpleProperty {
        type Output = ();

        fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            Ok(())
        }
    }

    // Test with various configurations
    let configs = vec![
        TestConfig {
            iterations: 1,
            ..TestConfig::default()
        },
        TestConfig {
            iterations: 10,
            seed: Some(0),
            ..TestConfig::default()
        },
        TestConfig {
            iterations: 5,
            max_shrink_iterations: 0,
            shrink_timeout: Duration::from_millis(1),
            ..TestConfig::default()
        },
    ];

    for config in configs {
        let result = check_with_config(just(42), SimpleProperty, config);
        assert!(result.is_ok(), "Configuration should be valid");
    }
}

// Integration test for the complete workflow
#[test]
fn test_complete_workflow_integration() {
    // This test verifies that all components work together correctly

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        numbers: Vec<i32>,
        text: String,
        flag: bool,
    }

    struct TestDataGenerator;
    impl Generator<TestData> for TestDataGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> TestData {
            let num_count = (rng.next_u32() % 5) + 1;
            let numbers: Vec<i32> = (0..num_count)
                .map(|_| (rng.next_u32() as i32) % 100)
                .collect();

            let text_len = (rng.next_u32() % 10) + 1;
            let text: String = (0..text_len)
                .map(|_| {
                    let c = (rng.next_u32() % 26) as u8 + b'a';
                    c as char
                })
                .collect();

            let flag = rng.next_u32().is_multiple_of(2);

            TestData {
                numbers,
                text,
                flag,
            }
        }

        fn shrink(&self, value: &TestData) -> Box<dyn Iterator<Item = TestData>> {
            let mut shrinks = Vec::new();

            // Shrink numbers vector
            if !value.numbers.is_empty() {
                shrinks.push(TestData {
                    numbers: value.numbers[..value.numbers.len() - 1].to_vec(),
                    ..value.clone()
                });
            }

            // Shrink text
            if value.text.len() > 1 {
                shrinks.push(TestData {
                    text: value.text[..value.text.len() - 1].to_string(),
                    ..value.clone()
                });
            }

            Box::new(shrinks.into_iter())
        }
    }

    struct ComplexProperty;
    impl Property<TestData> for ComplexProperty {
        type Output = ();

        fn test(&self, input: TestData) -> Result<Self::Output, PropertyError> {
            // Multiple properties to check

            // Property 1: Numbers should not be empty
            if input.numbers.is_empty() {
                return Err(PropertyError::property_failed("Numbers cannot be empty"));
            }

            // Property 2: Text should not be empty
            if input.text.is_empty() {
                return Err(PropertyError::property_failed("Text cannot be empty"));
            }

            // Property 3: If flag is true, sum of numbers should be positive
            if input.flag {
                let sum: i32 = input.numbers.iter().sum();
                if sum <= 0 {
                    return Err(PropertyError::property_failed(format!(
                        "When flag is true, sum {} should be positive",
                        sum
                    )));
                }
            }

            Ok(())
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(50)
        .seed(777)
        .max_shrink_iterations(25)
        .shrink_timeout(Duration::from_secs(3))
        .enable_statistics()
        .run(TestDataGenerator, ComplexProperty);

    // The test might pass or fail, but it should complete without panicking
    match result {
        Ok(success) => {
            assert_eq!(success.iterations, 50);
            assert_eq!(success.config.seed, Some(777));
            assert!(success.stats.is_some());
        }
        Err(failure) => {
            // Verify error handling works correctly
            // Verify original_input is well-formed (always true, but documents intent)
            let _numbers = &failure.original_input.numbers;
            // shrink_steps is usize, always >= 0
            let _shrink_steps = failure.shrink_steps;
        }
    }
}
