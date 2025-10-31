//! Performance optimizations for property testing including parallel execution,
//! lazy evaluation, and memory-efficient strategies.

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::{GeneratorConfig, TestConfig};
use crate::error::{PropertyError, PropertyResult, TestFailure, TestSuccess};
use crate::generator::Generator;
use crate::property::Property;
use crate::rng::create_seeded_rng;

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of threads to use for parallel execution
    pub num_threads: usize,
    /// Batch size for distributing work across threads
    pub batch_size: usize,
    /// Whether to enable parallel execution
    pub enabled: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get(),
            batch_size: 10,
            enabled: true,
        }
    }
}

/// Lazy generator wrapper that defers expensive computations
pub struct LazyGenerator<T, F> {
    generator_fn: F,
    _phantom: PhantomData<T>,
}

impl<T, F> LazyGenerator<T, F>
where
    F: Fn() -> Box<dyn Generator<T> + Send + Sync>,
{
    /// Create a new lazy generator
    pub fn new(generator_fn: F) -> Self {
        Self {
            generator_fn,
            _phantom: PhantomData,
        }
    }
}

impl<T, F> Generator<T> for LazyGenerator<T, F>
where
    T: 'static,
    F: Fn() -> Box<dyn Generator<T> + Send + Sync> + Send + Sync,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> T {
        let generator = (self.generator_fn)();
        generator.generate(rng, config)
    }

    fn shrink(&self, value: &T) -> Box<dyn Iterator<Item = T>> {
        let generator = (self.generator_fn)();
        generator.shrink(value)
    }
}

/// Memory-efficient shrinking strategy that uses streaming
pub struct StreamingShrinkStrategy<T> {
    max_memory_usage: usize,
    current_memory_usage: usize,
    _phantom: PhantomData<T>,
}

impl<T> StreamingShrinkStrategy<T> {
    /// Create a new streaming shrink strategy with memory limit
    pub fn new(max_memory_mb: usize) -> Self {
        Self {
            max_memory_usage: max_memory_mb * 1024 * 1024, // Convert MB to bytes
            current_memory_usage: 0,
            _phantom: PhantomData,
        }
    }

    /// Check if we're within memory limits
    pub fn within_memory_limit(&self, additional_bytes: usize) -> bool {
        self.current_memory_usage + additional_bytes <= self.max_memory_usage
    }

    /// Update memory usage tracking
    pub fn update_memory_usage(&mut self, bytes: usize) {
        self.current_memory_usage += bytes;
    }

    /// Reset memory usage tracking
    pub fn reset_memory_usage(&mut self) {
        self.current_memory_usage = 0;
    }
}

/// Parallel property test executor
pub struct ParallelPropertyTest<T, G, P> {
    generator: Arc<G>,
    property: Arc<P>,
    config: TestConfig,
    parallel_config: ParallelConfig,
    _phantom: PhantomData<T>,
}

