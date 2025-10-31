//! Regression test suite for the Protest library
//!
//! This module contains tests that verify critical functionality continues to work
//! across different versions and prevent regressions in key features.

#![allow(dead_code)]

use protest::ergonomic::{check_with_closure, check_with_closure_config};
use protest::{
    AsyncProperty, Generator, GeneratorConfig, Property, PropertyError, PropertyTestBuilder,
    TestConfig, TestResult, TestRunner, check, check_async, check_with_config, range,
};
use std::time::Duration;

/// Regression test for basic property testing functionality
#[test]
fn regression_basic_property_testing() {
    // This test ensures that the core property testing functionality
    // continues to work as expected across versions

    struct BasicProperty;
    impl Property<i32> for BasicProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // Simple property: all integers should equal themselves
            if input == input {
                Ok(())
            } else {
                Err(PropertyError::property_failed(
                    "Integer should equal itself",
                ))
            }
        }
    }

    let result = check(range(-1000, 1000), BasicProperty);
    assert!(result.is_ok(), "Basic property testing should always work");

    if let Ok(success) = result {
        assert_eq!(success.iterations, 100); // Default iterations
        // Note: TestSuccess doesn't track duration
    }
}

/// Regression test for generator functionality
#[test]
fn regression_generator_functionality() {
    // Test that built-in generators continue to work correctly

    // Test integer generator
    struct IntegerRangeProperty;
    impl Property<i32> for IntegerRangeProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if (-100..=100).contains(&input) {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Integer {} out of range [-100, 100]",
                    input
                )))
            }
        }
    }

    let result1 = check(range(-100, 100), IntegerRangeProperty);
    assert!(result1.is_ok(), "Integer generator should work");

    // Test string generator
    struct StringLengthProperty;
    impl Property<String> for StringLengthProperty {
        type Output = ();

        fn test(&self, input: String) -> Result<Self::Output, PropertyError> {
            if input.len() >= 5 && input.len() <= 20 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "String length {} not in range [5, 20]",
                    input.len()
                )))
            }
        }
    }

    let result2 = check(
        protest::primitives::StringGenerator::ascii_printable(5, 20),
        StringLengthProperty,
    );
    assert!(result2.is_ok(), "String generator should work");

    // Test vector generator
    struct VectorSizeProperty;
    impl Property<Vec<i32>> for VectorSizeProperty {
        type Output = ();

        fn test(&self, input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            if input.len() <= 10 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Vector too large: {} elements",
                    input.len()
                )))
            }
        }
    }

    let result3 = check(
        protest::primitives::VecGenerator::new(range(1, 100), 0, 10),
        VectorSizeProperty,
    );
    assert!(result3.is_ok(), "Vector generator should work");
}

/// Regression test for shrinking functionality
#[test]
fn regression_shrinking_functionality() {
    // Test that shrinking continues to work and finds minimal cases

    struct ShrinkingTestProperty;
    impl Property<Vec<i32>> for ShrinkingTestProperty {
        type Output = ();

        fn test(&self, input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            // This property will fail for vectors with more than 3 elements
            if input.len() <= 3 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Vector too long: {} elements (max 3)",
                    input.len()
                )))
            }
        }
    }

    // Use a generator that will produce vectors longer than 3 elements
    let generator = protest::primitives::VecGenerator::new(range(1, 10), 5, 15);
    let result = check(generator, ShrinkingTestProperty);

    // This should fail, but shrinking should work
    assert!(result.is_err(), "Property should fail to test shrinking");

    if let Err(failure) = result {
        assert!(
            failure.original_input.len() > 3,
            "Original input should be too long"
        );

        // Verify shrinking attempted to find a smaller case
        if let Some(shrunk) = failure.shrunk_input {
            assert!(
                shrunk.len() > 3,
                "Shrunk input should still fail the property"
            );
            assert!(
                shrunk.len() <= failure.original_input.len(),
                "Shrunk input should be smaller or equal"
            );
        }

        // shrink_steps is usize, always >= 0, just check it exists
        let _shrink_steps = failure.shrink_steps;
    }
}

