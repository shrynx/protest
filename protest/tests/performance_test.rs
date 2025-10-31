//! Tests for performance optimizations

use protest::generator::ConstantGenerator;
use protest::{ParallelConfig, Property, PropertyError, TestConfig, check, check_parallel};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// Test property that counts invocations
struct CountingProperty {
    counter: Arc<AtomicUsize>,
}

impl CountingProperty {
    fn new() -> (Self, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        (
            Self {
                counter: counter.clone(),
            },
            counter,
        )
    }
}

impl Property<i32> for CountingProperty {
    type Output = ();
    fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_parallel_execution_basic() {
    let generator = ConstantGenerator::new(42);
    let (property, counter) = CountingProperty::new();
    let config = TestConfig {
        iterations: 20,
        ..TestConfig::default()
    };
    let parallel_config = ParallelConfig {
        enabled: true,
        num_threads: 2,
        batch_size: 5,
    };

    let result = check_parallel(generator, property, config, parallel_config);

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 20);
}

#[test]
fn test_parallel_vs_sequential_performance() {
    let generator = ConstantGenerator::new(42);

    // Sequential test
    let (seq_property, seq_counter) = CountingProperty::new();
    let seq_result = check(generator.clone(), seq_property);
    assert!(seq_result.is_ok());
    assert_eq!(seq_counter.load(Ordering::SeqCst), 100); // Default iterations

    // Parallel test
    let (par_property, par_counter) = CountingProperty::new();
    let config = TestConfig {
        iterations: 100,
        ..TestConfig::default()
    };
    let parallel_config = ParallelConfig {
        enabled: true,
        num_threads: 4,
        batch_size: 10,
    };

    let par_result = check_parallel(generator, par_property, config, parallel_config);
    assert!(par_result.is_ok());
    assert_eq!(par_counter.load(Ordering::SeqCst), 100);
}

#[test]
fn test_parallel_config_defaults() {
    let config = ParallelConfig::default();
    assert!(config.enabled);
    assert!(config.num_threads > 0);
    assert_eq!(config.batch_size, 10);
}

#[test]
fn test_lazy_generator() {
    use protest::lazy;

    let lazy_gen = lazy(|| Box::new(ConstantGenerator::new(42)));
    let (property, counter) = CountingProperty::new();

    let result = check(lazy_gen, property);
    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 100);
}

#[test]
fn test_streaming_shrink_strategy() {
    use protest::StreamingShrinkStrategy;

    let mut strategy = StreamingShrinkStrategy::<i32>::new(1); // 1MB limit

    assert!(strategy.within_memory_limit(1024)); // 1KB should be fine

    strategy.update_memory_usage(1024);
    assert!(!strategy.within_memory_limit(1024 * 1024)); // 1MB more would exceed limit

    strategy.reset_memory_usage();
    assert!(strategy.within_memory_limit(1024 * 1024)); // Should be fine after reset
}

// Note: Async parallel tests removed as check_async_parallel requires tokio::spawn
// The library now only provides check_async() for runtime-agnostic async testing
// Users can implement their own parallel async execution with their chosen runtime