impl<T, G, P> ParallelPropertyTest<T, G, P>
where
    T: Clone + Send + Sync + 'static + std::fmt::Debug + PartialEq,
    G: Generator<T> + Send + Sync + 'static,
    P: Property<T> + Send + Sync + 'static,
{
    /// Create a new parallel property test
    pub fn new(
        generator: G,
        property: P,
        config: TestConfig,
        parallel_config: ParallelConfig,
    ) -> Self {
        Self {
            generator: Arc::new(generator),
            property: Arc::new(property),
            config,
            parallel_config,
            _phantom: PhantomData,
        }
    }

    /// Execute the property test in parallel
    pub fn run(self) -> PropertyResult<T> {
        if !self.parallel_config.enabled || self.config.iterations < self.parallel_config.batch_size
        {
            // Fall back to sequential execution for small test counts
            return self.run_sequential();
        }

        let test_start = Instant::now();
        let num_threads = self.parallel_config.num_threads.min(self.config.iterations);
        let iterations_per_thread = self.config.iterations / num_threads;
        let remaining_iterations = self.config.iterations % num_threads;

        // Use crossbeam for scoped threads to avoid lifetime issues
        let result = crossbeam::scope(|s| {
            let mut handles = Vec::new();

            for thread_id in 0..num_threads {
                let generator = Arc::clone(&self.generator);
                let property = Arc::clone(&self.property);
                let config = self.config.clone();

                let thread_iterations = if thread_id < remaining_iterations {
                    iterations_per_thread + 1
                } else {
                    iterations_per_thread
                };

                let handle = s.spawn(move |_| {
                    Self::run_thread_batch(
                        generator,
                        property,
                        config,
                        thread_id,
                        thread_iterations,
                    )
                });

                handles.push(handle);
            }

            // Collect results from all threads
            for handle in handles {
                match handle.join() {
                    Ok(Ok(_)) => continue,                   // Thread succeeded
                    Ok(Err(failure)) => return Err(failure), // Thread found a failure
                    Err(_) => {
                        // Thread panicked
                        return Err(TestFailure::new(
                            PropertyError::execution_failed(
                                "Thread panicked during parallel execution",
                            ),
                            // We need a dummy value here - this is a limitation of the current design
                            self.generator
                                .generate(&mut create_seeded_rng(0), &self.config.generator_config),
                            None,
                            0,
                            self.config.clone(),
                            0,
                            test_start.elapsed(),
                            Duration::from_secs(0),
                        ));
                    }
                }
            }

            // All threads succeeded
            Ok(TestSuccess::new(
                self.config.iterations,
                self.config,
                None, // Stats aggregation would need to be implemented
            ))
        });

        result.unwrap() // crossbeam::scope guarantees this won't panic
    }

    /// Run a batch of iterations in a single thread
    fn run_thread_batch(
        generator: Arc<G>,
        property: Arc<P>,
        config: TestConfig,
        thread_id: usize,
        iterations: usize,
    ) -> PropertyResult<T> {
        let mut rng = if let Some(seed) = config.seed {
            // Create a unique seed for each thread to avoid correlation
            create_seeded_rng(seed.wrapping_add(thread_id as u64))
        } else {
            crate::rng::create_rng()
        };

        for iteration in 0..iterations {
            let global_iteration = thread_id * iterations + iteration;

            // Generate test input
            let input = generator.generate(&mut rng, &config.generator_config);

            // Test the property
            match property.test(input.clone()) {
                Ok(_) => continue,
                Err(mut error) => {
                    // Add iteration context
                    error = match error {
                        PropertyError::PropertyFailed {
                            message,
                            context,
                            iteration: None,
                        } => PropertyError::PropertyFailed {
                            message,
                            context,
                            iteration: Some(global_iteration),
                        },
                        other => other,
                    };

                    // For parallel execution, we don't shrink immediately to avoid complexity
                    // The caller can shrink the failure if needed
                    return Err(TestFailure::new(
                        error,
                        input,
                        None, // No shrinking in parallel mode for now
                        0,
                        config,
                        global_iteration,
                        Duration::from_secs(0), // Thread doesn't track total time
                        Duration::from_secs(0),
                    ));
                }
            }
        }

        Ok(TestSuccess::new(iterations, config, None))
    }

    /// Fall back to sequential execution
    fn run_sequential(self) -> PropertyResult<T> {
        // For sequential fallback, we need to work with the Arc directly
        // since we can't guarantee Clone is implemented for all generators/properties
        let generator = Arc::try_unwrap(self.generator)
            .map_err(|_| "Failed to unwrap generator Arc")
            .unwrap();
        let property = Arc::try_unwrap(self.property)
            .map_err(|_| "Failed to unwrap property Arc")
            .unwrap();

        let test = crate::execution::PropertyTest::new(generator, property, self.config);
        test.run()
    }
}

// Note: ParallelAsyncPropertyTest has been removed to keep the library runtime-agnostic.
// Use check_async() with your own async runtime instead.

/// Execute a property test with parallel optimization
pub fn check_parallel<T, G, P>(
    generator: G,
    property: P,
    config: TestConfig,
    parallel_config: ParallelConfig,
) -> PropertyResult<T>
where
    T: Clone + Send + Sync + 'static + std::fmt::Debug + PartialEq,
    G: Generator<T> + Send + Sync + 'static,
    P: Property<T> + Send + Sync + 'static,
{
    let test = ParallelPropertyTest::new(generator, property, config, parallel_config);
    test.run()
}

// Note: check_async_parallel() has been removed to keep the library runtime-agnostic.
// Use check_async() with your own async runtime instead.

