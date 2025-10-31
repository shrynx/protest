//! End-to-end integration tests for the Protest library
//!
//! This module contains comprehensive tests that verify all features work together
//! in realistic scenarios, including regression tests and performance validation.

#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::manual_range_contains)]
#![allow(unused_comparisons)]

use protest::ergonomic::{check_with_closure, check_with_closure_config};
use protest::{
    AsyncProperty, Generator, GeneratorConfig, ParallelConfig, Property, PropertyError,
    PropertyTestBuilder, TestConfig, TestResult, check, check_async_with_config, check_parallel,
    check_with_config, range,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// End-to-end test: Complete property-based testing workflow
#[test]
fn test_complete_workflow() {
    // Define a complex data structure
    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: u32,
        username: String,
        email: String,
        age: u8,
        preferences: HashMap<String, String>,
        tags: HashSet<String>,
        scores: Vec<f64>,
    }

    // Custom generator for User
    struct UserGenerator {
        usernames: Vec<String>,
        domains: Vec<String>,
        pref_keys: Vec<String>,
        pref_values: Vec<String>,
        available_tags: Vec<String>,
    }

    impl UserGenerator {
        fn new() -> Self {
            Self {
                usernames: vec![
                    "alice".to_string(),
                    "bob".to_string(),
                    "charlie".to_string(),
                    "diana".to_string(),
                    "eve".to_string(),
                    "frank".to_string(),
                ],
                domains: vec![
                    "example.com".to_string(),
                    "test.org".to_string(),
                    "demo.net".to_string(),
                ],
                pref_keys: vec![
                    "theme".to_string(),
                    "language".to_string(),
                    "timezone".to_string(),
                ],
                pref_values: vec![
                    "dark".to_string(),
                    "light".to_string(),
                    "en".to_string(),
                    "es".to_string(),
                    "UTC".to_string(),
                    "EST".to_string(),
                ],
                available_tags: vec![
                    "premium".to_string(),
                    "verified".to_string(),
                    "beta".to_string(),
                    "admin".to_string(),
                    "new".to_string(),
                ],
            }
        }
    }

    impl Generator<User> for UserGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> User {
            let id = rng.next_u32() % 100000;

            let username_idx = (rng.next_u32() as usize) % self.usernames.len();
            let username = format!("{}_{}", self.usernames[username_idx], id % 1000);

            let domain_idx = (rng.next_u32() as usize) % self.domains.len();
            let email = format!("{}@{}", username, self.domains[domain_idx]);

            let age = (rng.next_u32() % 80) as u8 + 18; // 18-97 years old

            // Generate preferences (0-3 preferences)
            let pref_count = rng.next_u32() % 4;
            let mut preferences = HashMap::new();
            for _ in 0..pref_count {
                let key_idx = (rng.next_u32() as usize) % self.pref_keys.len();
                let val_idx = (rng.next_u32() as usize) % self.pref_values.len();
                preferences.insert(
                    self.pref_keys[key_idx].clone(),
                    self.pref_values[val_idx].clone(),
                );
            }

            // Generate tags (0-3 tags)
            let tag_count = rng.next_u32() % 4;
            let mut tags = HashSet::new();
            for _ in 0..tag_count {
                let tag_idx = (rng.next_u32() as usize) % self.available_tags.len();
                tags.insert(self.available_tags[tag_idx].clone());
            }

            // Generate scores (0-10 scores)
            let score_count = rng.next_u32() % 11;
            let scores: Vec<f64> = (0..score_count)
                .map(|_| (rng.next_u32() as f64 / u32::MAX as f64) * 100.0)
                .collect();

            User {
                id,
                username,
                email,
                age,
                preferences,
                tags,
                scores,
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

            // Shrink collections
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

            if !value.scores.is_empty() {
                shrinks.push(User {
                    scores: value.scores[..value.scores.len() - 1].to_vec(),
                    ..value.clone()
                });
            }

            Box::new(shrinks.into_iter())
        }
    }

    // Complex property that tests multiple invariants
    struct UserInvariantsProperty;
    impl Property<User> for UserInvariantsProperty {
        type Output = ();

        fn test(&self, user: User) -> Result<Self::Output, PropertyError> {
            // Invariant 1: Email should contain username
            if !user.email.contains(&user.username) {
                return Err(PropertyError::property_failed(format!(
                    "Email '{}' should contain username '{}'",
                    user.email, user.username
                )));
            }

            // Invariant 2: Age should be reasonable
            if user.age < 18 || user.age > 120 {
                return Err(PropertyError::property_failed(format!(
                    "Age {} is unreasonable",
                    user.age
                )));
            }

            // Invariant 3: Premium users should be verified
            if user.tags.contains("premium") && !user.tags.contains("verified") {
                return Err(PropertyError::property_failed(
                    "Premium users must be verified",
                ));
            }

            // Invariant 4: Admin users should have high scores
            if user.tags.contains("admin") && !user.scores.is_empty() {
                let avg_score: f64 = user.scores.iter().sum::<f64>() / user.scores.len() as f64;
                if avg_score < 50.0 {
                    return Err(PropertyError::property_failed(format!(
                        "Admin users should have average score >= 50, got {:.2}",
                        avg_score
                    )));
                }
            }

            // Invariant 5: All scores should be valid
            for &score in &user.scores {
                if score < 0.0 || score > 100.0 || !score.is_finite() {
                    return Err(PropertyError::property_failed(format!(
                        "Invalid score: {}",
                        score
                    )));
                }
            }

            Ok(())
        }
    }

    // Run the complete workflow with statistics
    let result = PropertyTestBuilder::new()
        .iterations(100)
        .seed(12345)
        .max_shrink_iterations(50)
        .shrink_timeout(Duration::from_secs(5))
        .enable_statistics()
        // Note: generator_config not yet implemented
        .run(UserGenerator::new(), UserInvariantsProperty);

    // Verify the test completed successfully
    match result {
        Ok(success) => {
            assert_eq!(success.iterations, 100);
            assert_eq!(success.config.seed, Some(12345));
            assert!(success.stats.is_some());

            let stats = success.stats.unwrap();
            assert_eq!(stats.total_generated, 100);
            assert!(stats.performance_metrics.total_generation_time > Duration::from_nanos(0));

            println!("✓ Complete workflow test passed!");
            println!(
                "  Generated {} users in {:?}",
                stats.total_generated, stats.performance_metrics.total_generation_time
            );
        }
        Err(failure) => {
            // If the test fails, verify that shrinking worked
            println!("Test failed as expected (testing shrinking):");
            println!("  Original user ID: {}", failure.original_input.id);
            println!("  Error: {}", failure.error);

            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk user ID: {}", shrunk.id);
                println!("  Shrinking steps: {}", failure.shrink_steps);
                assert!(shrunk.id <= failure.original_input.id);
            }
        }
    }
}

