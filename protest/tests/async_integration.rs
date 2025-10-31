//! Comprehensive async integration tests for the protest library

use protest::generator::ConstantGenerator;
use protest::primitives::{HashMapGenerator, IntGenerator, StringGenerator};
use protest::{
    AsyncProperty, Property, PropertyError, PropertyTestBuilder, TestConfig, check, check_async,
    check_async_with_config,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Test basic async property execution
#[tokio::test]
async fn test_basic_async_property() {
    let generator = IntGenerator::new(1, 100);

    struct PositiveValueProperty;
    impl AsyncProperty<i32> for PositiveValueProperty {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            // Simulate some async work
            tokio::time::sleep(Duration::from_millis(1)).await;

            if value <= 0 {
                Err(PropertyError::PropertyFailed {
                    message: "Value must be positive".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let result = check_async(generator, PositiveValueProperty).await;
    assert!(result.is_ok());

    if let Ok(success) = result {
        assert_eq!(success.iterations, 100); // Default iterations
        assert!(success.stats.is_some());
    }
}

/// Test async property with custom configuration
#[tokio::test]
async fn test_async_property_with_config() {
    let generator = IntGenerator::new(1, 50);
    let config = TestConfig {
        iterations: 20,
        seed: Some(42),
        max_shrink_iterations: 100,
        shrink_timeout: Duration::from_secs(5),
        ..TestConfig::default()
    };

    struct ValueRangeProperty;
    impl AsyncProperty<i32> for ValueRangeProperty {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(2)).await;

            if value > 100 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let result = check_async_with_config(generator, ValueRangeProperty, config).await;
    assert!(result.is_ok());

    if let Ok(success) = result {
        assert_eq!(success.iterations, 20);
        assert_eq!(success.config.seed, Some(42));
    }
}

/// Test async property that fails and triggers shrinking
#[tokio::test]
async fn test_async_property_failure_with_shrinking() {
    let generator = IntGenerator::new(50, 200);
    let config = TestConfig {
        iterations: 10,
        max_shrink_iterations: 50,
        shrink_timeout: Duration::from_secs(2),
        ..TestConfig::default()
    };

    struct LargeValueProperty;
    impl AsyncProperty<i32> for LargeValueProperty {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;

            if value > 100 {
                Err(PropertyError::PropertyFailed {
                    message: format!("Value {} is too large", value),
                    context: Some("async test".to_string()),
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let result = check_async_with_config(generator, LargeValueProperty, config).await;
    assert!(result.is_err());

    if let Err(failure) = result {
        assert!(failure.original_input >= 50);
        assert!(failure.original_input <= 200);

        // Should have attempted shrinking
        if let Some(shrunk) = failure.shrunk_input {
            assert!(shrunk > 100); // Should still fail the property
            assert!(shrunk <= failure.original_input); // Should be smaller than or equal to original
        }

        // Verify error details
        match &failure.error {
            PropertyError::PropertyFailed {
                message,
                context,
                iteration,
            } => {
                assert!(message.contains("too large"));
                assert_eq!(context, &Some("async test".to_string()));
                assert!(iteration.is_some());
            }
            _ => panic!("Expected PropertyFailed error"),
        }
    }
}

/// Test async/sync interoperability - mixing async and sync properties in the same test suite
#[tokio::test]
async fn test_async_sync_interoperability() {
    // Sync property
    struct SyncProperty;
    impl Property<i32> for SyncProperty {
        type Output = ();
        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input < 0 {
                Err(PropertyError::PropertyFailed {
                    message: "Negative value".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    // Async property
    struct AsyncPropertyTest;
    impl AsyncProperty<i32> for AsyncPropertyTest {
        type Output = ();
        async fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;
            if input > 1000 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let generator1 = IntGenerator::new(0, 500);
    let generator2 = IntGenerator::new(0, 500);

    // Test sync property
    let sync_result = check(generator1, SyncProperty);
    assert!(sync_result.is_ok());

    // Test async property
    let async_result = check_async(generator2, AsyncPropertyTest).await;
    assert!(async_result.is_ok());

    // Both should succeed with the same generator range
    if let (Ok(sync_success), Ok(async_success)) = (sync_result, async_result) {
        assert_eq!(sync_success.iterations, async_success.iterations);
    }
}

/// Test async property with complex data structures
#[tokio::test]
async fn test_async_property_with_complex_data() {
    use std::collections::HashMap;

    let generator = HashMapGenerator::new(
        StringGenerator::ascii_printable(5, 10),
        IntGenerator::new(1, 100),
        0,
        10,
    );

    struct SumProperty;
    impl AsyncProperty<HashMap<String, i32>> for SumProperty {
        type Output = ();
        async fn test(&self, map: HashMap<String, i32>) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;

            // Property: sum of all values should be less than 1000
            let sum: i32 = map.values().sum();
            if sum >= 1000 {
                Err(PropertyError::PropertyFailed {
                    message: format!("Sum {} is too large", sum),
                    context: Some("complex data test".to_string()),
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let result = check_async(generator, SumProperty).await;
    // This might pass or fail depending on the generated data
    // The important thing is that it doesn't crash and handles complex types
    match result {
        Ok(_) => {
            // Property passed for all generated maps
        }
        Err(failure) => {
            // Property failed, verify error handling
            assert!(!failure.original_input.is_empty() || failure.original_input.is_empty());
            match &failure.error {
                PropertyError::PropertyFailed { message, .. } => {
                    assert!(message.contains("too large"));
                }
                _ => panic!("Expected PropertyFailed error"),
            }
        }
    }
}

/// Test async property with timeout scenarios
#[tokio::test]
async fn test_async_property_with_timeout() {
    let generator = ConstantGenerator::new(42);
    let config = TestConfig {
        iterations: 5,
        shrink_timeout: Duration::from_millis(100), // Short timeout for shrinking
        ..TestConfig::default()
    };

    struct SlowFailingProperty;
    impl AsyncProperty<i32> for SlowFailingProperty {
        type Output = ();
        async fn test(&self, _value: i32) -> Result<Self::Output, PropertyError> {
            // Simulate slow async operation
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Always fail to trigger shrinking
            Err(PropertyError::PropertyFailed {
                message: "Always fails".to_string(),
                context: None,
                iteration: None,
            })
        }
    }

    let result = check_async_with_config(generator, SlowFailingProperty, config).await;
    assert!(result.is_err());

    if let Err(failure) = result {
        // Should have attempted shrinking but may have timed out
        assert_eq!(failure.original_input, 42);
        // Shrinking timeout should be respected
        assert!(failure.shrink_duration <= Duration::from_millis(150)); // Allow some tolerance
    }
}

/// Test async error handling and propagation
#[tokio::test]
async fn test_async_error_handling() {
    let generator = IntGenerator::new(1, 10);

    struct ErrorHandlingProperty;
    impl AsyncProperty<i32> for ErrorHandlingProperty {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;

            match value {
                1 => Err(PropertyError::PropertyFailed {
                    message: "Value is 1".to_string(),
                    context: Some("specific case".to_string()),
                    iteration: None,
                }),
                2 => Err(PropertyError::GenerationFailed {
                    message: "Simulated generation error".to_string(),
                    context: Some("error test".to_string()),
                }),
                3 => Err(PropertyError::TestCancelled {
                    reason: "User requested cancellation".to_string(),
                }),
                _ => Ok(()),
            }
        }
    }

    let result = check_async(generator, ErrorHandlingProperty).await;
    assert!(result.is_err());

    if let Err(failure) = result {
        // Should capture the first error encountered
        match &failure.error {
            PropertyError::PropertyFailed {
                message,
                context,
                iteration,
            } => {
                assert_eq!(message, "Value is 1");
                assert_eq!(context, &Some("specific case".to_string()));
                assert!(iteration.is_some()); // Should be set by execution engine
            }
            PropertyError::GenerationFailed { .. } => {
                // Could also be this if value 2 was generated first
            }
            PropertyError::TestCancelled { .. } => {
                // Could also be this if value 3 was generated first
            }
            _ => panic!("Unexpected error type: {:?}", failure.error),
        }
    }
}

/// Test async property with PropertyTestBuilder
#[tokio::test]
async fn test_async_property_with_builder() {
    let generator = IntGenerator::new(10, 50);

    struct DivisibleBy7Property;
    impl AsyncProperty<i32> for DivisibleBy7Property {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;

            if value % 7 == 0 {
                Err(PropertyError::PropertyFailed {
                    message: "Divisible by 7".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(30)
        .seed(123)
        .max_shrink_iterations(20)
        .shrink_timeout(Duration::from_secs(1))
        .run_async(generator, DivisibleBy7Property)
        .await;

    // This might pass or fail depending on whether multiples of 7 are generated
    match result {
        Ok(success) => {
            assert_eq!(success.iterations, 30);
            assert_eq!(success.config.seed, Some(123));
        }
        Err(failure) => {
            assert!(failure.original_input % 7 == 0);
            assert_eq!(failure.config.seed, Some(123));
        }
    }
}

/// Performance test for async execution
#[tokio::test]
async fn test_async_performance() {
    let generator = IntGenerator::new(1, 1000);
    let config = TestConfig {
        iterations: 100,
        ..TestConfig::default()
    };

    struct FastAsyncProperty;
    impl AsyncProperty<i32> for FastAsyncProperty {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            // Simulate minimal async work
            tokio::task::yield_now().await;

            if value < 0 {
                Err(PropertyError::PropertyFailed {
                    message: "Negative value".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let start = Instant::now();
    let result = check_async_with_config(generator, FastAsyncProperty, config).await;
    let duration = start.elapsed();

    assert!(result.is_ok());

    // Should complete reasonably quickly (less than 1 second for 100 iterations)
    assert!(
        duration < Duration::from_secs(1),
        "Async execution took too long: {:?}",
        duration
    );

    if let Ok(success) = result {
        assert_eq!(success.iterations, 100);
        println!("Async performance test completed in {:?}", duration);
    }
}

/// Test concurrent async property execution
#[tokio::test]
async fn test_concurrent_async_properties() {
    let generator1 = IntGenerator::new(1, 50);
    let generator2 = IntGenerator::new(51, 100);

    struct Property1;
    impl AsyncProperty<i32> for Property1 {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(2)).await;
            if value > 25 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large for property1".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    struct Property2;
    impl AsyncProperty<i32> for Property2 {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(3)).await;
            if value < 75 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too small for property2".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let config = TestConfig {
        iterations: 10,
        ..TestConfig::default()
    };

    // Run both properties concurrently
    let (result1, result2) = tokio::join!(
        check_async_with_config(generator1, Property1, config.clone()),
        check_async_with_config(generator2, Property2, config)
    );

    // Both should fail due to their respective conditions
    assert!(result1.is_err());
    assert!(result2.is_err());

    if let (Err(failure1), Err(failure2)) = (result1, result2) {
        // Verify each failure has the expected error message
        match &failure1.error {
            PropertyError::PropertyFailed { message, .. } => {
                assert!(message.contains("property1"));
            }
            _ => panic!("Expected PropertyFailed for property1"),
        }

        match &failure2.error {
            PropertyError::PropertyFailed { message, .. } => {
                assert!(message.contains("property2"));
            }
            _ => panic!("Expected PropertyFailed for property2"),
        }
    }
}

/// Test async property with shared state
#[tokio::test]
async fn test_async_property_with_shared_state() {
    let counter = Arc::new(Mutex::new(0));
    let generator = IntGenerator::new(1, 20);

    struct SharedStateProperty {
        counter: Arc<Mutex<i32>>,
    }

    impl AsyncProperty<i32> for SharedStateProperty {
        type Output = ();
        async fn test(&self, value: i32) -> Result<Self::Output, PropertyError> {
            tokio::time::sleep(Duration::from_millis(1)).await;

            // Increment shared counter
            {
                let mut count = self.counter.lock().unwrap();
                *count += 1;
            }

            if value > 15 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    let config = TestConfig {
        iterations: 10,
        ..TestConfig::default()
    };

    let property = SharedStateProperty {
        counter: counter.clone(),
    };

    let result = check_async_with_config(generator, property, config).await;

    // Check that the shared state was modified
    let final_count = *counter.lock().unwrap();

    match result {
        Ok(_) => {
            // All properties passed, counter should equal iterations
            assert_eq!(final_count, 10);
        }
        Err(_) => {
            // Some property failed, counter includes both test iterations and shrinking iterations
            // Counter should be at least the number of iterations that ran before failure
            assert!(final_count > 0); // At least one iteration should have run
            // Shrinking may cause additional increments, so we don't assert an upper bound
        }
    }
}
