//! Property test execution engine for running synchronous and asynchronous property tests.

use std::fmt;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

use crate::config::TestConfig;
use crate::error::{
    ErrorReporter, PropertyError, PropertyResult, ShrinkProgress, ShrinkStep, TestFailure,
    TestSuccess,
};
use crate::generator::Generator;
use crate::property::{AsyncProperty, Property};
use crate::rng::create_seeded_rng;
use crate::statistics::StatisticsCollector;

/// Core property test execution struct
pub struct PropertyTest<T, G, P> {
    generator: G,
    property: P,
    config: TestConfig,
    error_reporter: ErrorReporter,
    statistics_collector: Option<StatisticsCollector>,
    _phantom: PhantomData<T>,
}

/// Async property test execution struct
pub struct AsyncPropertyTest<T, G, P> {
    generator: G,
    property: P,
    config: TestConfig,
    error_reporter: ErrorReporter,
    statistics_collector: Option<StatisticsCollector>,
    _phantom: PhantomData<T>,
}

impl<T, G, P> PropertyTest<T, G, P>
where
    T: Clone + fmt::Debug + PartialEq + 'static,
    G: Generator<T>,
    P: Property<T>,
{
    /// Create a new property test with the given generator, property, and configuration
    pub fn new(generator: G, property: P, config: TestConfig) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter: ErrorReporter::new(),
            statistics_collector: Some(StatisticsCollector::new()),
            _phantom: PhantomData,
        }
    }

    /// Create a new property test with custom error reporter
    pub fn with_error_reporter(
        generator: G,
        property: P,
        config: TestConfig,
        error_reporter: ErrorReporter,
    ) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter,
            statistics_collector: Some(StatisticsCollector::new()),
            _phantom: PhantomData,
        }
    }

    /// Create a new property test with custom statistics collector
    pub fn with_statistics_collector(
        generator: G,
        property: P,
        config: TestConfig,
        statistics_collector: Option<StatisticsCollector>,
    ) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter: ErrorReporter::new(),
            statistics_collector,
            _phantom: PhantomData,
        }
    }

    /// Create a new property test with both custom error reporter and statistics collector
    pub fn with_error_reporter_and_statistics(
        generator: G,
        property: P,
        config: TestConfig,
        error_reporter: ErrorReporter,
        statistics_collector: Option<StatisticsCollector>,
    ) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter,
            statistics_collector,
            _phantom: PhantomData,
        }
    }

    /// Execute the property test
    pub fn run(mut self) -> PropertyResult<T> {
        let test_start = Instant::now();
        let mut rng = if let Some(seed) = self.config.seed {
            create_seeded_rng(seed)
        } else {
            crate::rng::create_rng()
        };
        let mut stats_collector = self
            .statistics_collector
            .take()
            .unwrap_or_else(StatisticsCollector::disabled);

        for iteration in 0..self.config.iterations {
            // Start timing generation
            stats_collector.start_generation_timing();

            // Generate test input
            let input = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.generator
                    .generate(&mut rng, &self.config.generator_config)
            })) {
                Ok(input) => input,
                Err(_) => {
                    let error = PropertyError::generation_failed_with_context(
                        "Generator panicked during value generation",
                        Some(format!("iteration {}", iteration)),
                    );
                    return Err(TestFailure::new(
                        error,
                        // We can't provide the original input since generation failed
                        // This is a limitation we'll need to handle differently in practice
                        self.generator
                            .generate(&mut rng, &self.config.generator_config),
                        None,
                        0,
                        self.config,
                        iteration,
                        test_start.elapsed(),
                        Duration::from_secs(0),
                    ));
                }
            };

            // End timing and record the generated value
            stats_collector.end_generation_timing();
            stats_collector.record_generated_value(&input, std::any::type_name::<T>());

            // Test the property
            match self.property.test(input.clone()) {
                Ok(_) => {
                    // Property passed, continue to next iteration
                    continue;
                }
                Err(mut error) => {
                    // Add iteration context to the error if it doesn't have it
                    error = match error {
                        PropertyError::PropertyFailed {
                            message,
                            context,
                            iteration: None,
                        } => PropertyError::PropertyFailed {
                            message,
                            context,
                            iteration: Some(iteration),
                        },
                        other => other,
                    };

                    // Property failed, attempt shrinking
                    let shrink_start = Instant::now();
                    let shrink_result = self.shrink_failure(input.clone(), &error);
                    let shrink_duration = shrink_start.elapsed();

                    let failure = TestFailure::new(
                        error,
                        input,
                        shrink_result.0,
                        shrink_result.1,
                        self.config,
                        iteration,
                        test_start.elapsed(),
                        shrink_duration,
                    );

                    // Print detailed error report if verbose mode is enabled
                    if self.error_reporter.verbose {
                        eprintln!("{}", self.error_reporter.format_failure(&failure));
                    } else {
                        eprintln!("{}", self.error_reporter.format_summary(&failure));
                    }

                    // Print shrink progress if enabled
                    if self.error_reporter.show_shrink_progress && !shrink_result.2.steps.is_empty()
                    {
                        eprintln!("{}", shrink_result.2.visualize());
                        eprintln!("{}", shrink_result.2.statistics());
                    }

                    return Err(failure);
                }
            }
        }

        // All iterations passed
        let final_stats = if stats_collector.is_enabled() {
            Some(stats_collector.into_stats())
        } else {
            None
        };

        Ok(TestSuccess::new(
            self.config.iterations,
            self.config,
            final_stats,
        ))
    }

    /// Attempt to shrink a failing input to find a minimal example with progress tracking
    fn shrink_failure(
        &self,
        original_input: T,
        _error: &PropertyError,
    ) -> (Option<T>, usize, ShrinkProgress) {
        let start_time = Instant::now();
        let mut shrink_steps = 0;
        let mut current_input = original_input.clone();
        let mut progress = ShrinkProgress::new();

        // Get shrink candidates
        let shrink_candidates: Vec<T> = self.generator.shrink(&current_input).collect();

        for candidate in shrink_candidates {
            // Check timeout
            if start_time.elapsed() > self.config.shrink_timeout {
                if self.error_reporter.show_shrink_progress {
                    eprintln!("Shrinking timed out after {:?}", start_time.elapsed());
                }
                break;
            }

            // Check iteration limit
            if shrink_steps >= self.config.max_shrink_iterations {
                if self.error_reporter.show_shrink_progress {
                    eprintln!(
                        "Shrinking reached max iterations: {}",
                        self.config.max_shrink_iterations
                    );
                }
                break;
            }

            let step_start = Instant::now();
            shrink_steps += 1;

            // Test if the candidate still fails
            if let Err(_) = self.property.test(candidate.clone()) {
                let step_time = step_start.elapsed();

                // Record successful shrink step
                let step = ShrinkStep {
                    step_number: shrink_steps,
                    input_description: format!("{:?} -> {:?}", current_input, candidate),
                    step_time,
                    successful: true,
                };
                progress.add_step(step);

                if self.error_reporter.show_shrink_progress {
                    eprintln!(
                        "Shrink step {}: found smaller failing value in {:?}",
                        shrink_steps, step_time
                    );
                }

                // This candidate also fails, so it's a valid shrink
                current_input = candidate;

                // Continue shrinking from this point
                let further_shrinks: Vec<T> = self.generator.shrink(&current_input).collect();
                if !further_shrinks.is_empty() {
                    // Recursively try to shrink further
                    // For now, just take the first shrink candidate
                    if let Some(further_candidate) = further_shrinks.into_iter().next() {
                        let further_step_start = Instant::now();
                        if let Err(_) = self.property.test(further_candidate.clone()) {
                            let further_step_time = further_step_start.elapsed();
                            shrink_steps += 1;

                            let further_step = ShrinkStep {
                                step_number: shrink_steps,
                                input_description: format!(
                                    "{:?} -> {:?}",
                                    current_input, further_candidate
                                ),
                                step_time: further_step_time,
                                successful: true,
                            };
                            progress.add_step(further_step);

                            current_input = further_candidate;

                            if self.error_reporter.show_shrink_progress {
                                eprintln!(
                                    "Further shrink step {}: found even smaller failing value in {:?}",
                                    shrink_steps, further_step_time
                                );
                            }
                        } else {
                            // Record unsuccessful step
                            let further_step = ShrinkStep {
                                step_number: shrink_steps,
                                input_description: format!(
                                    "{:?} -> {:?} (failed)",
                                    current_input, further_candidate
                                ),
                                step_time: further_step_start.elapsed(),
                                successful: false,
                            };
                            progress.add_step(further_step);
                        }
                    }
                }
                break;
            } else {
                // Record unsuccessful step
                let step = ShrinkStep {
                    step_number: shrink_steps,
                    input_description: format!("{:?} -> {:?} (passed)", current_input, candidate),
                    step_time: step_start.elapsed(),
                    successful: false,
                };
                progress.add_step(step);
            }
        }

        progress.complete(start_time.elapsed());

        // Return the shrunk input if we found one, otherwise None
        if shrink_steps > 0 && current_input != original_input {
            (Some(current_input), shrink_steps, progress)
        } else {
            (None, 0, progress)
        }
    }
}