/// Create a lazy generator that defers expensive computations
pub fn lazy<T, F>(generator_fn: F) -> LazyGenerator<T, F>
where
    F: Fn() -> Box<dyn Generator<T> + Send + Sync>,
{
    LazyGenerator::new(generator_fn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ConstantGenerator;
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

    // Async version of counting property - commented out since parallel async tests are disabled
    // struct AsyncCountingProperty {
    //     counter: Arc<AtomicUsize>,
    // }

    // impl AsyncCountingProperty {
    //     fn new() -> (Self, Arc<AtomicUsize>) {
    //         let counter = Arc::new(AtomicUsize::new(0));
    //         (
    //             Self {
    //                 counter: counter.clone(),
    //             },
    //             counter,
    //         )
    //     }
    // }

    // impl AsyncProperty<i32> for AsyncCountingProperty {
    //     type Output = ();
    //     async fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
    //         tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    //         self.counter.fetch_add(1, Ordering::SeqCst);
    //         Ok(())
    //     }
    // }

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.enabled);
        assert!(config.num_threads > 0);
        assert_eq!(config.batch_size, 10);
    }

    #[test]
    fn test_lazy_generator() {
        let lazy_gen = lazy(|| Box::new(ConstantGenerator::new(42)));
        let mut rng = crate::rng::create_rng();
        let config = GeneratorConfig::default();

        let value = lazy_gen.generate(&mut rng, &config);
        assert_eq!(value, 42);

        let shrinks: Vec<_> = lazy_gen.shrink(&value).collect();
        assert!(shrinks.is_empty());
    }

    #[test]
    fn test_streaming_shrink_strategy() {
        let mut strategy = StreamingShrinkStrategy::<i32>::new(1); // 1MB limit

        assert!(strategy.within_memory_limit(1024)); // 1KB should be fine

        strategy.update_memory_usage(1024);
        assert!(!strategy.within_memory_limit(1024 * 1024)); // 1MB more would exceed limit

        strategy.reset_memory_usage();
        assert!(strategy.within_memory_limit(1024 * 1024)); // Should be fine after reset
    }

    #[test]
    fn test_parallel_property_test_sequential_fallback() {
        let generator = ConstantGenerator::new(42);
        let (property, counter) = CountingProperty::new();
        let config = TestConfig {
            iterations: 5, // Small number to trigger sequential fallback
            ..TestConfig::default()
        };
        let parallel_config = ParallelConfig {
            batch_size: 10, // Larger than iterations
            ..ParallelConfig::default()
        };

        let test = ParallelPropertyTest::new(generator, property, config, parallel_config);
        let result = test.run();

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_parallel_property_test_disabled() {
        let generator = ConstantGenerator::new(42);
        let (property, counter) = CountingProperty::new();
        let config = TestConfig {
            iterations: 20,
            ..TestConfig::default()
        };
        let parallel_config = ParallelConfig {
            enabled: false,
            ..ParallelConfig::default()
        };

        let test = ParallelPropertyTest::new(generator, property, config, parallel_config);
        let result = test.run();

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 20);
    }

    #[test]
    fn test_parallel_property_test_enabled() {
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

        let test = ParallelPropertyTest::new(generator, property, config, parallel_config);
        let result = test.run();

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 20);
    }

    #[test]
    fn test_check_parallel_function() {
        let generator = ConstantGenerator::new(42);
        let (property, counter) = CountingProperty::new();
        let config = TestConfig {
            iterations: 15,
            ..TestConfig::default()
        };
        let parallel_config = ParallelConfig::default();

        let result = check_parallel(generator, property, config, parallel_config);

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 15);
    }

    // Note: These tests use ParallelAsyncPropertyTest which requires tokio::spawn
    // Removed to keep library runtime-agnostic
    // #[tokio::test]
    // async fn test_parallel_async_property_test_sequential_fallback() {
    //     let generator = ConstantGenerator::new(42);
    //     let (property, counter) = AsyncCountingProperty::new();
    //     let config = TestConfig {
    //         iterations: 5,
    //         ..TestConfig::default()
    //     };
    //     let parallel_config = ParallelConfig {
    //         batch_size: 10,
    //         ..ParallelConfig::default()
    //     };
    //
    //     let test = ParallelAsyncPropertyTest::new(generator, property, config, parallel_config);
    //     let result = test.run().await;
    //
    //     assert!(result.is_ok());
    //     assert_eq!(counter.load(Ordering::SeqCst), 5);
    // }
    //
    // #[tokio::test]
    // async fn test_parallel_async_property_test_enabled() {
    //     let generator = ConstantGenerator::new(42);
    //     let (property, counter) = AsyncCountingProperty::new();
    //     let config = TestConfig {
    //         iterations: 20,
    //         ..TestConfig::default()
    //     };
    //     let parallel_config = ParallelConfig {
    //         enabled: true,
    //         num_threads: 2,
    //         batch_size: 5,
    //     };
    //
    //     let test = ParallelAsyncPropertyTest::new(generator, property, config, parallel_config);
    //     let result = test.run().await;
    //
    //     assert!(result.is_ok());
    //     assert_eq!(counter.load(Ordering::SeqCst), 20);
    // }

    // Note: check_async_parallel function removed as it required tokio::spawn
    // Users can implement their own parallel async execution if needed
    // #[tokio::test]
    // async fn test_check_async_parallel_function() {
    //     let generator = ConstantGenerator::new(42);
    //     let (property, counter) = AsyncCountingProperty::new();
    //     let config = TestConfig {
    //         iterations: 15,
    //         ..TestConfig::default()
    //     };
    //     let parallel_config = ParallelConfig::default();
    //
    //     let result = check_async_parallel(generator, property, config, parallel_config).await;
    //
    //     assert!(result.is_ok());
    //     assert_eq!(counter.load(Ordering::SeqCst), 15);
    // }
}
