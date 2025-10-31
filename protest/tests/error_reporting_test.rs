//! Comprehensive tests for error reporting functionality

#![allow(dead_code)]

use protest::config::TestConfig;
use protest::error::{ErrorReporter, PropertyError, ShrinkProgress, ShrinkStep, TestFailure};
use protest::execution::{PropertyTestBuilder, check_with_config};
use protest::generator::{ConstantGenerator, Generator};
use protest::property::Property;
use std::time::Duration;

// Test property that always fails
struct AlwaysFailProperty;
impl Property<i32> for AlwaysFailProperty {
    type Output = ();
    fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
        Err(PropertyError::PropertyFailed {
            message: "Test always fails".to_string(),
            context: Some("for testing purposes".to_string()),
            iteration: None,
        })
    }
}

// Test property that fails for specific values
struct FailsForSpecificValueProperty {
    failing_value: i32,
}

impl FailsForSpecificValueProperty {
    fn new(failing_value: i32) -> Self {
        Self { failing_value }
    }
}

impl Property<i32> for FailsForSpecificValueProperty {
    type Output = ();
    fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
        if input == self.failing_value {
            Err(PropertyError::PropertyFailed {
                message: format!("Input equals failing value {}", self.failing_value),
                context: Some("specific value test".to_string()),
                iteration: None,
            })
        } else {
            Ok(())
        }
    }
}

// Generator that can actually shrink for testing shrinkage visualization
struct ShrinkableIntGenerator {
    value: i32,
}

impl ShrinkableIntGenerator {
    fn new(value: i32) -> Self {
        Self { value }
    }
}

impl Generator<i32> for ShrinkableIntGenerator {
    fn generate(
        &self,
        _rng: &mut dyn rand::RngCore,
        _config: &protest::config::GeneratorConfig,
    ) -> i32 {
        self.value
    }

    fn shrink(&self, value: &i32) -> Box<dyn Iterator<Item = i32>> {
        let mut candidates = Vec::new();

        // Always try 0 first if the value is not 0
        if *value != 0 {
            candidates.push(0);
        }

        // Try values closer to 0
        if *value > 1 {
            candidates.push(value / 2);
            candidates.push(value - 1);
        } else if *value < -1 {
            candidates.push(value / 2);
            candidates.push(value + 1);
        }

        Box::new(candidates.into_iter())
    }
}

#[test]
fn test_error_reporter_basic_functionality() {
    let reporter = ErrorReporter::new();

    // Test default settings
    assert!(!reporter.verbose);
    assert!(!reporter.show_shrink_progress);
    assert!(reporter.show_timing);
    assert!(!reporter.show_config);
}

#[test]
fn test_error_reporter_builder_pattern() {
    let reporter = ErrorReporter::new()
        .verbose()
        .show_shrink_progress()
        .show_config();

    assert!(reporter.verbose);
    assert!(reporter.show_shrink_progress);
    assert!(reporter.show_timing);
    assert!(reporter.show_config);
}

#[test]
fn test_comprehensive_error_report_formatting() {
    let error = PropertyError::PropertyFailed {
        message: "Assertion failed: x > 0".to_string(),
        context: Some("testing positive numbers".to_string()),
        iteration: Some(42),
    };

    let config = TestConfig {
        iterations: 100,
        seed: Some(12345),
        max_shrink_iterations: 1000,
        shrink_timeout: Duration::from_secs(5),
        ..TestConfig::default()
    };

    let failure = TestFailure::new(
        error,
        -50,
        Some(-1),
        15,
        config,
        42,
        Duration::from_millis(250),
        Duration::from_millis(100),
    );

    let reporter = ErrorReporter::new()
        .verbose()
        .show_shrink_progress()
        .show_config();

    let report = reporter.format_failure(&failure);

    // Check that all expected sections are present
    assert!(report.contains("PROPERTY TEST FAILURE"));
    assert!(report.contains("Test failed on iteration 42"));
    assert!(report.contains("Assertion failed: x > 0"));
    assert!(report.contains("Original input: -50"));
    assert!(report.contains("Shrunk input:   -1"));
    assert!(report.contains("Shrink steps:   15"));
    assert!(report.contains("TIMING INFORMATION"));
    assert!(report.contains("Total test time:"));
    assert!(report.contains("Shrinking time:"));
    assert!(report.contains("CONFIGURATION"));
    assert!(report.contains("Iterations:       100"));
    assert!(report.contains("Seed:             Some(12345)"));
    assert!(report.contains("DETAILED ERROR CONTEXT"));
    assert!(report.contains("Type: Property assertion failure"));
    assert!(report.contains("Context: testing positive numbers"));
    assert!(report.contains("SUGGESTIONS"));
    assert!(report.contains("Check if your property logic is correct"));
    assert!(report.contains("Focus on the shrunk input"));
}