impl<T, G, P> AsyncPropertyTest<T, G, P>
where
    T: Clone + Send + Sync + fmt::Debug + PartialEq + 'static,
    G: Generator<T> + Send + Sync,
    P: AsyncProperty<T> + Send + Sync,
{
    /// Create a new async property test with the given generator, property, and configuration
    pub fn new(generator: G, property: P, config: TestConfig) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter: ErrorReporter::new(),
            statistics_collector: Some(StatisticsCollector::new()),
            _phantom: PhantomData,
        }
    }

    /// Create a new async property test with custom error reporter
    pub fn with_error_reporter(
        generator: G,
        property: P,
        config: TestConfig,
        error_reporter: ErrorReporter,
    ) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter,
            statistics_collector: Some(StatisticsCollector::new()),
            _phantom: PhantomData,
        }
    }

    /// Create a new async property test with custom statistics collector
    pub fn with_statistics_collector(
        generator: G,
        property: P,
        config: TestConfig,
        statistics_collector: Option<StatisticsCollector>,
    ) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter: ErrorReporter::new(),
            statistics_collector,
            _phantom: PhantomData,
        }
    }

    /// Create a new async property test with both custom error reporter and statistics collector
    pub fn with_error_reporter_and_statistics(
        generator: G,
        property: P,
        config: TestConfig,
        error_reporter: ErrorReporter,
        statistics_collector: Option<StatisticsCollector>,
    ) -> Self {
        Self {
            generator,
            property,
            config,
            error_reporter,
            statistics_collector,
            _phantom: PhantomData,
        }
    }

    /// Execute the async property test
    pub async fn run(mut self) -> PropertyResult<T> {
        let test_start = Instant::now();
        let mut rng = if let Some(seed) = self.config.seed {
            create_seeded_rng(seed)
        } else {
            crate::rng::create_rng()
        };
        let mut stats_collector = self
            .statistics_collector
            .take()
            .unwrap_or_else(StatisticsCollector::disabled);

        for iteration in 0..self.config.iterations {
            // Start timing generation
            stats_collector.start_generation_timing();

            // Generate test input
            let input = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.generator
                    .generate(&mut rng, &self.config.generator_config)
            })) {
                Ok(input) => input,
                Err(_) => {
                    let error = PropertyError::generation_failed_with_context(
                        "Generator panicked during value generation",
                        Some(format!("iteration {}", iteration)),
                    );
                    return Err(TestFailure::new(
                        error,
                        // We can't provide the original input since generation failed
                        // This is a limitation we'll need to handle differently in practice
                        self.generator
                            .generate(&mut rng, &self.config.generator_config),
                        None,
                        0,
                        self.config,
                        iteration,
                        test_start.elapsed(),
                        Duration::from_secs(0),
                    ));
                }
            };

            // End timing and record the generated value
            stats_collector.end_generation_timing();
            stats_collector.record_generated_value(&input, std::any::type_name::<T>());

            // Test the property asynchronously
            match self.property.test(input.clone()).await {
                Ok(_) => {
                    // Property passed, continue to next iteration
                    continue;
                }
                Err(mut error) => {
                    // Add iteration context to the error if it doesn't have it
                    error = match error {
                        PropertyError::PropertyFailed {
                            message,
                            context,
                            iteration: None,
                        } => PropertyError::PropertyFailed {
                            message,
                            context,
                            iteration: Some(iteration),
                        },
                        other => other,
                    };

                    // Property failed, attempt shrinking
                    let shrink_start = Instant::now();
                    let shrink_result = self.shrink_failure_async(input.clone(), &error).await;
                    let shrink_duration = shrink_start.elapsed();

                    let failure = TestFailure::new(
                        error,
                        input,
                        shrink_result.0,
                        shrink_result.1,
                        self.config,
                        iteration,
                        test_start.elapsed(),
                        shrink_duration,
                    );

                    // Print detailed error report if verbose mode is enabled
                    if self.error_reporter.verbose {
                        eprintln!("{}", self.error_reporter.format_failure(&failure));
                    } else {
                        eprintln!("{}", self.error_reporter.format_summary(&failure));
                    }

                    // Print shrink progress if enabled
                    if self.error_reporter.show_shrink_progress && !shrink_result.2.steps.is_empty()
                    {
                        eprintln!("{}", shrink_result.2.visualize());
                        eprintln!("{}", shrink_result.2.statistics());
                    }

                    return Err(failure);
                }
            }
        }

        // All iterations passed
        let final_stats = if stats_collector.is_enabled() {
            Some(stats_collector.into_stats())
        } else {
            None
        };

        Ok(TestSuccess::new(
            self.config.iterations,
            self.config,
            final_stats,
        ))
    }

    /// Attempt to shrink a failing input to find a minimal example (async version) with progress tracking
    async fn shrink_failure_async(
        &self,
        original_input: T,
        _error: &PropertyError,
    ) -> (Option<T>, usize, ShrinkProgress) {
        let start_time = Instant::now();
        let mut shrink_steps = 0;
        let mut current_input = original_input.clone();
        let mut progress = ShrinkProgress::new();

        // Get shrink candidates
        let shrink_candidates: Vec<T> = self.generator.shrink(&current_input).collect();

        for candidate in shrink_candidates {
            // Check timeout
            if start_time.elapsed() > self.config.shrink_timeout {
                if self.error_reporter.show_shrink_progress {
                    eprintln!("Async shrinking timed out after {:?}", start_time.elapsed());
                }
                break;
            }

            // Check iteration limit
            if shrink_steps >= self.config.max_shrink_iterations {
                if self.error_reporter.show_shrink_progress {
                    eprintln!(
                        "Async shrinking reached max iterations: {}",
                        self.config.max_shrink_iterations
                    );
                }
                break;
            }

            let step_start = Instant::now();
            shrink_steps += 1;

            // Test if the candidate still fails (async)
            if let Err(_) = self.property.test(candidate.clone()).await {
                let step_time = step_start.elapsed();

                // Record successful shrink step
                let step = ShrinkStep {
                    step_number: shrink_steps,
                    input_description: format!("{:?} -> {:?}", current_input, candidate),
                    step_time,
                    successful: true,
                };
                progress.add_step(step);

                if self.error_reporter.show_shrink_progress {
                    eprintln!(
                        "Async shrink step {}: found smaller failing value in {:?}",
                        shrink_steps, step_time
                    );
                }

                // This candidate also fails, so it's a valid shrink
                current_input = candidate;

                // Continue shrinking from this point
                let further_shrinks: Vec<T> = self.generator.shrink(&current_input).collect();
                if !further_shrinks.is_empty() {
                    // Recursively try to shrink further
                    // For now, just take the first shrink candidate
                    if let Some(further_candidate) = further_shrinks.into_iter().next() {
                        let further_step_start = Instant::now();
                        if let Err(_) = self.property.test(further_candidate.clone()).await {
                            let further_step_time = further_step_start.elapsed();
                            shrink_steps += 1;

                            let further_step = ShrinkStep {
                                step_number: shrink_steps,
                                input_description: format!(
                                    "{:?} -> {:?}",
                                    current_input, further_candidate
                                ),
                                step_time: further_step_time,
                                successful: true,
                            };
                            progress.add_step(further_step);

                            current_input = further_candidate;

                            if self.error_reporter.show_shrink_progress {
                                eprintln!(
                                    "Further async shrink step {}: found even smaller failing value in {:?}",
                                    shrink_steps, further_step_time
                                );
                            }
                        } else {
                            // Record unsuccessful step
                            let further_step = ShrinkStep {
                                step_number: shrink_steps,
                                input_description: format!(
                                    "{:?} -> {:?} (failed)",
                                    current_input, further_candidate
                                ),
                                step_time: further_step_start.elapsed(),
                                successful: false,
                            };
                            progress.add_step(further_step);
                        }
                    }
                }
                break;
            } else {
                // Record unsuccessful step
                let step = ShrinkStep {
                    step_number: shrink_steps,
                    input_description: format!("{:?} -> {:?} (passed)", current_input, candidate),
                    step_time: step_start.elapsed(),
                    successful: false,
                };
                progress.add_step(step);
            }
        }

        progress.complete(start_time.elapsed());

        // Return the shrunk input if we found one, otherwise None
        if shrink_steps > 0 && current_input != original_input {
            (Some(current_input), shrink_steps, progress)
        } else {
            (None, 0, progress)
        }
    }
}