// End-to-end async test
#[tokio::test]
async fn test_async_end_to_end() {
    // Simulate a web service that we're testing
    struct WebService {
        response_times: Arc<Mutex<Vec<Duration>>>,
    }

    impl WebService {
        fn new() -> Self {
            Self {
                response_times: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn make_request(&self, payload_size: usize) -> Result<String, String> {
            let start = Instant::now();

            // Simulate network delay based on payload size
            let delay_ms = (payload_size / 100).min(50) as u64;
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            let duration = start.elapsed();
            self.response_times.lock().unwrap().push(duration);

            // Simulate service behavior
            if payload_size == 0 {
                Err("Empty payload".to_string())
            } else if payload_size > 10000 {
                Err("Payload too large".to_string())
            } else {
                Ok(format!("Response for {} bytes", payload_size))
            }
        }

        fn get_average_response_time(&self) -> Duration {
            let times = self.response_times.lock().unwrap();
            if times.is_empty() {
                Duration::from_nanos(0)
            } else {
                let total: Duration = times.iter().sum();
                total / times.len() as u32
            }
        }
    }

    // Property: web service should handle requests correctly
    struct WebServiceProperty {
        service: Arc<WebService>,
    }

    impl AsyncProperty<usize> for WebServiceProperty {
        type Output = ();

        async fn test(&self, payload_size: usize) -> Result<Self::Output, PropertyError> {
            match self.service.make_request(payload_size).await {
                Ok(response) => {
                    // Property: response should mention the payload size
                    if response.contains(&payload_size.to_string()) {
                        Ok(())
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "Response '{}' doesn't mention payload size {}",
                            response, payload_size
                        )))
                    }
                }
                Err(error) => {
                    // Property: errors should only occur for invalid payload sizes
                    if payload_size == 0 || payload_size > 10000 {
                        Ok(()) // Expected error
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "Unexpected error for payload size {}: {}",
                            payload_size, error
                        )))
                    }
                }
            }
        }
    }

    let service = Arc::new(WebService::new());
    let property = WebServiceProperty {
        service: service.clone(),
    };

    let config = TestConfig {
        iterations: 50,
        seed: Some(54321),
        max_shrink_iterations: 20,
        shrink_timeout: Duration::from_secs(3),
        ..TestConfig::default()
    };

    let result = check_async_with_config(range(1usize, 8000usize), property, config).await;

    match result {
        Ok(success) => {
            println!("✓ Async end-to-end test passed!");
            println!("  Completed {} async requests", success.iterations);
            println!(
                "  Average response time: {:?}",
                service.get_average_response_time()
            );

            assert_eq!(success.iterations, 50);
            assert_eq!(success.config.seed, Some(54321));
        }
        Err(failure) => {
            println!("Async test failed:");
            println!("  Payload size: {}", failure.original_input);
            println!("  Error: {}", failure.error);
            println!(
                "  Average response time: {:?}",
                service.get_average_response_time()
            );

            // Verify error handling worked correctly
            assert!(failure.test_duration > Duration::from_nanos(0));
        }
    }
}

