//! Async property testing examples
//!
//! This example demonstrates how to use Protest for testing asynchronous code,
//! including async properties, error handling, and integration with async runtimes.

use protest::{
    AsyncProperty, PropertyError, PropertyTestBuilder, TestConfig, check_async,
    check_async_with_config, range,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Example 1: Basic async property
async fn example_1_basic_async() {
    println!("=== Example 1: Basic Async Property ===");

    struct AsyncTimeoutProperty;
    impl AsyncProperty<u64> for AsyncTimeoutProperty {
        type Output = ();

        async fn test(&self, timeout_ms: u64) -> Result<Self::Output, PropertyError> {
            let start = Instant::now();
            sleep(Duration::from_millis(timeout_ms)).await;
            let elapsed = start.elapsed();

            // Property: actual sleep time should be close to requested time
            let expected = Duration::from_millis(timeout_ms);
            let tolerance = Duration::from_millis(10); // 10ms tolerance

            if elapsed >= expected && elapsed <= expected + tolerance {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Sleep time {} ms not close to expected {} ms",
                    elapsed.as_millis(),
                    expected.as_millis()
                )))
            }
        }
    }

    // Test with small timeout values to keep the example fast
    let generator = range(1u64, 20u64);

    match check_async(generator, AsyncTimeoutProperty).await {
        Ok(success) => {
            println!(
                "✓ Async timeout property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Timeout value: {} ms", failure.original_input);
        }
    }
}

// Example 2: Async property with external service simulation
async fn example_2_external_service() {
    println!("\n=== Example 2: External Service Simulation ===");

    // Simulate an external HTTP service
    async fn fetch_user_data(user_id: u32) -> Result<String, String> {
        // Simulate network delay
        sleep(Duration::from_millis(5)).await;

        // Simulate service behavior
        match user_id {
            0 => Err("Invalid user ID".to_string()),
            1..=1000 => Ok(format!("User data for ID {}", user_id)),
            _ => Err("User not found".to_string()),
        }
    }

    struct UserServiceProperty;
    impl AsyncProperty<u32> for UserServiceProperty {
        type Output = ();

        async fn test(&self, user_id: u32) -> Result<Self::Output, PropertyError> {
            match fetch_user_data(user_id).await {
                Ok(data) => {
                    // Property: returned data should contain the user ID
                    if data.contains(&user_id.to_string()) {
                        Ok(())
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "User data '{}' doesn't contain ID {}",
                            data, user_id
                        )))
                    }
                }
                Err(error) => {
                    // Property: errors should only occur for invalid IDs
                    if user_id == 0 || user_id > 1000 {
                        Ok(()) // Expected error
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "Unexpected error for valid ID {}: {}",
                            user_id, error
                        )))
                    }
                }
            }
        }
    }

    let config = TestConfig {
        iterations: 30,
        seed: Some(789),
        ..TestConfig::default()
    };

    match check_async_with_config(range(0u32, 1200u32), UserServiceProperty, config).await {
        Ok(success) => {
            println!(
                "✓ User service property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  User ID: {}", failure.original_input);
        }
    }
}

// Example 3: Async property with shared state
async fn example_3_shared_state() {
    println!("\n=== Example 3: Shared State Management ===");

    #[derive(Clone)]
    struct Counter {
        value: Arc<Mutex<i32>>,
    }

    impl Counter {
        fn new() -> Self {
            Self {
                value: Arc::new(Mutex::new(0)),
            }
        }

        async fn increment(&self, amount: i32) -> i32 {
            // Simulate async work
            sleep(Duration::from_millis(1)).await;

            let mut value = self.value.lock().unwrap();
            *value += amount;
            *value
        }

        fn get(&self) -> i32 {
            *self.value.lock().unwrap()
        }
    }

    struct CounterProperty {
        counter: Counter,
    }

    impl AsyncProperty<i32> for CounterProperty {
        type Output = ();

        async fn test(&self, increment: i32) -> Result<Self::Output, PropertyError> {
            let initial = self.counter.get();
            let result = self.counter.increment(increment).await;

            // Property: result should equal initial + increment
            if result == initial + increment {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Counter increment failed: {} + {} != {}",
                    initial, increment, result
                )))
            }
        }
    }

    let counter = Counter::new();
    let property = CounterProperty {
        counter: counter.clone(),
    };

    match check_async(range(-10, 10), property).await {
        Ok(success) => {
            println!(
                "✓ Counter property passed! ({} iterations)",
                success.iterations
            );
            println!("  Final counter value: {}", counter.get());
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Increment value: {}", failure.original_input);
            println!("  Final counter value: {}", counter.get());
        }
    }
}

// Example 4: Async property with timeout handling
async fn example_4_timeout_handling() {
    println!("\n=== Example 4: Timeout Handling ===");

    async fn slow_computation(n: u32) -> Result<u32, &'static str> {
        // Simulate computation that gets slower with larger inputs
        let delay_ms = n / 10; // 0.1ms per unit
        sleep(Duration::from_millis(delay_ms as u64)).await;

        if n > 1000 {
            Err("Input too large")
        } else {
            Ok(n * 2)
        }
    }

    struct TimeoutProperty;
    impl AsyncProperty<u32> for TimeoutProperty {
        type Output = ();

        async fn test(&self, input: u32) -> Result<Self::Output, PropertyError> {
            // Use tokio::time::timeout to enforce a timeout
            let timeout_duration = Duration::from_millis(50);

            match tokio::time::timeout(timeout_duration, slow_computation(input)).await {
                Ok(Ok(result)) => {
                    // Property: result should be double the input
                    if result == input * 2 {
                        Ok(())
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "Computation result {} != {} * 2",
                            result, input
                        )))
                    }
                }
                Ok(Err(error)) => {
                    // Expected error for large inputs
                    if input > 1000 {
                        Ok(())
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "Unexpected error for input {}: {}",
                            input, error
                        )))
                    }
                }
                Err(_) => {
                    // Timeout occurred
                    Err(PropertyError::property_failed(format!(
                        "Computation timed out for input {}",
                        input
                    )))
                }
            }
        }
    }

    let config = TestConfig {
        iterations: 25,
        max_shrink_iterations: 50,
        shrink_timeout: Duration::from_secs(2),
        ..TestConfig::default()
    };

    match check_async_with_config(range(1u32, 800u32), TimeoutProperty, config).await {
        Ok(success) => {
            println!(
                "✓ Timeout property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Input: {}", failure.original_input);
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk to: {}", shrunk);
            }
        }
    }
}