#[test]
fn test_error_report_without_shrinking() {
    let error = PropertyError::PropertyFailed {
        message: "Test failed".to_string(),
        context: None,
        iteration: None,
    };

    let config = TestConfig::default();
    let failure = TestFailure::new(
        error,
        42,
        None,
        0,
        config,
        0,
        Duration::from_millis(50),
        Duration::from_millis(0),
    );

    let reporter = ErrorReporter::new().verbose();
    let report = reporter.format_failure(&failure);

    assert!(report.contains("No shrinking performed"));
    assert!(report.contains("Consider implementing shrinking"));
}

#[test]
fn test_different_error_types_formatting() {
    let reporter = ErrorReporter::new().verbose();

    // Test GenerationFailed error
    let gen_error = PropertyError::GenerationFailed {
        message: "Generator panicked".to_string(),
        context: Some("while generating integers".to_string()),
    };
    let context = reporter.format_error_context(&gen_error);
    assert!(context.contains("Test data generation failure"));
    assert!(context.contains("Generator panicked"));
    assert!(context.contains("while generating integers"));

    // Test ShrinkageTimeout error
    let shrink_error = PropertyError::ShrinkageTimeout {
        iterations: 1000,
        last_successful_shrink: Some("42".to_string()),
    };
    let context = reporter.format_error_context(&shrink_error);
    assert!(context.contains("Shrinkage process timeout"));
    assert!(context.contains("Iterations attempted: 1000"));
    assert!(context.contains("Last successful shrink: 42"));

    // Test ConfigError
    let config_error = PropertyError::ConfigError {
        message: "Invalid iteration count".to_string(),
        field: Some("iterations".to_string()),
    };
    let context = reporter.format_error_context(&config_error);
    assert!(context.contains("Configuration error"));
    assert!(context.contains("Field: iterations"));

    // Test TestCancelled
    let cancel_error = PropertyError::TestCancelled {
        reason: "User interrupted".to_string(),
    };
    let context = reporter.format_error_context(&cancel_error);
    assert!(context.contains("Test cancellation"));
    assert!(context.contains("User interrupted"));

    // Test InternalError
    let internal_error = PropertyError::InternalError {
        message: "Unexpected state".to_string(),
        source_message: Some("Null pointer dereference".to_string()),
    };
    let context = reporter.format_error_context(&internal_error);
    assert!(context.contains("Internal framework error"));
    assert!(context.contains("Source: Null pointer dereference"));
}

#[test]
fn test_shrink_progress_visualization() {
    let mut progress = ShrinkProgress::new();

    // Add some shrinking steps
    progress.add_step(ShrinkStep {
        step_number: 1,
        input_description: "100 -> 50".to_string(),
        step_time: Duration::from_millis(10),
        successful: true,
    });

    progress.add_step(ShrinkStep {
        step_number: 2,
        input_description: "50 -> 25".to_string(),
        step_time: Duration::from_millis(8),
        successful: true,
    });

    progress.add_step(ShrinkStep {
        step_number: 3,
        input_description: "25 -> 12 (failed)".to_string(),
        step_time: Duration::from_millis(5),
        successful: false,
    });

    progress.complete(Duration::from_millis(25));

    let visualization = progress.visualize();
    assert!(visualization.contains("Shrinking Progress:"));
    assert!(visualization.contains("✓ Step 1: 100 -> 50"));
    assert!(visualization.contains("✓ Step 2: 50 -> 25"));
    assert!(visualization.contains("✗ Step 3: 25 -> 12 (failed)"));
    assert!(visualization.contains("Completed in"));
    assert!(visualization.contains("3 steps"));

    let stats = progress.statistics();
    assert_eq!(stats.total_steps, 3);
    assert_eq!(stats.successful_steps, 2);
    assert!((stats.success_rate - 2.0 / 3.0).abs() < 0.001);
    assert_eq!(stats.total_time, Duration::from_millis(25));
}