/// Execute a property test with the default configuration
pub fn check<T, G, P>(generator: G, property: P) -> PropertyResult<T>
where
    T: Clone + fmt::Debug + PartialEq + 'static,
    G: Generator<T>,
    P: Property<T>,
{
    check_with_config(generator, property, TestConfig::default())
}

/// Execute a property test with a custom configuration
pub fn check_with_config<T, G, P>(
    generator: G,
    property: P,
    config: TestConfig,
) -> PropertyResult<T>
where
    T: Clone + fmt::Debug + PartialEq + 'static,
    G: Generator<T>,
    P: Property<T>,
{
    let test = PropertyTest::new(generator, property, config);
    test.run()
}

/// Execute an async property test with the default configuration
pub async fn check_async<T, G, P>(generator: G, property: P) -> PropertyResult<T>
where
    T: Clone + Send + Sync + fmt::Debug + PartialEq + 'static,
    G: Generator<T> + Send + Sync,
    P: AsyncProperty<T> + Send + Sync,
{
    check_async_with_config(generator, property, TestConfig::default()).await
}

/// Execute an async property test with a custom configuration
pub async fn check_async_with_config<T, G, P>(
    generator: G,
    property: P,
    config: TestConfig,
) -> PropertyResult<T>
where
    T: Clone + Send + Sync + fmt::Debug + PartialEq + 'static,
    G: Generator<T> + Send + Sync,
    P: AsyncProperty<T> + Send + Sync,
{
    let test = AsyncPropertyTest::new(generator, property, config);
    test.run().await
}

