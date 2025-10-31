//! Test runner integration and compatibility utilities
//!
//! This module provides utilities for integrating Protest with various Rust test runners
//! and frameworks, including custom output formatting and test result reporting.

use crate::{PropertyResult, TestFailure, TestSuccess};
use std::fmt;
use std::time::Duration;

/// Test runner integration utilities
pub struct TestRunner;

impl TestRunner {
    /// Format a property test result for standard test output
    pub fn format_result<T>(result: &PropertyResult<T>) -> String
    where
        T: fmt::Debug,
    {
        match result {
            Ok(success) => Self::format_success(success),
            Err(failure) => Self::format_failure(failure),
        }
    }

    /// Format a successful test result
    pub fn format_success<T>(success: &TestSuccess<T>) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Property test PASSED after {} iterations",
            success.iterations
        ));

        if let Some(seed) = success.config.seed {
            output.push_str(&format!(" (seed: {})", seed));
        }

        // Note: TestSuccess doesn't have test_duration field
        // This would need to be tracked separately if needed

        if let Some(stats) = &success.stats {
            output.push_str(&format!("\nGenerated {} values", stats.total_generated));
            output.push_str(&format!(
                ", avg generation time: {:?}",
                stats.performance_metrics.average_generation_time
            ));
        }

        output
    }

    /// Format a failed test result
    pub fn format_failure<T>(failure: &TestFailure<T>) -> String
    where
        T: fmt::Debug,
    {
        let mut output = String::new();

        output.push_str(&format!("Property test FAILED: {}", failure.error));
        output.push_str(&format!(
            "\nOriginal failing input: {:?}",
            failure.original_input
        ));

        if let Some(shrunk) = &failure.shrunk_input {
            output.push_str(&format!("\nMinimal failing input: {:?}", shrunk));
            output.push_str(&format!(
                " (found after {} shrinking steps)",
                failure.shrink_steps
            ));
        }

        if let Some(seed) = failure.config.seed {
            output.push_str(&format!(
                "\nSeed: {} (use this to reproduce the failure)",
                seed
            ));
        }

        output.push_str(&format!("\nTest duration: {:?}", failure.test_duration));

        if failure.shrink_duration > Duration::from_millis(0) {
            output.push_str(&format!(
                ", shrinking duration: {:?}",
                failure.shrink_duration
            ));
        }

        output
    }

    /// Create a panic message for property test failures
    pub fn create_panic_message<T>(failure: &TestFailure<T>) -> String
    where
        T: fmt::Debug,
    {
        format!("Property test failed: {}", Self::format_failure(failure))
    }

    /// Check if we're running under cargo test
    pub fn is_cargo_test() -> bool {
        std::env::var("CARGO").is_ok() || std::env::var("CARGO_PKG_NAME").is_ok()
    }

    /// Check if we're running with verbose output
    pub fn is_verbose_output() -> bool {
        std::env::args().any(|arg| arg == "--nocapture" || arg == "-v" || arg == "--verbose")
    }

    /// Print test progress if verbose output is enabled
    pub fn print_progress(message: &str) {
        if Self::is_verbose_output() {
            println!("protest: {}", message);
        }
    }

    /// Print test statistics if verbose output is enabled
    pub fn print_statistics<T>(success: &TestSuccess<T>) {
        if Self::is_verbose_output()
            && let Some(stats) = &success.stats
        {
            println!("protest: Test statistics:");
            println!(
                "protest:   Total values generated: {}",
                stats.total_generated
            );
            println!(
                "protest:   Generation time: {:?}",
                stats.performance_metrics.total_generation_time
            );
            println!(
                "protest:   Average per value: {:?}",
                stats.performance_metrics.average_generation_time
            );

            if stats.performance_metrics.memory_stats.peak_memory_usage > 0 {
                println!(
                    "protest:   Peak memory usage: {} KB",
                    stats.performance_metrics.memory_stats.peak_memory_usage / 1024
                );
            }
        }
    }
}

/// Custom test result type for better integration with test frameworks
#[derive(Debug)]
pub enum TestResult {
    /// Test passed
    Passed {
        iterations: usize,
        duration: Duration,
        seed: Option<u64>,
    },
    /// Test failed
    Failed {
        error: String,
        original_input: String,
        shrunk_input: Option<String>,
        shrink_steps: usize,
        seed: Option<u64>,
        duration: Duration,
    },
    /// Test was skipped
    Skipped { reason: String },
}