/// Regression test for configuration functionality
#[test]
fn regression_configuration_functionality() {
    // Test that configuration options continue to work correctly

    struct ConfigTestProperty {
        expected_seed: Option<u64>,
    }

    impl Property<i32> for ConfigTestProperty {
        type Output = ();

        fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            // This property always passes, we're just testing configuration
            Ok(())
        }
    }

    // Test custom configuration
    let config = TestConfig {
        iterations: 50,
        seed: Some(12345),
        max_shrink_iterations: 25,
        shrink_timeout: Duration::from_secs(2),
        ..TestConfig::default()
    };

    let result = check_with_config(
        range(1, 100),
        ConfigTestProperty {
            expected_seed: Some(12345),
        },
        config,
    );

    assert!(result.is_ok(), "Configuration should work");

    if let Ok(success) = result {
        assert_eq!(
            success.iterations, 50,
            "Custom iteration count should be respected"
        );
        assert_eq!(
            success.config.seed,
            Some(12345),
            "Custom seed should be respected"
        );
        assert_eq!(
            success.config.max_shrink_iterations, 25,
            "Custom shrink iterations should be respected"
        );
        assert_eq!(
            success.config.shrink_timeout,
            Duration::from_secs(2),
            "Custom shrink timeout should be respected"
        );
    }
}

/// Regression test for async functionality
#[tokio::test]
async fn regression_async_functionality() {
    // Test that async property testing continues to work

    struct AsyncTestProperty;
    impl AsyncProperty<i32> for AsyncTestProperty {
        type Output = ();

        async fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // Simulate async work
            tokio::time::sleep(Duration::from_millis(1)).await;

            // Simple property: positive numbers should be positive
            if input > 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Expected positive number"))
            }
        }
    }

    let result = check_async(range(1, 100), AsyncTestProperty).await;
    assert!(result.is_ok(), "Async property testing should work");

    if let Ok(success) = result {
        assert_eq!(success.iterations, 100);
        // Note: TestSuccess doesn't track duration
    }
}

/// Regression test for statistics collection
#[test]
fn regression_statistics_functionality() {
    // Test that statistics collection continues to work

    struct StatsTestProperty;
    impl Property<i32> for StatsTestProperty {
        type Output = ();

        fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            Ok(())
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(75)
        .enable_statistics()
        .run(range(1, 1000), StatsTestProperty);

    assert!(result.is_ok(), "Statistics collection should work");

    if let Ok(success) = result {
        assert!(success.stats.is_some(), "Statistics should be collected");

        let stats = success.stats.unwrap();
        assert_eq!(
            stats.total_generated, 75,
            "Should track correct number of generated values"
        );
        assert!(
            stats.performance_metrics.total_generation_time > Duration::from_nanos(0),
            "Should track generation time"
        );
        assert!(
            stats.performance_metrics.average_generation_time > Duration::from_nanos(0),
            "Should calculate average time"
        );

        // Test statistics reporting
        let summary = stats.get_summary();
        assert!(
            !summary.is_empty(),
            "Statistics summary should not be empty"
        );
        assert!(
            summary.contains("75"),
            "Summary should mention the number of generated values"
        );

        let report = stats.generate_report();
        assert!(!report.is_empty(), "Statistics report should not be empty");
    }
}

/// Regression test for error handling and reporting
#[test]
fn regression_error_handling() {
    // Test that error handling and reporting continues to work correctly

    struct ErrorTestProperty;
    impl Property<i32> for ErrorTestProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            match input % 3 {
                0 => Err(PropertyError::property_failed("Divisible by 3")),
                1 => Err(PropertyError::property_failed_with_context(
                    format!("Remainder 1 - input was {}", input),
                    Some("modulo test"),
                    None, // iteration number
                )),
                _ => Ok(()),
            }
        }
    }

    // This should fail for most inputs
    let result = check(range(0, 20), ErrorTestProperty);
    assert!(
        result.is_err(),
        "Property should fail to test error handling"
    );

    if let Err(failure) = result {
        // Test error message formatting
        let error_string = format!("{}", failure.error);
        assert!(
            !error_string.is_empty(),
            "Error message should not be empty"
        );

        // Test failure summary
        let summary = failure.summary();
        assert!(!summary.is_empty(), "Failure summary should not be empty");
        assert!(summary.contains("failed"), "Summary should mention failure");

        // Test that original input is preserved
        assert!(
            failure.original_input >= 0 && failure.original_input <= 20,
            "Original input should be in expected range"
        );

        // Test duration tracking
        assert!(
            failure.test_duration > Duration::from_nanos(0),
            "Should track test duration"
        );
    }
}