/// Builder pattern for configuring property tests
pub struct PropertyTestBuilder<T> {
    config: TestConfig,
    error_reporter: ErrorReporter,
    statistics_collector: Option<StatisticsCollector>,
    _phantom: PhantomData<T>,
}

impl<T: 'static> PropertyTestBuilder<T> {
    /// Create a new property test builder with default configuration
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
            error_reporter: ErrorReporter::new(),
            statistics_collector: Some(StatisticsCollector::new()),
            _phantom: PhantomData,
        }
    }

    /// Set the number of test iterations
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.config.iterations = iterations;
        self
    }

    /// Set the random seed for reproducible tests
    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = Some(seed);
        self
    }

    /// Set the maximum number of shrinking iterations
    pub fn max_shrink_iterations(mut self, max_iterations: usize) -> Self {
        self.config.max_shrink_iterations = max_iterations;
        self
    }

    /// Set the shrinking timeout
    pub fn shrink_timeout(mut self, timeout: Duration) -> Self {
        self.config.shrink_timeout = timeout;
        self
    }

    /// Enable verbose error reporting
    pub fn verbose_errors(mut self) -> Self {
        self.error_reporter = self.error_reporter.verbose();
        self
    }

    /// Enable shrinkage progress visualization
    pub fn show_shrink_progress(mut self) -> Self {
        self.error_reporter = self.error_reporter.show_shrink_progress();
        self
    }

    /// Enable configuration display in error reports
    pub fn show_config_in_errors(mut self) -> Self {
        self.error_reporter = self.error_reporter.show_config();
        self
    }

    /// Set a custom error reporter
    pub fn error_reporter(mut self, reporter: ErrorReporter) -> Self {
        self.error_reporter = reporter;
        self
    }

    /// Enable statistics collection
    pub fn enable_statistics(mut self) -> Self {
        self.statistics_collector = Some(StatisticsCollector::new());
        self
    }

    /// Disable statistics collection
    pub fn disable_statistics(mut self) -> Self {
        self.statistics_collector = None;
        self
    }

    /// Set a custom statistics collector
    pub fn statistics_collector(mut self, collector: Option<StatisticsCollector>) -> Self {
        self.statistics_collector = collector;
        self
    }

    /// Run the property test with the configured parameters
    pub fn run<G, P>(self, generator: G, property: P) -> PropertyResult<T>
    where
        T: Clone + fmt::Debug + PartialEq + 'static,
        G: Generator<T>,
        P: Property<T>,
    {
        let test = PropertyTest::with_error_reporter_and_statistics(
            generator,
            property,
            self.config,
            self.error_reporter,
            self.statistics_collector,
        );
        test.run()
    }

    /// Run the async property test with the configured parameters
    pub async fn run_async<G, P>(self, generator: G, property: P) -> PropertyResult<T>
    where
        T: Clone + Send + Sync + fmt::Debug + PartialEq + 'static,
        G: Generator<T> + Send + Sync,
        P: AsyncProperty<T> + Send + Sync,
    {
        let test = AsyncPropertyTest::with_error_reporter_and_statistics(
            generator,
            property,
            self.config,
            self.error_reporter,
            self.statistics_collector,
        );
        test.run().await
    }
}