#[test]
fn test_error_suggestions_generation() {
    let reporter = ErrorReporter::new();
    let config = TestConfig::default();

    // Test suggestions for PropertyFailed with shrinking
    let error = PropertyError::PropertyFailed {
        message: "test failed".to_string(),
        context: None,
        iteration: None,
    };
    let failure_with_shrink = TestFailure::new(
        error.clone(),
        100,
        Some(0),
        5,
        config.clone(),
        0,
        Duration::from_millis(100),
        Duration::from_millis(50),
    );
    let suggestions = reporter.generate_suggestions(&failure_with_shrink);
    assert!(suggestions.contains("Focus on the shrunk input"));
    assert!(suggestions.contains("Check if your property logic is correct"));

    // Test suggestions for PropertyFailed without shrinking
    let failure_no_shrink = TestFailure::new(
        error,
        100,
        None,
        0,
        config.clone(),
        0,
        Duration::from_millis(100),
        Duration::from_millis(0),
    );
    let suggestions = reporter.generate_suggestions(&failure_no_shrink);
    assert!(suggestions.contains("Consider implementing shrinking"));

    // Test suggestions for GenerationFailed
    let gen_error = PropertyError::GenerationFailed {
        message: "generation failed".to_string(),
        context: None,
    };
    let gen_failure = TestFailure::new(
        gen_error,
        0,
        None,
        0,
        config,
        0,
        Duration::from_millis(10),
        Duration::from_millis(0),
    );
    let suggestions = reporter.generate_suggestions(&gen_failure);
    assert!(suggestions.contains("Check your generator implementation"));
    assert!(suggestions.contains("Verify generator constraints"));
}

#[test]
fn test_property_test_builder_with_error_reporting() {
    let result = PropertyTestBuilder::new()
        .iterations(1)
        .verbose_errors()
        .show_shrink_progress()
        .show_config_in_errors()
        .run(ConstantGenerator::new(42), AlwaysFailProperty);

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
fn test_shrinkage_progress_in_property_test() {
    // Create a property that fails for values >= 50
    struct FailsForLargeValuesProperty;
    impl Property<i32> for FailsForLargeValuesProperty {
        type Output = ();
        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input >= 50 {
                Err(PropertyError::PropertyFailed {
                    message: format!("Input {} is too large", input),
                    context: Some("large value test".to_string()),
                    iteration: None,
                })
            } else {
                Ok(())
            }
        }
    }

    // Use a generator that can actually shrink
    let generator = ShrinkableIntGenerator::new(100);
    let property = FailsForLargeValuesProperty;

    let config = TestConfig {
        iterations: 1,
        max_shrink_iterations: 10,
        shrink_timeout: Duration::from_secs(1),
        ..TestConfig::default()
    };

    let result = check_with_config(generator, property, config);
    assert!(result.is_err());

    if let Err(failure) = result {
        assert_eq!(failure.original_input, 100);
        // Should have shrunk towards a smaller failing value
        assert!(failure.shrunk_input.is_some());
        assert!(failure.shrink_steps > 0);
        // The shrunk value should be >= 50 (the failing threshold)
        if let Some(shrunk) = failure.shrunk_input {
            assert!(shrunk >= 50);
        }
    }
}

#[test]
fn test_error_reporter_summary_vs_detailed() {
    let error = PropertyError::PropertyFailed {
        message: "Test failed".to_string(),
        context: Some("test context".to_string()),
        iteration: Some(5),
    };

    let config = TestConfig::default();
    let failure = TestFailure::new(
        error,
        42,
        Some(0),
        3,
        config,
        5,
        Duration::from_millis(100),
        Duration::from_millis(30),
    );

    let reporter = ErrorReporter::new();

    // Test summary format
    let summary = reporter.format_summary(&failure);
    assert!(summary.contains("QUICK SUMMARY"));
    assert!(summary.contains("Focus on input: 0"));

    // Test detailed format
    let detailed = reporter.verbose().format_failure(&failure);
    assert!(detailed.contains("PROPERTY TEST FAILURE"));
    assert!(detailed.contains("DETAILED ERROR CONTEXT"));
    assert!(detailed.contains("SUGGESTIONS"));

    // Detailed should be longer than summary
    assert!(detailed.len() > summary.len());
}