impl TestResult {
    /// Create a TestResult from a PropertyResult
    pub fn from_property_result<T>(result: PropertyResult<T>) -> Self
    where
        T: fmt::Debug,
    {
        match result {
            Ok(success) => TestResult::Passed {
                iterations: success.iterations,
                duration: Duration::from_nanos(0), // TestSuccess doesn't track duration
                seed: success.config.seed,
            },
            Err(failure) => TestResult::Failed {
                error: failure.error.to_string(),
                original_input: format!("{:?}", failure.original_input),
                shrunk_input: failure.shrunk_input.as_ref().map(|s| format!("{:?}", s)),
                shrink_steps: failure.shrink_steps,
                seed: failure.config.seed,
                duration: failure.test_duration,
            },
        }
    }

    /// Check if the test passed
    pub fn is_passed(&self) -> bool {
        matches!(self, TestResult::Passed { .. })
    }

    /// Check if the test failed
    pub fn is_failed(&self) -> bool {
        matches!(self, TestResult::Failed { .. })
    }

    /// Check if the test was skipped
    pub fn is_skipped(&self) -> bool {
        matches!(self, TestResult::Skipped { .. })
    }

    /// Get the test duration if available
    pub fn duration(&self) -> Option<Duration> {
        match self {
            TestResult::Passed { duration, .. } => Some(*duration),
            TestResult::Failed { duration, .. } => Some(*duration),
            TestResult::Skipped { .. } => None,
        }
    }

    /// Get the seed used for the test if available
    pub fn seed(&self) -> Option<u64> {
        match self {
            TestResult::Passed { seed, .. } => *seed,
            TestResult::Failed { seed, .. } => *seed,
            TestResult::Skipped { .. } => None,
        }
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestResult::Passed {
                iterations,
                duration,
                seed,
            } => {
                write!(f, "PASSED ({} iterations in {:?}", iterations, duration)?;
                if let Some(seed) = seed {
                    write!(f, ", seed: {}", seed)?;
                }
                write!(f, ")")
            }
            TestResult::Failed {
                error,
                original_input,
                shrunk_input,
                shrink_steps,
                seed,
                duration,
            } => {
                write!(f, "FAILED: {}", error)?;
                write!(f, "\n  Original input: {}", original_input)?;
                if let Some(shrunk) = shrunk_input {
                    write!(
                        f,
                        "\n  Minimal input: {} (after {} steps)",
                        shrunk, shrink_steps
                    )?;
                }
                if let Some(seed) = seed {
                    write!(f, "\n  Seed: {}", seed)?;
                }
                write!(f, "\n  Duration: {:?}", duration)
            }
            TestResult::Skipped { reason } => {
                write!(f, "SKIPPED: {}", reason)
            }
        }
    }
}

/// Trait for custom test output formatting
pub trait TestOutputFormatter {
    /// Format the start of a test
    fn format_test_start(&self, test_name: &str) -> String;

    /// Format a successful test result
    fn format_test_success(&self, test_name: &str, result: &TestResult) -> String;

    /// Format a failed test result
    fn format_test_failure(&self, test_name: &str, result: &TestResult) -> String;

    /// Format a skipped test result
    fn format_test_skipped(&self, test_name: &str, result: &TestResult) -> String;
}

/// Default test output formatter compatible with cargo test
pub struct DefaultFormatter;

impl TestOutputFormatter for DefaultFormatter {
    fn format_test_start(&self, test_name: &str) -> String {
        format!("test {} ... ", test_name)
    }

    fn format_test_success(&self, _test_name: &str, result: &TestResult) -> String {
        match result {
            TestResult::Passed {
                iterations,
                duration,
                ..
            } => {
                format!("ok ({} iterations, {:?})", iterations, duration)
            }
            _ => "ok".to_string(),
        }
    }

    fn format_test_failure(&self, _test_name: &str, result: &TestResult) -> String {
        match result {
            TestResult::Failed { .. } => "FAILED".to_string(),
            _ => "FAILED".to_string(),
        }
    }

    fn format_test_skipped(&self, _test_name: &str, _result: &TestResult) -> String {
        "ignored".to_string()
    }
}