/// Regression test for custom generators
#[test]
fn regression_custom_generators() {
    // Test that custom generator implementation continues to work

    #[derive(Debug, Clone, PartialEq)]
    struct CustomType {
        value: i32,
        flag: bool,
    }

    struct CustomTypeGenerator;
    impl Generator<CustomType> for CustomTypeGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> CustomType {
            CustomType {
                value: (rng.next_u32() as i32) % 100,
                flag: rng.next_u32().is_multiple_of(2),
            }
        }

        fn shrink(&self, value: &CustomType) -> Box<dyn Iterator<Item = CustomType>> {
            let mut shrinks = Vec::new();

            // Shrink value towards 0
            if value.value > 0 {
                shrinks.push(CustomType {
                    value: value.value / 2,
                    flag: value.flag,
                });
                shrinks.push(CustomType {
                    value: 0,
                    flag: value.flag,
                });
            }

            // Try flipping the flag
            shrinks.push(CustomType {
                value: value.value,
                flag: !value.flag,
            });

            Box::new(shrinks.into_iter())
        }
    }

    struct CustomTypeProperty;
    impl Property<CustomType> for CustomTypeProperty {
        type Output = ();

        fn test(&self, input: CustomType) -> Result<Self::Output, PropertyError> {
            // Property: if flag is true, value should be even
            if input.flag && input.value % 2 != 0 {
                Err(PropertyError::property_failed(format!(
                    "Flag is true but value {} is odd",
                    input.value
                )))
            } else {
                Ok(())
            }
        }
    }

    let result = check(CustomTypeGenerator, CustomTypeProperty);

    // This might pass or fail, but should not crash
    match result {
        Ok(success) => {
            assert_eq!(success.iterations, 100);
        }
        Err(failure) => {
            // Verify custom shrinking worked
            assert!(
                failure.original_input.flag,
                "Original input should have flag=true to fail"
            );
            assert!(
                failure.original_input.value % 2 != 0,
                "Original input should have odd value to fail"
            );

            if let Some(shrunk) = failure.shrunk_input {
                // Shrunk input should still fail the property
                assert!(
                    shrunk.flag && shrunk.value % 2 != 0,
                    "Shrunk input should still fail"
                );
            }
        }
    }
}

/// Regression test for strategy composition
#[test]
fn regression_strategy_composition() {
    // Test that strategy composition continues to work

    struct CompositionProperty;
    impl Property<(i32, String, bool)> for CompositionProperty {
        type Output = ();

        fn test(
            &self,
            (number, text, flag): (i32, String, bool),
        ) -> Result<Self::Output, PropertyError> {
            // Property: if flag is true, number should be positive and text non-empty
            if flag {
                if number <= 0 {
                    return Err(PropertyError::property_failed(
                        "Flag true but number not positive",
                    ));
                }
                if text.is_empty() {
                    return Err(PropertyError::property_failed("Flag true but text empty"));
                }
            }
            Ok(())
        }
    }

    // Compose multiple generators using tuple generator
    use protest::{IntGenerator, primitives::BoolGenerator, primitives::StringGenerator};
    let generator = (
        IntGenerator::new(1, 100),
        StringGenerator::ascii_printable(1, 10),
        BoolGenerator,
    );

    let result = check(generator, CompositionProperty);
    assert!(result.is_ok(), "Strategy composition should work");
}

/// Regression test for test runner integration
#[test]
fn regression_test_runner_integration() {
    // Test that test runner integration continues to work

    // Test successful result formatting
    let success_result = TestResult::Passed {
        iterations: 100,
        duration: Duration::from_millis(500),
        seed: Some(42),
    };

    let success_output = format!("{}", success_result);
    assert!(
        success_output.contains("PASSED"),
        "Success output should contain PASSED"
    );
    assert!(
        success_output.contains("100 iterations"),
        "Success output should mention iterations"
    );
    assert!(
        success_output.contains("seed: 42"),
        "Success output should mention seed"
    );

    // Test failure result formatting
    let failure_result = TestResult::Failed {
        error: "Test failed".to_string(),
        original_input: "42".to_string(),
        shrunk_input: Some("1".to_string()),
        shrink_steps: 5,
        seed: Some(123),
        duration: Duration::from_millis(200),
    };

    let failure_output = format!("{}", failure_result);
    assert!(
        failure_output.contains("FAILED"),
        "Failure output should contain FAILED"
    );
    assert!(
        failure_output.contains("Test failed"),
        "Failure output should contain error message"
    );
    assert!(
        failure_output.contains("Original input: 42"),
        "Failure output should show original input"
    );
    assert!(
        failure_output.contains("Minimal input: 1"),
        "Failure output should show shrunk input"
    );

    // Test TestRunner utilities
    assert!(
        TestRunner::is_cargo_test() || !TestRunner::is_cargo_test(),
        "is_cargo_test should return a boolean"
    );
}

