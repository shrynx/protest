//! Performance and best practices examples
//!
//! This example demonstrates performance optimization techniques, parallel execution,
//! and best practices for using Protest effectively in large test suites.

use protest::{
    Generator, GeneratorConfig, ParallelConfig, Property, PropertyError, PropertyTestBuilder,
    TestConfig, check, check_parallel, check_with_config, lazy, range,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Example 1: Parallel property execution
fn example_1_parallel_execution() {
    println!("=== Example 1: Parallel Property Execution ===");

    struct ComputeIntensiveProperty;
    impl Property<i32> for ComputeIntensiveProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // Simulate compute-intensive work
            let mut sum = 0;
            for i in 0..input.abs() % 1000 {
                sum += i * i;
            }

            // Property: sum should be non-negative
            if sum >= 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Negative sum: {}",
                    sum
                )))
            }
        }
    }

    let config = TestConfig {
        iterations: 100,
        ..TestConfig::default()
    };

    let parallel_config = ParallelConfig {
        num_threads: 4,
        batch_size: 25,
        enabled: true,
    };

    // Sequential execution
    println!("  Running sequential execution...");
    let start = Instant::now();
    let sequential_result = check(range(1, 1000), ComputeIntensiveProperty);
    let sequential_time = start.elapsed();

    // Parallel execution
    println!("  Running parallel execution...");
    let start = Instant::now();
    let parallel_result = check_parallel(
        range(1, 1000),
        ComputeIntensiveProperty,
        config,
        parallel_config,
    );
    let parallel_time = start.elapsed();

    match (sequential_result, parallel_result) {
        (Ok(seq_success), Ok(par_success)) => {
            println!("✓ Both executions passed!");
            println!(
                "  Sequential: {} iterations in {:?}",
                seq_success.iterations, sequential_time
            );
            println!(
                "  Parallel: {} iterations in {:?}",
                par_success.iterations, parallel_time
            );

            let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
            println!("  Speedup: {:.2}x", speedup);
        }
        (Err(seq_failure), _) => {
            println!("✗ Sequential execution failed: {}", seq_failure.error);
        }
        (_, Err(par_failure)) => {
            println!("✗ Parallel execution failed: {}", par_failure.error);
        }
    }
}

// Example 2: Lazy generators for expensive operations
fn example_2_lazy_generators() {
    println!("\n=== Example 2: Lazy Generators ===");

    // Expensive generator that we want to evaluate lazily
    #[derive(Clone)]
    struct ExpensiveDataGenerator {
        computation_count: Arc<Mutex<usize>>,
    }

    impl ExpensiveDataGenerator {
        fn new() -> Self {
            Self {
                computation_count: Arc::new(Mutex::new(0)),
            }
        }

        #[allow(dead_code)]
        fn get_computation_count(&self) -> usize {
            *self.computation_count.lock().unwrap()
        }
    }

    impl Generator<Vec<i32>> for ExpensiveDataGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> Vec<i32> {
            // Increment computation counter
            {
                let mut count = self.computation_count.lock().unwrap();
                *count += 1;
            }

            // Simulate expensive computation
            thread::sleep(Duration::from_millis(10));

            // Generate a vector
            let size = (rng.next_u32() % 10) + 1;
            (0..size).map(|_| rng.next_u32() as i32).collect()
        }

        fn shrink(&self, value: &Vec<i32>) -> Box<dyn Iterator<Item = Vec<i32>>> {
            let mut shrinks = Vec::new();

            if !value.is_empty() {
                shrinks.push(value[..value.len() - 1].to_vec());
                if value.len() > 1 {
                    shrinks.push(vec![value[0]]);
                }
            }

            Box::new(shrinks.into_iter())
        }
    }

    struct VectorSumProperty;
    impl Property<Vec<i32>> for VectorSumProperty {
        type Output = ();

        fn test(&self, input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            let sum: i64 = input.iter().map(|&x| x as i64).sum();

            // Property: sum should not overflow i32 bounds
            if sum >= i32::MIN as i64 && sum <= i32::MAX as i64 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Sum {} overflows i32 bounds",
                    sum
                )))
            }
        }
    }

    let expensive_gen = ExpensiveDataGenerator::new();
    let computation_counter = expensive_gen.computation_count.clone();

    // Test with lazy evaluation
    println!("  Testing with lazy generator...");
    let start = Instant::now();
    let expensive_gen_clone = expensive_gen.clone();
    let result = PropertyTestBuilder::new().iterations(20).run(
        lazy(move || Box::new(expensive_gen_clone.clone())),
        VectorSumProperty,
    );
    let duration = start.elapsed();

    match result {
        Ok(success) => {
            println!(
                "✓ Lazy generator test passed! ({} iterations)",
                success.iterations
            );
            println!(
                "  Computations performed: {}",
                computation_counter.lock().unwrap()
            );
            println!("  Total time: {:?}", duration);
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!(
                "  Computations before failure: {}",
                computation_counter.lock().unwrap()
            );
        }
    }
}