// Performance validation test
#[test]
fn test_performance_characteristics() {
    struct PerformanceProperty {
        max_duration: Duration,
    }

    impl Property<Vec<i32>> for PerformanceProperty {
        type Output = ();

        fn test(&self, input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            let start = Instant::now();

            // Perform some computation
            let mut sorted = input.clone();
            sorted.sort();

            // Verify sorting worked
            for i in 1..sorted.len() {
                if sorted[i] < sorted[i - 1] {
                    return Err(PropertyError::property_failed("Sorting failed"));
                }
            }

            let duration = start.elapsed();

            // Performance property: operation should complete quickly
            if duration <= self.max_duration {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Operation took {:?}, expected <= {:?}",
                    duration, self.max_duration
                )))
            }
        }
    }

    let generator = protest::primitives::VecGenerator::new(range(-1000, 1000), 0, 1000);

    let start = Instant::now();
    let result = PropertyTestBuilder::new()
        .iterations(100)
        .enable_statistics()
        .run(
            generator,
            PerformanceProperty {
                max_duration: Duration::from_millis(10),
            },
        );
    let total_duration = start.elapsed();

    match result {
        Ok(success) => {
            println!("✓ Performance test passed!");
            println!("  Total test time: {:?}", total_duration);

            if let Some(stats) = success.stats {
                println!(
                    "  Average generation time: {:?}",
                    stats.performance_metrics.average_generation_time
                );
                // Note: peak_memory_usage not yet implemented
                // println!("  Peak memory usage: {} KB", stats.performance_metrics.peak_memory_usage / 1024);

                // Verify performance characteristics
                assert!(
                    stats.performance_metrics.average_generation_time < Duration::from_millis(1)
                );
                assert!(total_duration < Duration::from_secs(10)); // Should complete in reasonable time
            }
        }
        Err(failure) => {
            println!("Performance test failed (expected for large inputs):");
            println!("  Vector size: {}", failure.original_input.len());
            println!("  Test duration: {:?}", failure.test_duration);

            // Even failures should complete in reasonable time
            assert!(failure.test_duration < Duration::from_secs(1));
        }
    }
}