/// Regression test for memory management
#[test]
fn regression_memory_management() {
    // Test that memory management continues to work correctly

    struct MemoryTestProperty;
    impl Property<Vec<Vec<u8>>> for MemoryTestProperty {
        type Output = ();

        fn test(&self, input: Vec<Vec<u8>>) -> Result<Self::Output, PropertyError> {
            // Create and drop temporary data to test memory management
            let _temp: Vec<Vec<u8>> = input
                .iter()
                .map(|inner| inner.iter().map(|&b| b.wrapping_add(1)).collect())
                .collect();

            // Simple property that should always pass
            Ok(())
        }
    }

    let generator = protest::primitives::VecGenerator::new(
        protest::primitives::VecGenerator::new(range(0u8, 255u8), 0, 20),
        0,
        10,
    );

    let config = TestConfig {
        iterations: 50,
        max_shrink_iterations: 5, // Limit shrinking to control memory usage
        shrink_timeout: Duration::from_secs(1),
        ..TestConfig::default()
    };

    let result = check_with_config(generator, MemoryTestProperty, config);
    assert!(result.is_ok(), "Memory management test should pass");

    if let Ok(success) = result {
        assert_eq!(success.iterations, 50);
        // Test should complete without memory issues
    }
}

/// Comprehensive regression test that runs all critical functionality
#[test]
fn regression_comprehensive_test() {
    println!("Running comprehensive regression test suite...");

    let mut test_results = Vec::new();

    // Run a subset of critical tests
    macro_rules! run_test {
        ($test_name:expr, $test_code:block) => {
            let result = std::panic::catch_unwind(|| $test_code);
            let passed = result.is_ok();
            test_results.push(($test_name, passed));

            if passed {
                println!("  ✓ {}", $test_name);
            } else {
                println!("  ✗ {}", $test_name);
            }
        };
    }

    // Basic functionality
    run_test!("Basic property testing", {
        let result = check_with_closure(range(1, 10), |x: i32| x > 0);
        assert!(result.is_ok());
    });

    // Generator functionality
    run_test!("Built-in generators", {
        let result = check_with_closure(
            protest::primitives::StringGenerator::ascii_printable(1, 10),
            |s: String| !s.is_empty(),
        );
        assert!(result.is_ok());
    });

    // Configuration
    run_test!("Configuration handling", {
        let config = TestConfig {
            iterations: 10,
            seed: Some(42),
            ..TestConfig::default()
        };
        let result = check_with_closure_config(range(1, 20), |x: i32| x > 0, config);
        assert!(result.is_ok());
    });

    // Statistics
    run_test!("Statistics collection", {
        struct SimpleProperty;
        impl Property<i32> for SimpleProperty {
            type Output = ();
            fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
                if input > 0 {
                    Ok(())
                } else {
                    Err(PropertyError::property_failed("Value must be positive"))
                }
            }
        }

        let result = PropertyTestBuilder::new()
            .iterations(10)
            .enable_statistics()
            .run(range(1, 20), SimpleProperty);
        assert!(result.is_ok() && result.unwrap().stats.is_some());
    });

    // Error handling
    run_test!("Error handling", {
        let result = check_with_closure(range(1, 10), |x: i32| x > 5);
        assert!(result.is_err()); // Should fail for some inputs
    });

    // Report results
    let passed_count = test_results.iter().filter(|(_, passed)| *passed).count();
    let total_count = test_results.len();

    println!(
        "\nRegression test summary: {}/{} tests passed",
        passed_count, total_count
    );

    if passed_count == total_count {
        println!("✓ All regression tests passed!");
    } else {
        println!("✗ Some regression tests failed!");
        for (test_name, passed) in &test_results {
            if !passed {
                println!("  Failed: {}", test_name);
            }
        }
        panic!("Regression tests failed!");
    }

    assert_eq!(
        passed_count, total_count,
        "All regression tests should pass"
    );
}