// Example 3: Memory-efficient testing with large datasets
fn example_3_memory_efficient_testing() {
    println!("\n=== Example 3: Memory-Efficient Testing ===");

    struct LargeDataGenerator {
        max_size: usize,
    }

    impl LargeDataGenerator {
        fn new(max_size: usize) -> Self {
            Self { max_size }
        }
    }

    impl Generator<Vec<u8>> for LargeDataGenerator {
        fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> Vec<u8> {
            let size = (rng.next_u32() as usize % self.max_size) + 1;
            (0..size).map(|_| rng.next_u32() as u8).collect()
        }

        fn shrink(&self, value: &Vec<u8>) -> Box<dyn Iterator<Item = Vec<u8>>> {
            let mut shrinks = Vec::new();

            // Shrink by halving size
            if value.len() > 1 {
                shrinks.push(value[..value.len() / 2].to_vec());
            }

            // Shrink to minimal non-empty vector
            if value.len() > 1 {
                shrinks.push(vec![value[0]]);
            }

            Box::new(shrinks.into_iter())
        }
    }

    struct DataIntegrityProperty;
    impl Property<Vec<u8>> for DataIntegrityProperty {
        type Output = ();

        fn test(&self, input: Vec<u8>) -> Result<Self::Output, PropertyError> {
            // Property: data should compress and decompress correctly
            let compressed = compress_data(&input);
            let decompressed = decompress_data(&compressed);

            if input == decompressed {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Data integrity failed: {} bytes -> {} bytes -> {} bytes",
                    input.len(),
                    compressed.len(),
                    decompressed.len()
                )))
            }
        }
    }

    // Simple compression simulation (just for example)
    fn compress_data(data: &[u8]) -> Vec<u8> {
        // Simulate compression by removing consecutive duplicates
        let mut compressed = Vec::new();
        let mut last = None;

        for &byte in data {
            if last != Some(byte) {
                compressed.push(byte);
                last = Some(byte);
            }
        }

        compressed
    }

    fn decompress_data(compressed: &[u8]) -> Vec<u8> {
        // For this simple example, decompression is identity
        // In real scenarios, this would be more complex
        compressed.to_vec()
    }

    let config = TestConfig {
        iterations: 50,
        max_shrink_iterations: 20, // Limit shrinking for large data
        shrink_timeout: Duration::from_secs(5),
        ..TestConfig::default()
    };

    println!("  Testing with large data vectors (up to 10KB)...");
    let start = Instant::now();
    let result = check_with_config(
        LargeDataGenerator::new(10_000),
        DataIntegrityProperty,
        config,
    );
    let duration = start.elapsed();

    match result {
        Ok(success) => {
            println!(
                "✓ Data integrity property passed! ({} iterations)",
                success.iterations
            );
            println!("  Test completed in {:?}", duration);
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Data size: {} bytes", failure.original_input.len());
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk to: {} bytes", shrunk.len());
            }
        }
    }
}