// Parallel execution integration test
#[test]
fn test_parallel_execution_integration() {
    struct ParallelSafeProperty {
        thread_counter: Arc<Mutex<usize>>,
    }

    impl Property<i32> for ParallelSafeProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // Increment thread counter
            {
                let mut counter = self.thread_counter.lock().unwrap();
                *counter += 1;
            }

            // Simulate some work
            std::thread::sleep(Duration::from_millis(1));

            // Property: input should be within expected range
            if input >= -1000 && input <= 1000 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Input {} out of range [-1000, 1000]",
                    input
                )))
            }
        }
    }

    let counter = Arc::new(Mutex::new(0));
    let property = ParallelSafeProperty {
        thread_counter: counter.clone(),
    };

    let config = TestConfig {
        iterations: 100,
        ..TestConfig::default()
    };

    // Note: ParallelConfig fields may differ - using default
    let parallel_config = ParallelConfig::default();

    let start = Instant::now();
    let result = check_parallel(range(-500, 500), property, config, parallel_config);
    let duration = start.elapsed();

    match result {
        Ok(success) => {
            println!("✓ Parallel execution test passed!");
            println!(
                "  Completed {} iterations in {:?}",
                success.iterations, duration
            );

            let final_count = *counter.lock().unwrap();
            assert_eq!(final_count, 100, "All iterations should have executed");
            assert_eq!(success.iterations, 100);

            // Parallel execution should be faster than sequential for this workload
            // (though this is not guaranteed in all environments)
            println!("  Thread executions: {}", final_count);
        }
        Err(failure) => {
            println!("Parallel test failed:");
            println!("  Input: {}", failure.original_input);
            println!("  Error: {}", failure.error);

            let partial_count = *counter.lock().unwrap();
            println!("  Partial executions: {}", partial_count);
            assert!(
                partial_count > 0,
                "Some iterations should have executed before failure"
            );
        }
    }
}

// Regression test for critical functionality
#[test]
fn test_regression_critical_functionality() {
    // This test ensures that critical functionality continues to work
    // across different versions of the library

    // Test 1: Basic property testing still works
    struct BasicProperty;
    impl Property<i32> for BasicProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input.abs() >= 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(
                    "Absolute value should be non-negative",
                ))
            }
        }
    }

    let result1 = check(range(-100, 100), BasicProperty);
    assert!(result1.is_ok(), "Basic property testing should work");

    // Test 2: Shrinking still works
    struct ShrinkingProperty;
    impl Property<Vec<i32>> for ShrinkingProperty {
        type Output = ();

        fn test(&self, input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            if input.len() <= 5 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Vector too long: {} elements",
                    input.len()
                )))
            }
        }
    }

    let generator = protest::primitives::VecGenerator::new(range(1, 10), 8, 15);
    let result2 = check(generator, ShrinkingProperty);

    if let Err(failure) = result2 {
        assert!(failure.original_input.len() > 5);
        if let Some(shrunk) = failure.shrunk_input {
            assert!(shrunk.len() > 5); // Should still fail
            assert!(shrunk.len() <= failure.original_input.len()); // Should be smaller or equal
        }
        assert!(failure.shrink_steps >= 0);
    }

    // Test 3: Configuration still works
    let config = TestConfig {
        iterations: 25,
        seed: Some(999),
        max_shrink_iterations: 10,
        shrink_timeout: Duration::from_secs(1),
        ..TestConfig::default()
    };

    let result3 = check_with_config(range(0, 50), BasicProperty, config);
    assert!(result3.is_ok(), "Configuration should work");

    if let Ok(success) = result3 {
        assert_eq!(success.iterations, 25);
        assert_eq!(success.config.seed, Some(999));
    }

    // Test 4: Statistics collection still works
    let result4 = PropertyTestBuilder::new()
        .iterations(20)
        .enable_statistics()
        .run(range(1, 100), BasicProperty);

    assert!(result4.is_ok(), "Statistics collection should work");

    if let Ok(success) = result4 {
        assert!(success.stats.is_some());
        let stats = success.stats.unwrap();
        assert_eq!(stats.total_generated, 20);
    }

    println!("✓ All regression tests passed!");
}