/// Verbose test output formatter with detailed information
pub struct VerboseFormatter;

impl TestOutputFormatter for VerboseFormatter {
    fn format_test_start(&self, test_name: &str) -> String {
        format!("Running property test: {}", test_name)
    }

    fn format_test_success(&self, test_name: &str, result: &TestResult) -> String {
        format!("✓ {} {}", test_name, result)
    }

    fn format_test_failure(&self, test_name: &str, result: &TestResult) -> String {
        format!("✗ {} {}", test_name, result)
    }

    fn format_test_skipped(&self, test_name: &str, result: &TestResult) -> String {
        format!("- {} {}", test_name, result)
    }
}

/// JSON test output formatter for machine-readable results
pub struct JsonFormatter;

impl TestOutputFormatter for JsonFormatter {
    fn format_test_start(&self, test_name: &str) -> String {
        format!(
            r#"{{"event":"started","name":"{}","type":"property_test"}}"#,
            test_name
        )
    }

    fn format_test_success(&self, test_name: &str, result: &TestResult) -> String {
        match result {
            TestResult::Passed {
                iterations,
                duration,
                seed,
            } => {
                let seed_json = seed
                    .map(|s| format!(r#","seed":{}"#, s))
                    .unwrap_or_default();
                format!(
                    r#"{{"event":"ok","name":"{}","type":"property_test","iterations":{},"duration_ms":{}{}}}"#,
                    test_name,
                    iterations,
                    duration.as_millis(),
                    seed_json
                )
            }
            _ => format!(
                r#"{{"event":"ok","name":"{}","type":"property_test"}}"#,
                test_name
            ),
        }
    }

    fn format_test_failure(&self, test_name: &str, result: &TestResult) -> String {
        match result {
            TestResult::Failed {
                error,
                original_input,
                shrunk_input,
                shrink_steps,
                seed,
                duration,
            } => {
                let seed_json = seed
                    .map(|s| format!(r#","seed":{}"#, s))
                    .unwrap_or_default();
                let shrunk_json = shrunk_input
                    .as_ref()
                    .map(|s| format!(r#","shrunk_input":"{}","shrink_steps":{}"#, s, shrink_steps))
                    .unwrap_or_default();

                format!(
                    r#"{{"event":"failed","name":"{}","type":"property_test","error":"{}","original_input":"{}","duration_ms":{}{}{}}}"#,
                    test_name,
                    error.replace('"', r#"\""#),
                    original_input.replace('"', r#"\""#),
                    duration.as_millis(),
                    seed_json,
                    shrunk_json
                )
            }
            _ => format!(
                r#"{{"event":"failed","name":"{}","type":"property_test"}}"#,
                test_name
            ),
        }
    }

    fn format_test_skipped(&self, test_name: &str, result: &TestResult) -> String {
        match result {
            TestResult::Skipped { reason } => {
                format!(
                    r#"{{"event":"ignored","name":"{}","type":"property_test","reason":"{}"}}"#,
                    test_name,
                    reason.replace('"', r#"\""#)
                )
            }
            _ => format!(
                r#"{{"event":"ignored","name":"{}","type":"property_test"}}"#,
                test_name
            ),
        }
    }
}

/// Test execution context for custom test runners
pub struct TestContext {
    pub test_name: String,
    pub formatter: Box<dyn TestOutputFormatter>,
    pub capture_output: bool,
    pub verbose: bool,
}

impl TestContext {
    /// Create a new test context with default settings
    pub fn new(test_name: String) -> Self {
        let verbose = TestRunner::is_verbose_output();
        let formatter: Box<dyn TestOutputFormatter> = if verbose {
            Box::new(VerboseFormatter)
        } else {
            Box::new(DefaultFormatter)
        };

        Self {
            test_name,
            formatter,
            capture_output: !verbose,
            verbose,
        }
    }

    /// Create a test context with JSON output
    pub fn with_json_output(test_name: String) -> Self {
        Self {
            test_name,
            formatter: Box::new(JsonFormatter),
            capture_output: false,
            verbose: false,
        }
    }

    /// Create a test context with custom formatter
    pub fn with_formatter(test_name: String, formatter: Box<dyn TestOutputFormatter>) -> Self {
        Self {
            test_name,
            formatter,
            capture_output: false,
            verbose: false,
        }
    }

    /// Execute a property test with this context
    pub fn execute<T, F>(&self, test_fn: F) -> TestResult
    where
        T: fmt::Debug,
        F: FnOnce() -> PropertyResult<T>,
    {
        if !self.capture_output {
            print!("{}", self.formatter.format_test_start(&self.test_name));
        }

        let result = test_fn();
        let test_result = TestResult::from_property_result(result);

        if !self.capture_output {
            match &test_result {
                TestResult::Passed { .. } => {
                    println!(
                        "{}",
                        self.formatter
                            .format_test_success(&self.test_name, &test_result)
                    );
                }
                TestResult::Failed { .. } => {
                    println!(
                        "{}",
                        self.formatter
                            .format_test_failure(&self.test_name, &test_result)
                    );
                }
                TestResult::Skipped { .. } => {
                    println!(
                        "{}",
                        self.formatter
                            .format_test_skipped(&self.test_name, &test_result)
                    );
                }
            }
        }

        test_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PropertyError, TestConfig};
    use std::time::Duration;

    #[test]
    fn test_format_success() {
        let success: TestSuccess<i32> = TestSuccess::new(
            100,
            TestConfig {
                seed: Some(42),
                ..TestConfig::default()
            },
            None,
        );

        let formatted = TestRunner::format_success(&success);
        assert!(formatted.contains("PASSED"));
        assert!(formatted.contains("100 iterations"));
        assert!(formatted.contains("seed: 42"));
        // Note: TestSuccess doesn't track duration, so we don't check for it
    }

    #[test]
    fn test_format_failure() {
        let failure = TestFailure::new(
            PropertyError::property_failed("Test error"),
            42,
            Some(1),
            5,
            TestConfig {
                seed: Some(123),
                ..TestConfig::default()
            },
            1, // failed_iteration
            Duration::from_millis(200),
            Duration::from_millis(50),
        );

        let formatted = TestRunner::format_failure(&failure);
        assert!(formatted.contains("FAILED"));
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("42"));
        assert!(formatted.contains("1"));
        assert!(formatted.contains("5 shrinking steps"));
        assert!(formatted.contains("123"));
    }

    #[test]
    fn test_test_result_conversion() {
        let success: PropertyResult<i32> = Ok(TestSuccess::new(50, TestConfig::default(), None));

        let test_result = TestResult::from_property_result(success);
        assert!(test_result.is_passed());
        // TestSuccess doesn't track duration, so it defaults to 0ns
        assert_eq!(test_result.duration(), Some(Duration::from_nanos(0)));
    }

    #[test]
    fn test_default_formatter() {
        let formatter = DefaultFormatter;
        let result = TestResult::Passed {
            iterations: 100,
            duration: Duration::from_millis(500),
            seed: Some(42),
        };

        let start = formatter.format_test_start("my_test");
        assert_eq!(start, "test my_test ... ");

        let success = formatter.format_test_success("my_test", &result);
        assert!(success.contains("ok"));
        assert!(success.contains("100 iterations"));
    }

    #[test]
    fn test_verbose_formatter() {
        let formatter = VerboseFormatter;
        let result = TestResult::Failed {
            error: "Property failed".to_string(),
            original_input: "42".to_string(),
            shrunk_input: Some("1".to_string()),
            shrink_steps: 3,
            seed: Some(123),
            duration: Duration::from_millis(200),
        };

        let failure = formatter.format_test_failure("my_test", &result);
        assert!(failure.contains("✗"));
        assert!(failure.contains("my_test"));
        assert!(failure.contains("FAILED"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter;
        let result = TestResult::Passed {
            iterations: 100,
            duration: Duration::from_millis(500),
            seed: Some(42),
        };

        let success = formatter.format_test_success("my_test", &result);
        assert!(success.contains(r#""event":"ok""#));
        assert!(success.contains(r#""name":"my_test""#));
        assert!(success.contains(r#""iterations":100"#));
        assert!(success.contains(r#""seed":42"#));
    }

    #[test]
    fn test_test_context_creation() {
        let context = TestContext::new("test_name".to_string());
        assert_eq!(context.test_name, "test_name");

        let json_context = TestContext::with_json_output("json_test".to_string());
        assert_eq!(json_context.test_name, "json_test");
        assert!(!json_context.verbose);
    }
}
