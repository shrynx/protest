//! Test framework integration tests
//!
//! This module tests the integration between Protest and Rust's standard test framework,
//! including macro functionality, test runner compatibility, and output formatting.

use protest::{
    Property, PropertyError, PropertyTestBuilder, TestConfig, check, check_with_config, just, range,
};
use std::time::Duration;

// Test basic integration with standard test framework
#[test]
fn test_standard_test_integration() {
    struct SimpleProperty;
    impl Property<i32> for SimpleProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input >= 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Negative input"))
            }
        }
    }

    let result = check(range(0, 100), SimpleProperty);
    assert!(
        result.is_ok(),
        "Property should pass with non-negative inputs"
    );
}

#[test]
fn test_property_failure_integration() {
    struct FailingProperty;
    impl Property<i32> for FailingProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input < 50 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Input {} is too large",
                    input
                )))
            }
        }
    }

    let result = check(range(60, 100), FailingProperty);
    assert!(result.is_err(), "Property should fail with large inputs");

    if let Err(failure) = result {
        assert!(failure.original_input >= 60);
        assert!(failure.original_input <= 100);
    }
}

#[test]
fn test_panic_on_property_failure() {
    struct PanicProperty;
    impl Property<i32> for PanicProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input == 42 {
                panic!("The answer to everything!");
            }
            Ok(())
        }
    }

    // This test verifies that panics in properties are handled correctly
    let result = std::panic::catch_unwind(|| check(just(42), PanicProperty));

    assert!(result.is_err(), "Property panic should be caught");
}

#[test]
fn test_custom_test_config_integration() {
    struct ConfigurableProperty {
        #[allow(dead_code)]
        expected_iterations: usize,
    }

    impl Property<i32> for ConfigurableProperty {
        type Output = ();

        fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            Ok(())
        }
    }

    let config = TestConfig {
        iterations: 25,
        seed: Some(123),
        max_shrink_iterations: 50,
        shrink_timeout: Duration::from_secs(1),
        ..TestConfig::default()
    };

    let result = check_with_config(
        range(1, 100),
        ConfigurableProperty {
            expected_iterations: 25,
        },
        config,
    );

    assert!(result.is_ok());
    if let Ok(success) = result {
        assert_eq!(success.iterations, 25);
        assert_eq!(success.config.seed, Some(123));
    }
}

#[test]
fn test_builder_pattern_integration() {
    struct BuilderProperty;
    impl Property<String> for BuilderProperty {
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

    let result = PropertyTestBuilder::new()
        .iterations(30)
        .seed(456)
        .max_shrink_iterations(20)
        .enable_statistics()
        .run(
            protest::primitives::StringGenerator::ascii_printable(5, 20),
            BuilderProperty,
        );

    assert!(result.is_ok());
    if let Ok(success) = result {
        assert_eq!(success.iterations, 30);
        assert_eq!(success.config.seed, Some(456));
        assert!(success.stats.is_some());
    }
}

// Test async integration with tokio test framework
#[tokio::test]
async fn test_async_test_integration() {
    use protest::check_async;

    struct RangeProperty;
    impl protest::AsyncProperty<i32> for RangeProperty {
        type Output = ();

        async fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;

            if (2..=100).contains(&input) {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Out of range"))
            }
        }
    }

    let result = check_async(range(2, 100), RangeProperty).await;
    assert!(
        result.is_ok(),
        "Async property should pass with inputs in range"
    );
}

#[tokio::test]
async fn test_async_property_failure() {
    use protest::check_async;

    struct AsyncFailingProperty;
    impl protest::AsyncProperty<i32> for AsyncFailingProperty {
        type Output = ();

        async fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;

            if input < 10 {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Input too large"))
            }
        }
    }

    let result = check_async(range(15, 25), AsyncFailingProperty).await;
    assert!(
        result.is_err(),
        "Async property should fail with large inputs"
    );
}

// Test output formatting and error reporting
#[test]
fn test_error_message_formatting() {
    struct DetailedErrorProperty;
    impl Property<i32> for DetailedErrorProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input == 13 {
                Err(PropertyError::property_failed_with_context(
                    format!("Unlucky number encountered: input was {}", input),
                    Some("superstition check"),
                    None, // iteration number
                ))
            } else {
                Ok(())
            }
        }
    }

    let result = check(just(13), DetailedErrorProperty);
    assert!(result.is_err());

    if let Err(failure) = result {
        let error_string = format!("{}", failure.error);
        assert!(error_string.contains("Unlucky number"));

        let summary = failure.summary();
        assert!(summary.contains("13"));
    }
}

#[test]
fn test_shrinking_output_formatting() {
    struct ShrinkableProperty;
    impl Property<Vec<i32>> for ShrinkableProperty {
        type Output = ();

        fn test(&self, input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            if input.len() <= 3 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Vector too long: {} elements",
                    input.len()
                )))
            }
        }
    }

    let generator = protest::primitives::VecGenerator::new(
        range(1, 10),
        5, // Minimum length that will cause failure
        10,
    );

    let result = check(generator, ShrinkableProperty);
    assert!(result.is_err());

    if let Err(failure) = result {
        assert!(failure.original_input.len() >= 5);

        // Should have attempted shrinking
        if let Some(shrunk) = failure.shrunk_input {
            assert!(shrunk.len() > 3); // Should still fail
            assert!(shrunk.len() <= failure.original_input.len()); // Should be smaller or equal
        }

        // shrink_steps is usize, always >= 0
        let _steps = failure.shrink_steps;
    }
}