// Example 5: Async property with PropertyTestBuilder
async fn example_5_builder_pattern() {
    println!("\n=== Example 5: Async Builder Pattern ===");

    // Simulate a database operation
    async fn database_query(query_size: usize) -> Result<Vec<String>, String> {
        // Simulate query processing time based on size
        sleep(Duration::from_millis(query_size as u64 / 100)).await;

        if query_size == 0 {
            return Err("Empty query".to_string());
        }

        if query_size > 1000 {
            return Err("Query too large".to_string());
        }

        // Return mock results
        Ok((0..query_size.min(10))
            .map(|i| format!("Result {}", i))
            .collect())
    }

    struct DatabaseProperty;
    impl AsyncProperty<usize> for DatabaseProperty {
        type Output = ();

        async fn test(&self, query_size: usize) -> Result<Self::Output, PropertyError> {
            match database_query(query_size).await {
                Ok(results) => {
                    // Property: non-empty queries should return results
                    if query_size > 0 && results.is_empty() {
                        Err(PropertyError::property_failed(format!(
                            "Query size {} returned no results",
                            query_size
                        )))
                    } else {
                        Ok(())
                    }
                }
                Err(error) => {
                    // Property: errors should only occur for invalid query sizes
                    if query_size == 0 || query_size > 1000 {
                        Ok(()) // Expected error
                    } else {
                        Err(PropertyError::property_failed(format!(
                            "Unexpected error for query size {}: {}",
                            query_size, error
                        )))
                    }
                }
            }
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(40)
        .seed(456)
        .max_shrink_iterations(30)
        .shrink_timeout(Duration::from_secs(3))
        .enable_statistics()
        .run_async(range(0usize, 1200usize), DatabaseProperty)
        .await;

    match result {
        Ok(success) => {
            println!(
                "✓ Database property passed! ({} iterations)",
                success.iterations
            );
            if let Some(stats) = success.stats {
                println!("  Total queries tested: {}", stats.total_generated);
                // Note: test_duration field not available
                // println!("  Test duration: {:?}", success.test_duration);
            }
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Query size: {}", failure.original_input);
            // Note: test_duration field not available
            // println!("  Test duration: {:?}", failure.test_duration);
        }
    }
}

// Example 6: Concurrent async properties
async fn example_6_concurrent_properties() {
    println!("\n=== Example 6: Concurrent Async Properties ===");

    struct FastProperty;
    impl AsyncProperty<i32> for FastProperty {
        type Output = ();

        async fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            sleep(Duration::from_millis(1)).await;

            if input < 0 {
                Err(PropertyError::property_failed("Negative input"))
            } else {
                Ok(())
            }
        }
    }

    struct SlowProperty;
    impl AsyncProperty<i32> for SlowProperty {
        type Output = ();

        async fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            sleep(Duration::from_millis(5)).await;

            if input > 100 {
                Err(PropertyError::property_failed("Input too large"))
            } else {
                Ok(())
            }
        }
    }

    let config = TestConfig {
        iterations: 15,
        ..TestConfig::default()
    };

    // Run both properties concurrently
    let start = Instant::now();
    let (result1, result2) = tokio::join!(
        check_async_with_config(range(0, 50), FastProperty, config.clone()),
        check_async_with_config(range(0, 50), SlowProperty, config)
    );
    let duration = start.elapsed();

    println!("  Concurrent execution completed in {:?}", duration);

    match (result1, result2) {
        (Ok(success1), Ok(success2)) => {
            println!("✓ Both properties passed!");
            println!("  Fast property: {} iterations", success1.iterations);
            println!("  Slow property: {} iterations", success2.iterations);
        }
        (Err(failure), Ok(_)) => {
            println!("✗ Fast property failed: {}", failure.error);
        }
        (Ok(_), Err(failure)) => {
            println!("✗ Slow property failed: {}", failure.error);
        }
        (Err(failure1), Err(failure2)) => {
            println!("✗ Both properties failed:");
            println!("  Fast: {}", failure1.error);
            println!("  Slow: {}", failure2.error);
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Protest Library - Async Examples");
    println!("================================");

    example_1_basic_async().await;
    example_2_external_service().await;
    example_3_shared_state().await;
    example_4_timeout_handling().await;
    example_5_builder_pattern().await;
    example_6_concurrent_properties().await;

    println!("\n=== Summary ===");
    println!("These async examples demonstrate:");
    println!("• Basic async property testing");
    println!("• Testing external service interactions");
    println!("• Managing shared state in async tests");
    println!("• Handling timeouts and cancellation");
    println!("• Using PropertyTestBuilder with async properties");
    println!("• Running concurrent async property tests");
    println!("\nAsync support in Protest allows you to test:");
    println!("• Network operations and HTTP clients");
    println!("• Database interactions");
    println!("• File I/O operations");
    println!("• Timer and scheduling logic");
    println!("• Any async/await based code");
}