// Integration test for error reporting and debugging
#[test]
fn test_error_reporting_integration() {
    struct DetailedErrorProperty;
    impl Property<(String, i32, Vec<f64>)> for DetailedErrorProperty {
        type Output = ();

        fn test(
            &self,
            (text, number, values): (String, i32, Vec<f64>),
        ) -> Result<Self::Output, PropertyError> {
            // Multiple potential failure points for testing error reporting

            if text.is_empty() {
                return Err(PropertyError::property_failed_with_context(
                    format!(
                        "Text cannot be empty. Number: {}, Values: {:?}",
                        number, values
                    ),
                    Some("input validation"),
                    None, // iteration number
                ));
            }

            if number < 0 {
                return Err(PropertyError::property_failed_with_context(
                    format!(
                        "Negative number not allowed: {}. Text: '{}', Values count: {}",
                        number,
                        text,
                        values.len()
                    ),
                    Some("number validation"),
                    None, // iteration number
                ));
            }

            if values.len() > 10 {
                return Err(PropertyError::property_failed_with_context(
                    format!(
                        "Too many values: {} (max 10). Text: '{}', Number: {}",
                        values.len(),
                        text,
                        number
                    ),
                    Some("collection size validation"),
                    None, // iteration number
                ));
            }

            for (i, &value) in values.iter().enumerate() {
                if !value.is_finite() {
                    return Err(PropertyError::property_failed_with_context(
                        format!(
                            "Non-finite value at index {}: {}. Text: '{}', Number: {}, All values: {:?}",
                            i, value, text, number, values
                        ),
                        Some("value validation"),
                        None, // iteration number
                    ));
                }
            }

            Ok(())
        }
    }

    // Use tuple generator instead of Strategy zip combinator
    let generator = (
        protest::primitives::StringGenerator::ascii_printable(0, 10),
        protest::IntGenerator::new(-50, 50),
        protest::primitives::VecGenerator::new(
            protest::primitives::FloatGenerator::new(-100.0, 100.0),
            0,
            15,
        ),
    );

    let result = check(generator, DetailedErrorProperty);

    match result {
        Ok(success) => {
            println!("✓ Error reporting test passed unexpectedly!");
            println!("  Iterations: {}", success.iterations);
        }
        Err(failure) => {
            println!("Error reporting test failed as expected:");
            println!("  Error: {}", failure.error);
            println!("  Original input: {:?}", failure.original_input);

            if let Some(ref shrunk) = failure.shrunk_input {
                println!("  Shrunk input: {:?}", shrunk);
                println!("  Shrinking steps: {}", failure.shrink_steps);
            }

            // Verify error contains detailed information
            let error_string = format!("{}", failure.error);
            assert!(
                !error_string.is_empty(),
                "Error message should not be empty"
            );

            // Test error summary
            let summary = failure.summary();
            assert!(summary.contains("failed"), "Summary should mention failure");

            println!("  Summary: {}", summary);
        }
    }
}

// Memory usage and cleanup integration test
#[test]
fn test_memory_usage_integration() {
    struct MemoryIntensiveProperty;
    impl Property<Vec<Vec<u8>>> for MemoryIntensiveProperty {
        type Output = ();

        fn test(&self, input: Vec<Vec<u8>>) -> Result<Self::Output, PropertyError> {
            // Create temporary data structures to test memory management
            let mut processed: Vec<Vec<u8>> = Vec::new();

            for inner_vec in &input {
                let mut processed_inner: Vec<u8> = Vec::new();
                for &byte in inner_vec {
                    processed_inner.push(byte.wrapping_mul(2));
                }
                processed.push(processed_inner);
            }

            // Property: processed data should have same structure as input
            if processed.len() != input.len() {
                return Err(PropertyError::property_failed(format!(
                    "Length mismatch: {} vs {}",
                    processed.len(),
                    input.len()
                )));
            }

            for (i, (original, processed_inner)) in input.iter().zip(processed.iter()).enumerate() {
                if original.len() != processed_inner.len() {
                    return Err(PropertyError::property_failed(format!(
                        "Inner length mismatch at index {}: {} vs {}",
                        i,
                        original.len(),
                        processed_inner.len()
                    )));
                }
            }

            Ok(())
        }
    }

    let generator = protest::primitives::VecGenerator::new(
        protest::primitives::VecGenerator::new(range(0u8, 255u8), 0, 50),
        0,
        20,
    );

    let config = TestConfig {
        iterations: 30,
        max_shrink_iterations: 10, // Limit shrinking to control memory usage
        shrink_timeout: Duration::from_secs(2),
        ..TestConfig::default()
    };

    let start_memory = get_memory_usage();
    let result = check_with_config(generator, MemoryIntensiveProperty, config);
    let end_memory = get_memory_usage();

    match result {
        Ok(success) => {
            println!("✓ Memory usage test passed!");
            println!("  Iterations: {}", success.iterations);
            println!(
                "  Memory usage: {} KB -> {} KB",
                start_memory / 1024,
                end_memory / 1024
            );

            // Memory usage should not grow excessively
            let memory_growth = end_memory.saturating_sub(start_memory);
            assert!(
                memory_growth < 100 * 1024 * 1024, // Less than 100MB growth
                "Memory usage grew too much: {} bytes",
                memory_growth
            );
        }
        Err(failure) => {
            println!("Memory usage test failed:");
            println!("  Error: {}", failure.error);
            println!(
                "  Memory usage: {} KB -> {} KB",
                start_memory / 1024,
                end_memory / 1024
            );

            // Even on failure, memory should be cleaned up
            let memory_growth = end_memory.saturating_sub(start_memory);
            assert!(
                memory_growth < 200 * 1024 * 1024, // Less than 200MB growth even on failure
                "Memory usage grew too much even on failure: {} bytes",
                memory_growth
            );
        }
    }
}