// Test statistics integration with test output
#[test]
fn test_statistics_output_integration() {
    struct StatisticsProperty;
    impl Property<i32> for StatisticsProperty {
        type Output = ();

        fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            Ok(())
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(50)
        .enable_statistics()
        .run(range(1, 100), StatisticsProperty);

    assert!(result.is_ok());

    if let Ok(success) = result {
        assert!(success.stats.is_some());
        let stats = success.stats.unwrap();

        assert_eq!(stats.total_generated, 50);
        assert!(stats.performance_metrics.total_generation_time > Duration::from_nanos(0));

        // Verify statistics can be formatted for output
        let summary = stats.get_summary();
        assert!(summary.contains("50"));

        let report = stats.generate_report();
        assert!(!report.is_empty());
    }
}

// Test compatibility with different test runners
#[test]
fn test_cargo_test_compatibility() {
    // This test verifies that Protest works correctly with `cargo test`

    struct CargoTestProperty;
    impl Property<u32> for CargoTestProperty {
        type Output = ();

        fn test(&self, _input: u32) -> Result<Self::Output, PropertyError> {
            // Property: u32 values are always non-negative by type definition
            Ok(())
        }
    }

    let result = check(range(0u32, 1000u32), CargoTestProperty);
    assert!(result.is_ok());
}

#[test]
fn test_test_name_and_module_integration() {
    // Test that property tests work correctly within modules and with specific names

    mod inner_module {
        use super::*;

        pub struct ModuleProperty;
        impl Property<i32> for ModuleProperty {
            type Output = ();

            fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
                if input.abs() == input {
                    Ok(())
                } else {
                    Err(PropertyError::property_failed("Absolute value mismatch"))
                }
            }
        }
    }

    let result = check(range(0, 100), inner_module::ModuleProperty);
    assert!(result.is_ok());
}

// Test thread safety and concurrent test execution
#[test]
fn test_concurrent_property_tests() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for i in 0..4 {
        let counter_clone = counter.clone();
        let handle = thread::spawn(move || {
            struct ThreadSafeProperty {
                thread_id: usize,
                counter: Arc<Mutex<i32>>,
            }

            impl Property<i32> for ThreadSafeProperty {
                type Output = ();

                fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
                    // Increment counter to track concurrent execution
                    {
                        let mut count = self.counter.lock().unwrap();
                        *count += 1;
                    }

                    if input >= 0 {
                        Ok(())
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "Thread {} got negative input",
                            self.thread_id
                        )))
                    }
                }
            }

            let property = ThreadSafeProperty {
                thread_id: i,
                counter: counter_clone,
            };

            let config = TestConfig {
                iterations: 10,
                ..TestConfig::default()
            };

            check_with_config(range(0, 50), property, config)
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut all_passed = true;
    for handle in handles {
        match handle.join() {
            Ok(Ok(_)) => {}                   // Test passed
            Ok(Err(_)) => all_passed = false, // Test failed
            Err(_) => all_passed = false,     // Thread panicked
        }
    }

    assert!(all_passed, "All concurrent property tests should pass");

    // Verify that all threads executed
    let final_count = *counter.lock().unwrap();
    assert_eq!(
        final_count, 40,
        "Expected 4 threads × 10 iterations = 40 executions"
    );
}

// Test integration with custom test harnesses
#[test]
fn test_custom_harness_compatibility() {
    // This test verifies that Protest can work with custom test harnesses

    struct HarnessProperty {
        test_metadata: String,
    }

    impl Property<i32> for HarnessProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // Use the metadata in the test
            if self.test_metadata.is_empty() {
                return Err(PropertyError::property_failed("No test metadata"));
            }

            if input > 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Non-positive input"))
            }
        }
    }

    let property = HarnessProperty {
        test_metadata: "custom_harness_test".to_string(),
    };

    let result = check(range(1, 100), property);
    assert!(result.is_ok());
}

// Test memory usage and cleanup
#[test]
fn test_memory_cleanup_integration() {
    struct MemoryTestProperty;
    impl Property<Vec<u8>> for MemoryTestProperty {
        type Output = ();

        fn test(&self, input: Vec<u8>) -> Result<Self::Output, PropertyError> {
            // Create some temporary data to test memory cleanup
            let _temp_data: Vec<u8> = input.iter().map(|&x| x.wrapping_mul(2)).collect();

            if input.len() <= 1000 {
                Ok(())
            } else {
                Err(PropertyError::property_failed("Vector too large"))
            }
        }
    }

    let generator = protest::primitives::VecGenerator::new(
        range(0u8, 255u8),
        0,
        100, // Keep size reasonable for test
    );

    let config = TestConfig {
        iterations: 20,
        max_shrink_iterations: 10,
        shrink_timeout: Duration::from_secs(1),
        ..TestConfig::default()
    };

    let result = check_with_config(generator, MemoryTestProperty, config);
    assert!(result.is_ok());

    // Test should complete without memory issues
}

// Test output capture and formatting
#[test]
fn test_output_capture_integration() {
    struct VerboseProperty;
    impl Property<i32> for VerboseProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // This property might print debug information
            if input == 0 {
                Err(PropertyError::property_failed_with_context(
                    "Zero input detected - this is additional context for debugging",
                    Some("verbose test"),
                    None, // iteration number
                ))
            } else {
                Ok(())
            }
        }
    }

    let result = check(just(0), VerboseProperty);
    assert!(result.is_err());

    if let Err(failure) = result {
        // Verify that error information is properly formatted
        let error_display = format!("{}", failure.error);
        assert!(error_display.contains("Zero input"));

        let failure_summary = failure.summary();
        assert!(failure_summary.contains("0"));
    }
}