#[test]
fn test_error_reporter_for_test_output_integration() {
    let error = PropertyError::PropertyFailed {
        message: "Integration test failure".to_string(),
        context: None,
        iteration: None,
    };

    let config = TestConfig::default();
    let failure = TestFailure::new(
        error,
        123,
        Some(1),
        2,
        config,
        0,
        Duration::from_millis(75),
        Duration::from_millis(25),
    );

    // Test verbose mode for test output
    let verbose_reporter = ErrorReporter::new().verbose();
    let verbose_output = verbose_reporter.format_for_test_output(&failure);
    assert!(verbose_output.contains("PROPERTY TEST FAILURE"));
    assert!(verbose_output.contains("DETAILED ERROR CONTEXT"));

    // Test concise mode for test output
    let concise_reporter = ErrorReporter::new();
    let concise_output = concise_reporter.format_for_test_output(&failure);
    assert!(concise_output.contains("QUICK SUMMARY"));
    assert!(!concise_output.contains("DETAILED ERROR CONTEXT"));
}

#[test]
fn test_shrink_progress_with_no_steps() {
    let progress = ShrinkProgress::new();
    let stats = progress.statistics();

    assert_eq!(stats.total_steps, 0);
    assert_eq!(stats.successful_steps, 0);
    assert_eq!(stats.success_rate, 0.0);
    assert_eq!(stats.total_time, Duration::from_secs(0));
    assert_eq!(stats.average_step_time, Duration::from_secs(0));

    let visualization = progress.visualize();
    assert!(visualization.contains("Shrinking Progress:"));
    assert!(visualization.contains("In progress..."));
}

#[test]
fn test_error_message_quality_and_completeness() {
    let reporter = ErrorReporter::new()
        .verbose()
        .show_config()
        .show_shrink_progress();

    // Create a comprehensive failure scenario
    let error = PropertyError::PropertyFailed {
        message: "Property assertion failed: expected positive number".to_string(),
        context: Some("validating user input range".to_string()),
        iteration: Some(73),
    };

    let config = TestConfig {
        iterations: 200,
        seed: Some(98765),
        max_shrink_iterations: 500,
        shrink_timeout: Duration::from_secs(3),
        ..TestConfig::default()
    };

    let failure = TestFailure::new(
        error,
        -1000,
        Some(-1),
        25,
        config,
        73,
        Duration::from_millis(500),
        Duration::from_millis(150),
    );

    let report = reporter.format_failure(&failure);

    // Verify completeness of error information
    assert!(report.contains("Property assertion failed: expected positive number"));
    assert!(report.contains("validating user input range"));
    assert!(report.contains("iteration 73"));
    assert!(report.contains("Original input: -1000"));
    assert!(report.contains("Shrunk input:   -1"));
    assert!(report.contains("Shrink steps:   25"));
    assert!(report.contains("Total test time:"));
    assert!(report.contains("Shrinking time:"));
    assert!(report.contains("Iterations:       200"));
    assert!(report.contains("Seed:             Some(98765)"));
    assert!(report.contains("Max shrink iter:  500"));
    assert!(report.contains("Shrink timeout:   3s"));

    // Verify helpful suggestions are present
    assert!(report.contains("SUGGESTIONS"));
    assert!(report.contains("Check if your property logic is correct"));
    assert!(report.contains("Focus on the shrunk input"));
    assert!(report.contains("Run with a fixed seed"));

    // Verify visual formatting
    assert!(report.contains("═══════════════════════════════════════════════════════════════"));
    assert!(report.contains("❌"));
    assert!(report.contains("📝"));
    assert!(report.contains("📊"));
    assert!(report.contains("⏱️"));
    assert!(report.contains("⚙️"));
    assert!(report.contains("🔍"));
    assert!(report.contains("💡"));
}