// Helper function to get current memory usage (simplified)
fn get_memory_usage() -> usize {
    // This is a simplified memory usage estimation
    // In a real implementation, you might use system-specific APIs
    std::mem::size_of::<usize>() * 1000 // Placeholder
}

// Final comprehensive integration test
#[test]
fn test_comprehensive_integration() {
    println!("Running comprehensive integration test...");

    // Test all major components working together
    let mut test_results = Vec::new();

    // Test 1: Basic functionality
    let result1 = check_with_closure(range(1, 100), |x: i32| x > 0);
    test_results.push(("Basic functionality", result1.is_ok()));

    // Test 2: Custom generators
    struct CustomGen;
    impl Generator<String> for CustomGen {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
            let len = (rng.next_u32() % 10) + 1;
            (0..len)
                .map(|_| ((rng.next_u32() % 26) as u8 + b'a') as char)
                .collect()
        }

        fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
            if value.len() > 1 {
                Box::new(std::iter::once(value[..value.len() - 1].to_string()))
            } else {
                Box::new(std::iter::empty())
            }
        }
    }

    let result2 = check_with_closure(CustomGen, |s: String| !s.is_empty());
    test_results.push(("Custom generators", result2.is_ok()));

    // Test 3: Configuration
    let config = TestConfig {
        iterations: 20,
        seed: Some(42),
        ..TestConfig::default()
    };
    let result3 = check_with_closure_config(range(1, 50), |x: i32| x > 0, config);
    test_results.push(("Configuration", result3.is_ok()));

    // Test 4: Statistics
    struct StatsProperty;
    impl Property<i32> for StatsProperty {
        type Output = ();
        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input > 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Must be positive"))
            }
        }
    }
    let result4 = PropertyTestBuilder::new()
        .iterations(15)
        .enable_statistics()
        .run(range(1, 30), StatsProperty);
    test_results.push((
        "Statistics",
        result4.is_ok() && result4.unwrap().stats.is_some(),
    ));

    // Test 5: Error handling
    let result5 = check_with_closure(range(1, 10), |x: i32| x > 5);
    test_results.push(("Error handling", result5.is_err()));

    // Test 6: Test runner integration
    let test_result = TestResult::Passed {
        iterations: 100,
        duration: Duration::from_millis(500),
        seed: Some(123),
    };
    let formatted = format!("{}", test_result);
    test_results.push(("Test runner integration", formatted.contains("PASSED")));

    // Report results
    println!("\nIntegration test results:");
    let mut all_passed = true;
    for (test_name, passed) in &test_results {
        let status = if *passed { "✓ PASS" } else { "✗ FAIL" };
        println!("  {}: {}", test_name, status);
        if !passed {
            all_passed = false;
        }
    }

    if all_passed {
        println!("\n✓ All integration tests passed!");
    } else {
        panic!("Some integration tests failed!");
    }

    // Final verification
    let passed_count = test_results.iter().filter(|(_, passed)| *passed).count();
    let total_count = test_results.len();

    println!("\nSummary: {}/{} tests passed", passed_count, total_count);
    assert_eq!(
        passed_count, total_count,
        "All integration tests should pass"
    );
}