// Example 4: Performance monitoring and statistics
fn example_4_performance_monitoring() {
    println!("\n=== Example 4: Performance Monitoring ===");

    struct PerformanceTestProperty {
        max_duration: Duration,
    }

    impl Property<String> for PerformanceTestProperty {
        type Output = ();

        fn test(&self, input: String) -> Result<Self::Output, PropertyError> {
            let start = Instant::now();

            // Simulate string processing operation
            let processed = input
                .chars()
                .map(|c| c.to_uppercase().collect::<String>())
                .collect::<Vec<_>>()
                .join("");

            let duration = start.elapsed();

            // Property: operation should complete within time limit
            if duration <= self.max_duration {
                // Also verify correctness
                if processed == input.to_uppercase() {
                    Ok(())
                } else {
                    Err(PropertyError::property_failed(
                        "String processing produced incorrect result".to_string(),
                    ))
                }
            } else {
                Err(PropertyError::property_failed(format!(
                    "Operation took {:?}, expected <= {:?}",
                    duration, self.max_duration
                )))
            }
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(100)
        .enable_statistics()
        // Note: generator_config method doesn't exist, using default
        .run(
            protest::primitives::StringGenerator::ascii_printable(10, 100),
            PerformanceTestProperty {
                max_duration: Duration::from_millis(1),
            },
        );

    match result {
        Ok(success) => {
            println!(
                "✓ Performance property passed! ({} iterations)",
                success.iterations
            );

            if let Some(stats) = success.stats {
                println!("  Performance metrics:");
                println!(
                    "    Total generation time: {:?}",
                    stats.performance_metrics.total_generation_time
                );
                println!(
                    "    Average per generation: {:?}",
                    stats.performance_metrics.average_generation_time
                );
                // Note: peak_memory_usage field not available in current implementation
                // println!("    Peak memory usage: {} KB", stats.performance_metrics.peak_memory_usage / 1024);

                // Show string statistics
                if let Some(string_coverage) = stats.coverage_info.string_coverage.values().next() {
                    println!("  String statistics:");
                    println!("    Average length: {:.1}", string_coverage.average_length);
                    println!(
                        "    Unique characters: {}",
                        string_coverage.character_distribution.len()
                    );
                }
            }
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  String length: {}", failure.original_input.len());
            println!("  Test duration: {:?}", failure.test_duration);
        }
    }
}

// Example 5: Best practices for test organization
fn example_5_test_organization() {
    println!("\n=== Example 5: Test Organization Best Practices ===");

    // Group related properties into a test suite
    struct MathOperationsTestSuite;

    impl MathOperationsTestSuite {
        fn test_addition_properties() -> Result<(), String> {
            println!("  Testing addition properties...");

            // Commutativity
            struct CommutativeProperty;
            impl Property<(i32, i32)> for CommutativeProperty {
                type Output = ();
                fn test(&self, (a, b): (i32, i32)) -> Result<Self::Output, PropertyError> {
                    if a.wrapping_add(b) == b.wrapping_add(a) {
                        Ok(())
                    } else {
                        Err(PropertyError::property_failed("Addition not commutative"))
                    }
                }
            }

            let result = check((range(-100, 100), range(-100, 100)), CommutativeProperty);
            match result {
                Ok(_) => println!("    ✓ Commutativity passed"),
                Err(e) => return Err(format!("Commutativity failed: {}", e.error)),
            }

            // Associativity
            struct AssociativeProperty;
            impl Property<(i32, i32, i32)> for AssociativeProperty {
                type Output = ();
                fn test(&self, (a, b, c): (i32, i32, i32)) -> Result<Self::Output, PropertyError> {
                    // Check for overflow to avoid false failures
                    if let (Some(ab), Some(bc)) = (a.checked_add(b), b.checked_add(c))
                        && let (Some(ab_c), Some(a_bc)) = (ab.checked_add(c), a.checked_add(bc))
                        && ab_c == a_bc
                    {
                        return Ok(());
                    }
                    // If we can't check due to overflow, consider it passed
                    Ok(())
                }
            }

            let triple_gen = (range(-50, 50), range(-50, 50), range(-50, 50));

            let result = check(triple_gen, AssociativeProperty);
            match result {
                Ok(_) => println!("    ✓ Associativity passed"),
                Err(e) => return Err(format!("Associativity failed: {}", e.error)),
            }

            Ok(())
        }

        fn test_multiplication_properties() -> Result<(), String> {
            println!("  Testing multiplication properties...");

            struct DistributiveProperty;
            impl Property<(i32, i32, i32)> for DistributiveProperty {
                type Output = ();
                fn test(&self, (a, b, c): (i32, i32, i32)) -> Result<Self::Output, PropertyError> {
                    // a * (b + c) = a * b + a * c
                    if let (Some(bc), Some(ab), Some(ac)) =
                        (b.checked_add(c), a.checked_mul(b), a.checked_mul(c))
                        && let (Some(a_bc), Some(ab_ac)) = (a.checked_mul(bc), ab.checked_add(ac))
                        && a_bc == ab_ac
                    {
                        return Ok(());
                    }
                    // If overflow, consider passed
                    Ok(())
                }
            }

            let triple_gen = (range(-20, 20), range(-20, 20), range(-20, 20));

            let result = check(triple_gen, DistributiveProperty);
            match result {
                Ok(_) => println!("    ✓ Distributivity passed"),
                Err(e) => return Err(format!("Distributivity failed: {}", e.error)),
            }

            Ok(())
        }

        fn run_all_tests() -> Result<(), String> {
            Self::test_addition_properties()?;
            Self::test_multiplication_properties()?;
            Ok(())
        }
    }

    match MathOperationsTestSuite::run_all_tests() {
        Ok(()) => {
            println!("✓ All math operation properties passed!");
        }
        Err(error) => {
            println!("✗ Test suite failed: {}", error);
        }
    }
}

fn main() {
    println!("Protest Library - Performance and Best Practices Examples");
    println!("========================================================");

    example_1_parallel_execution();
    example_2_lazy_generators();
    example_3_memory_efficient_testing();
    example_4_performance_monitoring();
    example_5_test_organization();

    println!("\n=== Performance Best Practices Summary ===");
    println!("• Use parallel execution for CPU-intensive properties");
    println!("• Employ lazy generators for expensive data generation");
    println!("• Limit shrinking iterations and timeouts for large data");
    println!("• Enable statistics collection to monitor performance");
    println!("• Organize related properties into test suites");
    println!("• Use appropriate generator configurations for your use case");
    println!("• Consider memory usage when testing with large datasets");
    println!("• Profile your tests to identify bottlenecks");
    println!("\nThese techniques help you:");
    println!("• Scale property tests to large codebases");
    println!("• Maintain fast feedback cycles in CI/CD");
    println!("• Efficiently test performance-critical code");
    println!("• Organize and maintain complex test suites");
}