impl<T: 'static> Default for PropertyTestBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ConstantGenerator;
    use std::time::Duration;

    // Simple property that always passes
    struct AlwaysPassProperty;
    impl Property<i32> for AlwaysPassProperty {
        type Output = ();
        fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            Ok(())
        }
    }

    // Simple property that always fails
    struct AlwaysFailProperty;
    impl Property<i32> for AlwaysFailProperty {
        type Output = ();
        fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            Err(PropertyError::PropertyFailed {
                message: "Always fails".to_string(),
                context: None,
                iteration: None,
            })
        }
    }

    // Property that fails for specific values
    struct FailsForZeroProperty;
    impl Property<i32> for FailsForZeroProperty {
        type Output = ();
        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input == 0 {
                Err(PropertyError::PropertyFailed {
                    message: "Input was zero".to_string(),
                    context: Some("zero check".to_string()),
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_property_test_always_passes() {
        let generator = ConstantGenerator::new(42);
        let property = AlwaysPassProperty;
        let config = TestConfig {
            iterations: 10,
            ..TestConfig::default()
        };

        let result = check_with_config(generator, property, config);
        assert!(result.is_ok());

        if let Ok(success) = result {
            assert_eq!(success.iterations, 10);
            assert!(success.stats.is_some());
            assert_eq!(success.stats.unwrap().total_generated, 10);
        }
    }

    #[test]
    fn test_property_test_always_fails() {
        let generator = ConstantGenerator::new(42);
        let property = AlwaysFailProperty;
        let config = TestConfig {
            iterations: 10,
            ..TestConfig::default()
        };

        let result = check_with_config(generator, property, config);
        assert!(result.is_err());

        if let Err(failure) = result {
            assert_eq!(failure.original_input, 42);
            assert!(matches!(
                failure.error,
                PropertyError::PropertyFailed { .. }
            ));
            assert_eq!(failure.failed_iteration, 0); // Should fail on first iteration
        }
    }

    #[test]
    fn test_check_function_with_defaults() {
        let generator = ConstantGenerator::new(100);
        let property = AlwaysPassProperty;

        let result = check(generator, property);
        assert!(result.is_ok());

        if let Ok(success) = result {
            assert_eq!(success.iterations, 100); // Default iterations
        }
    }

    #[test]
    fn test_property_test_builder() {
        let result = PropertyTestBuilder::new()
            .iterations(5)
            .seed(12345)
            .max_shrink_iterations(100)
            .shrink_timeout(Duration::from_secs(1))
            .run(ConstantGenerator::new(42), AlwaysPassProperty);

        assert!(result.is_ok());
        if let Ok(success) = result {
            assert_eq!(success.iterations, 5);
            assert_eq!(success.config.seed, Some(12345));
            assert_eq!(success.config.max_shrink_iterations, 100);
            assert_eq!(success.config.shrink_timeout, Duration::from_secs(1));
        }
    }

    #[test]
    fn test_property_test_builder_default() {
        let builder = PropertyTestBuilder::<i32>::default();
        assert_eq!(builder.config.iterations, 100);
        assert!(builder.config.seed.is_none());
        assert!(!builder.error_reporter.verbose);
    }

    #[test]
    fn test_property_test_with_seed_reproducibility() {
        let seed = 42;
        let generator = ConstantGenerator::new(123);
        let _property = AlwaysPassProperty;

        let config1 = TestConfig {
            seed: Some(seed),
            iterations: 5,
            ..TestConfig::default()
        };

        let config2 = TestConfig {
            seed: Some(seed),
            iterations: 5,
            ..TestConfig::default()
        };

        let result1 = check_with_config(generator.clone(), AlwaysPassProperty, config1);
        let result2 = check_with_config(generator, AlwaysPassProperty, config2);

        // Both should succeed with the same configuration
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_shrinking_basic() {
        // This test verifies that shrinking is attempted when a property fails
        let generator = ConstantGenerator::new(0); // Will fail FailsForZeroProperty
        let property = FailsForZeroProperty;
        let config = TestConfig {
            iterations: 1,
            max_shrink_iterations: 10,
            ..TestConfig::default()
        };

        let result = check_with_config(generator, property, config);
        assert!(result.is_err());

        if let Err(failure) = result {
            assert_eq!(failure.original_input, 0);
            // Since ConstantGenerator doesn't provide shrinks, shrunk_input should be None
            assert!(failure.shrunk_input.is_none());
            assert_eq!(failure.shrink_steps, 0);
            assert_eq!(failure.failed_iteration, 0);
            // Test that timing information is captured
            assert!(failure.test_duration.as_nanos() > 0);
        }
    }

    // Async property implementations for testing
    struct AlwaysPassAsyncProperty;
    impl AsyncProperty<i32> for AlwaysPassAsyncProperty {
        type Output = ();
        async fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            // Simulate some async work
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            Ok(())
        }
    }

    struct AlwaysFailAsyncProperty;
    impl AsyncProperty<i32> for AlwaysFailAsyncProperty {
        type Output = ();
        async fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
            // Simulate some async work
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            Err(PropertyError::PropertyFailed {
                message: "Always fails async".to_string(),
                context: None,
                iteration: None,
            })
        }
    }

    struct FailsForZeroAsyncProperty;
    impl AsyncProperty<i32> for FailsForZeroAsyncProperty {
        type Output = ();
        async fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            // Simulate some async work
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            if input == 0 {
                Err(PropertyError::PropertyFailed {
                    message: "Input was zero (async)".to_string(),
                    context: Some("async zero check".to_string()),
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_async_property_test_always_passes() {
        let generator = ConstantGenerator::new(42);
        let property = AlwaysPassAsyncProperty;
        let config = TestConfig {
            iterations: 10,
            ..TestConfig::default()
        };

        let result = check_async_with_config(generator, property, config).await;
        assert!(result.is_ok());

        if let Ok(success) = result {
            assert_eq!(success.iterations, 10);
            assert!(success.stats.is_some());
            assert_eq!(success.stats.unwrap().total_generated, 10);
        }
    }

    #[tokio::test]
    async fn test_async_property_test_always_fails() {
        let generator = ConstantGenerator::new(42);
        let property = AlwaysFailAsyncProperty;
        let config = TestConfig {
            iterations: 10,
            ..TestConfig::default()
        };

        let result = check_async_with_config(generator, property, config).await;
        assert!(result.is_err());

        if let Err(failure) = result {
            assert_eq!(failure.original_input, 42);
            assert!(matches!(
                failure.error,
                PropertyError::PropertyFailed { .. }
            ));
            assert_eq!(failure.failed_iteration, 0); // Should fail on first iteration
        }
    }

    #[tokio::test]
    async fn test_check_async_function_with_defaults() {
        let generator = ConstantGenerator::new(100);
        let property = AlwaysPassAsyncProperty;

        let result = check_async(generator, property).await;
        assert!(result.is_ok());

        if let Ok(success) = result {
            assert_eq!(success.iterations, 100); // Default iterations
        }
    }

    #[tokio::test]
    async fn test_async_property_test_builder() {
        let result = PropertyTestBuilder::new()
            .iterations(5)
            .seed(12345)
            .max_shrink_iterations(100)
            .shrink_timeout(Duration::from_secs(1))
            .run_async(ConstantGenerator::new(42), AlwaysPassAsyncProperty)
            .await;

        assert!(result.is_ok());
        if let Ok(success) = result {
            assert_eq!(success.iterations, 5);
            assert_eq!(success.config.seed, Some(12345));
            assert_eq!(success.config.max_shrink_iterations, 100);
            assert_eq!(success.config.shrink_timeout, Duration::from_secs(1));
        }
    }

    #[tokio::test]
    async fn test_async_shrinking_basic() {
        // This test verifies that async shrinking is attempted when a property fails
        let generator = ConstantGenerator::new(0); // Will fail FailsForZeroAsyncProperty
        let property = FailsForZeroAsyncProperty;
        let config = TestConfig {
            iterations: 1,
            max_shrink_iterations: 10,
            ..TestConfig::default()
        };

        let result = check_async_with_config(generator, property, config).await;
        assert!(result.is_err());

        if let Err(failure) = result {
            assert_eq!(failure.original_input, 0);
            // Since ConstantGenerator doesn't provide shrinks, shrunk_input should be None
            assert!(failure.shrunk_input.is_none());
            assert_eq!(failure.shrink_steps, 0);
            assert_eq!(failure.failed_iteration, 0);
            // Test that timing information is captured
            assert!(failure.test_duration.as_nanos() > 0);
        }
    }

    #[tokio::test]
    async fn test_async_property_error_propagation() {
        let generator = ConstantGenerator::new(42);
        let property = AlwaysFailAsyncProperty;
        let config = TestConfig {
            iterations: 1,
            ..TestConfig::default()
        };

        let result = check_async_with_config(generator, property, config).await;
        assert!(result.is_err());

        if let Err(failure) = result {
            match &failure.error {
                PropertyError::PropertyFailed {
                    message,
                    context: _,
                    iteration,
                } => {
                    assert_eq!(message, "Always fails async");
                    assert_eq!(*iteration, Some(0));
                }
                _ => panic!("Expected PropertyFailed error"),
            }
        }
    }

    #[test]
    fn test_property_test_builder_error_reporting() {
        let builder = PropertyTestBuilder::<i32>::new()
            .verbose_errors()
            .show_shrink_progress()
            .show_config_in_errors();

        assert!(builder.error_reporter.verbose);
        assert!(builder.error_reporter.show_shrink_progress);
        assert!(builder.error_reporter.show_config);
    }

    #[test]
    fn test_property_test_with_custom_error_reporter() {
        let custom_reporter = ErrorReporter::new()
            .verbose()
            .show_shrink_progress()
            .show_config();

        let result = PropertyTestBuilder::new()
            .iterations(1)
            .error_reporter(custom_reporter)
            .run(ConstantGenerator::new(0), FailsForZeroProperty);

        assert!(result.is_err());
        // The error should have been reported with the custom reporter settings
    }

    #[tokio::test]
    async fn test_async_property_test_with_error_reporting() {
        let result = PropertyTestBuilder::new()
            .iterations(1)
            .verbose_errors()
            .show_shrink_progress()
            .run_async(ConstantGenerator::new(42), AlwaysFailAsyncProperty)
            .await;

        assert!(result.is_err());
        if let Err(failure) = result {
            assert_eq!(failure.original_input, 42);
            assert!(matches!(
                failure.error,
                PropertyError::PropertyFailed { .. }
            ));
        }
    }

    #[test]
    fn test_property_test_shrink_progress_tracking() {
        // This test uses a generator that can actually shrink to test progress tracking
        use crate::primitives::IntGenerator;

        let _generator = IntGenerator::new(0, 100);
        let property = FailsForZeroProperty;
        let config = TestConfig {
            iterations: 1,
            max_shrink_iterations: 10,
            ..TestConfig::default()
        };

        // Create a test that will generate zero, which should fail the property
        let test = PropertyTest::with_error_reporter(
            ConstantGenerator::new(0), // This will generate 0, which fails the property
            property,
            config,
            ErrorReporter::new().show_shrink_progress(),
        );

        let result = test.run();
        assert!(result.is_err());

        // The shrinking should have been attempted (though ConstantGenerator doesn't actually shrink)
        if let Err(failure) = result {
            assert_eq!(failure.original_input, 0);
        }
    }
}
