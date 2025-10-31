//! Error types and result handling for property-based testing.

use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;

use crate::config::{GenerationStats, TestConfig};

/// Comprehensive error type for property testing failures
#[derive(Debug, Clone)]
pub enum PropertyError {
    /// Property test failed with a specific message and optional context
    PropertyFailed {
        message: String,
        context: Option<String>,
        iteration: Option<usize>,
    },

    /// Generation of test data failed
    GenerationFailed {
        message: String,
        context: Option<String>,
    },

    /// Shrinkage process timed out
    ShrinkageTimeout {
        iterations: usize,
        last_successful_shrink: Option<String>,
    },

    /// Configuration error
    ConfigError {
        message: String,
        field: Option<String>,
    },

    /// Test execution was cancelled
    TestCancelled { reason: String },

    /// Internal error in the testing framework
    InternalError {
        message: String,
        source_message: Option<String>,
    },
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyError::PropertyFailed {
                message,
                context,
                iteration,
            } => {
                write!(f, "Property failed: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, " (context: {})", ctx)?;
                }
                if let Some(iter) = iteration {
                    write!(f, " (iteration: {})", iter)?;
                }
                Ok(())
            }
            PropertyError::GenerationFailed { message, context } => {
                write!(f, "Generation failed: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, " (context: {})", ctx)?;
                }
                Ok(())
            }
            PropertyError::ShrinkageTimeout {
                iterations,
                last_successful_shrink,
            } => {
                write!(f, "Shrinkage timeout after {} iterations", iterations)?;
                if let Some(shrink) = last_successful_shrink {
                    write!(f, " (last successful shrink: {})", shrink)?;
                }
                Ok(())
            }
            PropertyError::ConfigError { message, field } => {
                write!(f, "Configuration error: {}", message)?;
                if let Some(field_name) = field {
                    write!(f, " (field: {})", field_name)?;
                }
                Ok(())
            }
            PropertyError::TestCancelled { reason } => {
                write!(f, "Test cancelled: {}", reason)
            }
            PropertyError::InternalError {
                message,
                source_message,
            } => {
                write!(f, "Internal error: {}", message)?;
                if let Some(src) = source_message {
                    write!(f, " (source: {})", src)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for PropertyError {}

/// Result of a property test execution
pub type PropertyResult<T> = Result<TestSuccess<T>, TestFailure<T>>;

/// Helper functions for creating PropertyError instances with context
impl PropertyError {
    /// Create a simple property failed error
    pub fn property_failed(message: impl Into<String>) -> Self {
        Self::PropertyFailed {
            message: message.into(),
            context: None,
            iteration: None,
        }
    }

    /// Create a property failed error with context
    pub fn property_failed_with_context(
        message: impl Into<String>,
        context: Option<impl Into<String>>,
        iteration: Option<usize>,
    ) -> Self {
        Self::PropertyFailed {
            message: message.into(),
            context: context.map(|c| c.into()),
            iteration,
        }
    }

    /// Create a generation failed error with context
    pub fn generation_failed_with_context(
        message: impl Into<String>,
        context: Option<impl Into<String>>,
    ) -> Self {
        Self::GenerationFailed {
            message: message.into(),
            context: context.map(|c| c.into()),
        }
    }

    /// Create a configuration error with field information
    pub fn config_error_with_field(
        message: impl Into<String>,
        field: Option<impl Into<String>>,
    ) -> Self {
        Self::ConfigError {
            message: message.into(),
            field: field.map(|f| f.into()),
        }
    }

    /// Create a shrinkage timeout error with last successful shrink
    pub fn shrinkage_timeout_with_context(
        iterations: usize,
        last_successful_shrink: Option<impl Into<String>>,
    ) -> Self {
        Self::ShrinkageTimeout {
            iterations,
            last_successful_shrink: last_successful_shrink.map(|s| s.into()),
        }
    }

    /// Create an internal error
    pub fn internal_error(
        message: impl Into<String>,
        source_message: Option<impl Into<String>>,
    ) -> Self {
        Self::InternalError {
            message: message.into(),
            source_message: source_message.map(|s| s.into()),
        }
    }

    /// Create a test cancelled error
    pub fn test_cancelled(reason: impl Into<String>) -> Self {
        Self::TestCancelled {
            reason: reason.into(),
        }
    }

    /// Create an execution failed error (convenience method)
    pub fn execution_failed(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            source_message: None,
        }
    }
}

/// Information about a successful test run
#[derive(Debug)]
pub struct TestSuccess<T> {
    /// Number of iterations completed
    pub iterations: usize,
    /// Test configuration used
    pub config: TestConfig,
    /// Optional statistics about generated values
    pub stats: Option<GenerationStats>,
    _phantom: PhantomData<T>,
}

impl<T> TestSuccess<T> {
    /// Create a new TestSuccess instance
    pub fn new(iterations: usize, config: TestConfig, stats: Option<GenerationStats>) -> Self {
        Self {
            iterations,
            config,
            stats,
            _phantom: PhantomData,
        }
    }
}

/// Information about a failed test run
#[derive(Debug)]
pub struct TestFailure<T> {
    /// The error that caused the failure
    pub error: PropertyError,
    /// Original input that caused the failure
    pub original_input: T,
    /// Shrunk input (if shrinking was successful)
    pub shrunk_input: Option<T>,
    /// Number of shrinking steps performed
    pub shrink_steps: usize,
    /// Test configuration used
    pub config: TestConfig,
    /// Iteration number where the failure occurred
    pub failed_iteration: usize,
    /// Total time spent on the test
    pub test_duration: std::time::Duration,
    /// Time spent on shrinking
    pub shrink_duration: std::time::Duration,
}

impl<T> TestFailure<T> {
    /// Create a new TestFailure instance
    pub fn new(
        error: PropertyError,
        original_input: T,
        shrunk_input: Option<T>,
        shrink_steps: usize,
        config: TestConfig,
        failed_iteration: usize,
        test_duration: std::time::Duration,
        shrink_duration: std::time::Duration,
    ) -> Self {
        Self {
            error,
            original_input,
            shrunk_input,
            shrink_steps,
            config,
            failed_iteration,
            test_duration,
            shrink_duration,
        }
    }

    /// Get a detailed report of the test failure
    pub fn detailed_report(&self) -> String
    where
        T: fmt::Debug,
    {
        let mut report = String::new();

        report.push_str(&format!(
            "Property test failed on iteration {}\n",
            self.failed_iteration
        ));
        report.push_str(&format!("Error: {}\n", self.error));
        report.push_str(&format!("Original input: {:?}\n", self.original_input));

        if let Some(ref shrunk) = self.shrunk_input {
            report.push_str(&format!("Shrunk input: {:?}\n", shrunk));
            report.push_str(&format!("Shrinking steps: {}\n", self.shrink_steps));
            report.push_str(&format!("Shrinking time: {:?}\n", self.shrink_duration));
        } else {
            report.push_str("No shrinking performed\n");
        }

        report.push_str(&format!("Total test time: {:?}\n", self.test_duration));
        report.push_str(&format!(
            "Test configuration: iterations={}, seed={:?}\n",
            self.config.iterations, self.config.seed
        ));

        report
    }

    /// Get a concise summary of the test failure
    pub fn summary(&self) -> String
    where
        T: fmt::Debug,
    {
        if let Some(ref shrunk) = self.shrunk_input {
            format!(
                "Property failed with input {:?} (shrunk from {:?}) on iteration {}",
                shrunk, self.original_input, self.failed_iteration
            )
        } else {
            format!(
                "Property failed with input {:?} on iteration {}",
                self.original_input, self.failed_iteration
            )
        }
    }
}

impl<T: fmt::Debug> fmt::Display for TestFailure<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Comprehensive error reporter with various output modes
pub struct ErrorReporter {
    pub verbose: bool,
    pub show_shrink_progress: bool,
    pub show_timing: bool,
    pub show_config: bool,
}

impl ErrorReporter {
    /// Create a new error reporter with default settings
    pub fn new() -> Self {
        Self {
            verbose: false,
            show_shrink_progress: false,
            show_timing: true,
            show_config: false,
        }
    }

    /// Enable verbose output mode
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Enable shrinkage progress visualization
    pub fn show_shrink_progress(mut self) -> Self {
        self.show_shrink_progress = true;
        self
    }

    /// Enable timing information display
    pub fn show_timing(mut self) -> Self {
        self.show_timing = true;
        self
    }

    /// Enable configuration display
    pub fn show_config(mut self) -> Self {
        self.show_config = true;
        self
    }

    /// Generate a comprehensive error report
    pub fn format_failure<T>(&self, failure: &TestFailure<T>) -> String
    where
        T: fmt::Debug,
    {
        let mut report = String::new();

        // Header
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("                    PROPERTY TEST FAILURE                     \n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Basic failure information
        report.push_str(&format!(
            "❌ Test failed on iteration {}\n",
            failure.failed_iteration
        ));
        report.push_str(&format!("📝 Error: {}\n\n", failure.error));

        // Input information
        report.push_str("📊 INPUT INFORMATION:\n");
        report.push_str(&format!(
            "   Original input: {:?}\n",
            failure.original_input
        ));

        if let Some(ref shrunk) = failure.shrunk_input {
            report.push_str(&format!("   Shrunk input:   {:?}\n", shrunk));
            report.push_str(&format!("   Shrink steps:   {}\n", failure.shrink_steps));

            if self.show_shrink_progress {
                report.push_str(&self.format_shrink_progress(failure));
            }
        } else {
            report.push_str("   No shrinking performed\n");
        }
        report.push('\n');

        // Timing information
        if self.show_timing {
            report.push_str("⏱️  TIMING INFORMATION:\n");
            report.push_str(&format!(
                "   Total test time:  {:?}\n",
                failure.test_duration
            ));
            report.push_str(&format!(
                "   Shrinking time:   {:?}\n",
                failure.shrink_duration
            ));
            if failure.shrink_steps > 0 {
                let avg_shrink_time =
                    failure.shrink_duration.as_nanos() / failure.shrink_steps as u128;
                report.push_str(&format!(
                    "   Avg shrink time:  {:?}\n",
                    Duration::from_nanos(avg_shrink_time as u64)
                ));
            }
            report.push('\n');
        }

        // Configuration information
        if self.show_config {
            report.push_str("⚙️  CONFIGURATION:\n");
            report.push_str(&format!(
                "   Iterations:       {}\n",
                failure.config.iterations
            ));
            report.push_str(&format!("   Seed:             {:?}\n", failure.config.seed));
            report.push_str(&format!(
                "   Max shrink iter:  {}\n",
                failure.config.max_shrink_iterations
            ));
            report.push_str(&format!(
                "   Shrink timeout:   {:?}\n",
                failure.config.shrink_timeout
            ));
            report.push('\n');
        }

        // Verbose error context
        if self.verbose {
            report.push_str("🔍 DETAILED ERROR CONTEXT:\n");
            report.push_str(&self.format_error_context(&failure.error));
            report.push('\n');
        }

        // Suggestions
        report.push_str("💡 SUGGESTIONS:\n");
        report.push_str(&self.generate_suggestions(failure));

        report.push_str("═══════════════════════════════════════════════════════════════\n");

        report
    }

    /// Format shrinkage progress visualization
    fn format_shrink_progress<T>(&self, failure: &TestFailure<T>) -> String
    where
        T: fmt::Debug,
    {
        let mut progress = String::new();

        if failure.shrink_steps > 0 {
            progress.push_str("   Shrink progress:\n");

            // Create a simple progress visualization
            let max_width = 50;
            let progress_width = if failure.shrink_steps > max_width {
                max_width
            } else {
                failure.shrink_steps
            };

            progress.push_str("   ");
            for i in 0..progress_width {
                if i < progress_width - 1 {
                    progress.push('█');
                } else {
                    progress.push('▶');
                }
            }

            if failure.shrink_steps > max_width {
                progress.push_str(&format!(" ({} steps)", failure.shrink_steps));
            }
            progress.push('\n');

            // Show shrinking efficiency
            let efficiency = if failure.shrink_duration.as_millis() > 0 {
                failure.shrink_steps as f64 / failure.shrink_duration.as_millis() as f64 * 1000.0
            } else {
                0.0
            };
            progress.push_str(&format!("   Shrink rate:    {:.1} steps/sec\n", efficiency));
        }

        progress
    }

    /// Format detailed error context
    pub fn format_error_context(&self, error: &PropertyError) -> String {
        let mut context = String::new();

        match error {
            PropertyError::PropertyFailed {
                message,
                context: ctx,
                iteration,
            } => {
                context.push_str("   Type: Property assertion failure\n");
                context.push_str(&format!("   Message: {}\n", message));
                if let Some(ctx) = ctx {
                    context.push_str(&format!("   Context: {}\n", ctx));
                }
                if let Some(iter) = iteration {
                    context.push_str(&format!("   Failed at iteration: {}\n", iter));
                }
            }
            PropertyError::GenerationFailed {
                message,
                context: ctx,
            } => {
                context.push_str("   Type: Test data generation failure\n");
                context.push_str(&format!("   Message: {}\n", message));
                if let Some(ctx) = ctx {
                    context.push_str(&format!("   Context: {}\n", ctx));
                }
            }
            PropertyError::ShrinkageTimeout {
                iterations,
                last_successful_shrink,
            } => {
                context.push_str("   Type: Shrinkage process timeout\n");
                context.push_str(&format!("   Iterations attempted: {}\n", iterations));
                if let Some(shrink) = last_successful_shrink {
                    context.push_str(&format!("   Last successful shrink: {}\n", shrink));
                }
            }
            PropertyError::ConfigError { message, field } => {
                context.push_str("   Type: Configuration error\n");
                context.push_str(&format!("   Message: {}\n", message));
                if let Some(field) = field {
                    context.push_str(&format!("   Field: {}\n", field));
                }
            }
            PropertyError::TestCancelled { reason } => {
                context.push_str("   Type: Test cancellation\n");
                context.push_str(&format!("   Reason: {}\n", reason));
            }
            PropertyError::InternalError {
                message,
                source_message,
            } => {
                context.push_str("   Type: Internal framework error\n");
                context.push_str(&format!("   Message: {}\n", message));
                if let Some(source) = source_message {
                    context.push_str(&format!("   Source: {}\n", source));
                }
            }
        }

        context
    }

    /// Generate helpful suggestions based on the failure
    pub fn generate_suggestions<T>(&self, failure: &TestFailure<T>) -> String
    where
        T: fmt::Debug,
    {
        let mut suggestions = String::new();

        match &failure.error {
            PropertyError::PropertyFailed { .. } => {
                suggestions.push_str("   • Check if your property logic is correct\n");
                suggestions.push_str("   • Verify that the failing input reveals a real bug\n");
                if failure.shrunk_input.is_some() {
                    suggestions.push_str("   • Focus on the shrunk input for easier debugging\n");
                } else {
                    suggestions
                        .push_str("   • Consider implementing shrinking for your data type\n");
                }
            }
            PropertyError::GenerationFailed { .. } => {
                suggestions.push_str("   • Check your generator implementation for panics\n");
                suggestions.push_str("   • Verify generator constraints are satisfiable\n");
                suggestions.push_str("   • Consider adding bounds checking to your generator\n");
            }
            PropertyError::ShrinkageTimeout { .. } => {
                suggestions.push_str("   • Increase shrink timeout if needed\n");
                suggestions.push_str("   • Optimize your shrinking strategy\n");
                suggestions.push_str("   • Consider reducing max shrink iterations\n");
            }
            PropertyError::ConfigError { .. } => {
                suggestions.push_str("   • Review your test configuration parameters\n");
                suggestions.push_str("   • Check for valid ranges and constraints\n");
            }
            PropertyError::TestCancelled { .. } => {
                suggestions.push_str("   • Check if cancellation was intentional\n");
                suggestions.push_str("   • Review timeout settings if applicable\n");
            }
            PropertyError::InternalError { .. } => {
                suggestions.push_str("   • This may be a bug in the testing framework\n");
                suggestions.push_str("   • Consider reporting this issue\n");
                suggestions.push_str("   • Try simplifying your test case\n");
            }
        }

        // General suggestions
        suggestions.push_str("   • Run with a fixed seed for reproducible results\n");
        suggestions.push_str("   • Try increasing iterations to find edge cases\n");

        suggestions
    }

    /// Generate a concise summary for quick debugging
    pub fn format_summary<T>(&self, failure: &TestFailure<T>) -> String
    where
        T: fmt::Debug,
    {
        let mut summary = String::new();

        summary.push_str("🚨 QUICK SUMMARY:\n");
        summary.push_str(&format!("   {}\n", failure.summary()));

        if let Some(ref shrunk) = failure.shrunk_input {
            summary.push_str(&format!("   Focus on input: {:?}\n", shrunk));
        } else {
            summary.push_str(&format!(
                "   Focus on input: {:?}\n",
                failure.original_input
            ));
        }

        summary.push_str(&format!("   Error: {}\n", failure.error));

        summary
    }

    /// Format error for integration with standard test output
    pub fn format_for_test_output<T>(&self, failure: &TestFailure<T>) -> String
    where
        T: fmt::Debug,
    {
        if self.verbose {
            self.format_failure(failure)
        } else {
            self.format_summary(failure)
        }
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Shrinkage progress tracker for visualization
#[derive(Debug, Clone)]
pub struct ShrinkProgress {
    pub steps: Vec<ShrinkStep>,
    pub total_time: Duration,
    pub completed: bool,
}

/// Individual shrinking step information
#[derive(Debug, Clone)]
pub struct ShrinkStep {
    pub step_number: usize,
    pub input_description: String,
    pub step_time: Duration,
    pub successful: bool,
}

impl ShrinkProgress {
    /// Create a new shrink progress tracker
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            total_time: Duration::from_secs(0),
            completed: false,
        }
    }

    /// Add a shrinking step
    pub fn add_step(&mut self, step: ShrinkStep) {
        self.steps.push(step);
    }

    /// Mark shrinking as completed
    pub fn complete(&mut self, total_time: Duration) {
        self.total_time = total_time;
        self.completed = true;
    }

    /// Get a visualization of the shrinking progress
    pub fn visualize(&self) -> String {
        let mut viz = String::new();

        viz.push_str("Shrinking Progress:\n");
        for step in &self.steps {
            let status = if step.successful { "✓" } else { "✗" };
            viz.push_str(&format!(
                "  {} Step {}: {} ({:?})\n",
                status, step.step_number, step.input_description, step.step_time
            ));
        }

        if self.completed {
            viz.push_str(&format!(
                "Completed in {:?} ({} steps)\n",
                self.total_time,
                self.steps.len()
            ));
        } else {
            viz.push_str("In progress...\n");
        }

        viz
    }

    /// Get shrinking statistics
    pub fn statistics(&self) -> ShrinkStatistics {
        let successful_steps = self.steps.iter().filter(|s| s.successful).count();
        let total_steps = self.steps.len();
        let avg_step_time = if total_steps > 0 {
            self.total_time / total_steps as u32
        } else {
            Duration::from_secs(0)
        };

        ShrinkStatistics {
            total_steps,
            successful_steps,
            total_time: self.total_time,
            average_step_time: avg_step_time,
            success_rate: if total_steps > 0 {
                successful_steps as f64 / total_steps as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for ShrinkProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about shrinking performance
#[derive(Debug, Clone)]
pub struct ShrinkStatistics {
    pub total_steps: usize,
    pub successful_steps: usize,
    pub total_time: Duration,
    pub average_step_time: Duration,
    pub success_rate: f64,
}

impl fmt::Display for ShrinkStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Shrink Stats: {}/{} steps successful ({:.1}%), avg time: {:?}",
            self.successful_steps,
            self.total_steps,
            self.success_rate * 100.0,
            self.average_step_time
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::time::Duration;

    #[test]
    fn test_property_error_display_with_context() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
            context: Some("during validation".to_string()),
            iteration: Some(42),
        };
        let display = format!("{}", error);
        assert!(display.contains("Property failed: test failed"));
        assert!(display.contains("context: during validation"));
        assert!(display.contains("iteration: 42"));
    }

    #[test]
    fn test_property_error_display_without_context() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
            context: None,
            iteration: None,
        };
        let display = format!("{}", error);
        assert_eq!(display, "Property failed: test failed");
    }

    #[test]
    fn test_generation_failed_error_display() {
        let error = PropertyError::GenerationFailed {
            message: "generation error".to_string(),
            context: Some("while generating integers".to_string()),
        };
        let display = format!("{}", error);
        assert!(display.contains("Generation failed: generation error"));
        assert!(display.contains("context: while generating integers"));
    }

    #[test]
    fn test_shrinkage_timeout_error_display() {
        let error = PropertyError::ShrinkageTimeout {
            iterations: 1000,
            last_successful_shrink: Some("42".to_string()),
        };
        let display = format!("{}", error);
        assert!(display.contains("Shrinkage timeout after 1000 iterations"));
        assert!(display.contains("last successful shrink: 42"));
    }

    #[test]
    fn test_config_error_display() {
        let error = PropertyError::ConfigError {
            message: "invalid value".to_string(),
            field: Some("iterations".to_string()),
        };
        let display = format!("{}", error);
        assert!(display.contains("Configuration error: invalid value"));
        assert!(display.contains("field: iterations"));
    }

    #[test]
    fn test_test_cancelled_error_display() {
        let error = PropertyError::TestCancelled {
            reason: "user requested cancellation".to_string(),
        };
        let display = format!("{}", error);
        assert_eq!(display, "Test cancelled: user requested cancellation");
    }

    #[test]
    fn test_internal_error_display() {
        let error = PropertyError::InternalError {
            message: "unexpected state".to_string(),
            source_message: Some("internal failure".to_string()),
        };
        let display = format!("{}", error);
        assert!(display.contains("Internal error: unexpected state"));
        assert!(display.contains("source: internal failure"));
    }

    #[test]
    fn test_property_error_helper_functions() {
        let error = PropertyError::property_failed_with_context(
            "test failed",
            Some("during execution"),
            Some(10),
        );
        match error {
            PropertyError::PropertyFailed {
                message,
                context,
                iteration,
            } => {
                assert_eq!(message, "test failed");
                assert_eq!(context, Some("during execution".to_string()));
                assert_eq!(iteration, Some(10));
            }
            _ => panic!("Expected PropertyFailed variant"),
        }

        let error = PropertyError::generation_failed_with_context("gen error", Some("context"));
        match error {
            PropertyError::GenerationFailed { message, context } => {
                assert_eq!(message, "gen error");
                assert_eq!(context, Some("context".to_string()));
            }
            _ => panic!("Expected GenerationFailed variant"),
        }

        let error = PropertyError::config_error_with_field("bad config", Some("field_name"));
        match error {
            PropertyError::ConfigError { message, field } => {
                assert_eq!(message, "bad config");
                assert_eq!(field, Some("field_name".to_string()));
            }
            _ => panic!("Expected ConfigError variant"),
        }
    }

    #[test]
    fn test_test_failure_detailed_report() {
        let error = PropertyError::PropertyFailed {
            message: "assertion failed".to_string(),
            context: None,
            iteration: None,
        };
        let config = TestConfig::default();
        let failure = TestFailure::new(
            error,
            42,
            Some(0),
            5,
            config,
            10,
            Duration::from_millis(100),
            Duration::from_millis(50),
        );

        let report = failure.detailed_report();
        assert!(report.contains("Property test failed on iteration 10"));
        assert!(report.contains("Error: Property failed: assertion failed"));
        assert!(report.contains("Original input: 42"));
        assert!(report.contains("Shrunk input: 0"));
        assert!(report.contains("Shrinking steps: 5"));
        assert!(report.contains("Total test time:"));
    }

    #[test]
    fn test_test_failure_summary() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
            context: None,
            iteration: None,
        };
        let config = TestConfig::default();

        // Test with shrinking
        let failure = TestFailure::new(
            error.clone(),
            100,
            Some(0),
            3,
            config.clone(),
            5,
            Duration::from_millis(100),
            Duration::from_millis(30),
        );
        let summary = failure.summary();
        assert!(summary.contains("Property failed with input 0"));
        assert!(summary.contains("shrunk from 100"));
        assert!(summary.contains("iteration 5"));

        // Test without shrinking
        let failure = TestFailure::new(
            error,
            100,
            None,
            0,
            config,
            5,
            Duration::from_millis(100),
            Duration::from_millis(0),
        );
        let summary = failure.summary();
        assert!(summary.contains("Property failed with input 100"));
        assert!(summary.contains("iteration 5"));
        assert!(!summary.contains("shrunk"));
    }

    #[test]
    fn test_test_success_constructor() {
        let config = TestConfig::default();
        let stats = GenerationStats {
            total_generated: 100,
            ..Default::default()
        };
        let success: TestSuccess<i32> = TestSuccess::new(100, config.clone(), Some(stats));

        assert_eq!(success.iterations, 100);
        assert_eq!(success.config.iterations, config.iterations);
        assert!(success.stats.is_some());
        assert_eq!(success.stats.unwrap().total_generated, 100);
    }

    #[test]
    fn test_property_error_source() {
        let error = PropertyError::GenerationFailed {
            message: "original error".to_string(),
            context: None,
        };

        // Since we simplified the error structure, source() will return None
        assert!(error.source().is_none());

        let error = PropertyError::PropertyFailed {
            message: "test".to_string(),
            context: None,
            iteration: None,
        };
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_reporter_basic() {
        let reporter = ErrorReporter::new();
        assert!(!reporter.verbose);
        assert!(!reporter.show_shrink_progress);
        assert!(reporter.show_timing);
        assert!(!reporter.show_config);
    }

    #[test]
    fn test_error_reporter_builder() {
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
    fn test_error_reporter_format_failure() {
        let error = PropertyError::PropertyFailed {
            message: "test assertion failed".to_string(),
            context: Some("during validation".to_string()),
            iteration: Some(42),
        };
        let config = TestConfig::default();
        let failure = TestFailure::new(
            error,
            100,
            Some(0),
            5,
            config,
            42,
            Duration::from_millis(150),
            Duration::from_millis(75),
        );

        let reporter = ErrorReporter::new().verbose().show_config();
        let report = reporter.format_failure(&failure);

        assert!(report.contains("PROPERTY TEST FAILURE"));
        assert!(report.contains("Test failed on iteration 42"));
        assert!(report.contains("Original input: 100"));
        assert!(report.contains("Shrunk input:   0"));
        assert!(report.contains("Shrink steps:   5"));
        assert!(report.contains("TIMING INFORMATION"));
        assert!(report.contains("CONFIGURATION"));
        assert!(report.contains("DETAILED ERROR CONTEXT"));
        assert!(report.contains("SUGGESTIONS"));
    }

    #[test]
    fn test_error_reporter_format_summary() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
            context: None,
            iteration: None,
        };
        let config = TestConfig::default();
        let failure = TestFailure::new(
            error,
            42,
            Some(0),
            3,
            config,
            10,
            Duration::from_millis(100),
            Duration::from_millis(50),
        );

        let reporter = ErrorReporter::new();
        let summary = reporter.format_summary(&failure);

        assert!(summary.contains("QUICK SUMMARY"));
        assert!(summary.contains("Focus on input: 0"));
        assert!(summary.contains("Property failed"));
    }

    #[test]
    fn test_error_reporter_suggestions() {
        let reporter = ErrorReporter::new();

        // Test PropertyFailed suggestions
        let error = PropertyError::PropertyFailed {
            message: "test".to_string(),
            context: None,
            iteration: None,
        };
        let config = TestConfig::default();
        let failure = TestFailure::new(
            error,
            42,
            Some(0),
            1,
            config.clone(),
            0,
            Duration::from_millis(10),
            Duration::from_millis(5),
        );
        let suggestions = reporter.generate_suggestions(&failure);
        assert!(suggestions.contains("Check if your property logic is correct"));
        assert!(suggestions.contains("Focus on the shrunk input"));

        // Test GenerationFailed suggestions
        let error = PropertyError::GenerationFailed {
            message: "gen failed".to_string(),
            context: None,
        };
        let failure = TestFailure::new(
            error,
            42,
            None,
            0,
            config,
            0,
            Duration::from_millis(10),
            Duration::from_millis(0),
        );
        let suggestions = reporter.generate_suggestions(&failure);
        assert!(suggestions.contains("Check your generator implementation"));
    }

    #[test]
    fn test_shrink_progress_tracking() {
        let mut progress = ShrinkProgress::new();
        assert!(progress.steps.is_empty());
        assert!(!progress.completed);

        let step1 = ShrinkStep {
            step_number: 1,
            input_description: "100 -> 50".to_string(),
            step_time: Duration::from_millis(10),
            successful: true,
        };
        progress.add_step(step1);

        let step2 = ShrinkStep {
            step_number: 2,
            input_description: "50 -> 25".to_string(),
            step_time: Duration::from_millis(8),
            successful: true,
        };
        progress.add_step(step2);

        progress.complete(Duration::from_millis(20));

        assert_eq!(progress.steps.len(), 2);
        assert!(progress.completed);
        assert_eq!(progress.total_time, Duration::from_millis(20));

        let viz = progress.visualize();
        assert!(viz.contains("Shrinking Progress"));
        assert!(viz.contains("Step 1: 100 -> 50"));
        assert!(viz.contains("Step 2: 50 -> 25"));
        assert!(viz.contains("Completed in"));

        let stats = progress.statistics();
        assert_eq!(stats.total_steps, 2);
        assert_eq!(stats.successful_steps, 2);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[test]
    fn test_shrink_statistics_display() {
        let stats = ShrinkStatistics {
            total_steps: 10,
            successful_steps: 7,
            total_time: Duration::from_millis(100),
            average_step_time: Duration::from_millis(10),
            success_rate: 0.7,
        };

        let display = format!("{}", stats);
        assert!(display.contains("7/10 steps successful"));
        assert!(display.contains("70.0%"));
        assert!(display.contains("10ms"));
    }

    #[test]
    fn test_error_reporter_different_error_types() {
        let reporter = ErrorReporter::new().verbose();

        // Test ShrinkageTimeout error
        let error = PropertyError::ShrinkageTimeout {
            iterations: 1000,
            last_successful_shrink: Some("42".to_string()),
        };
        let context = reporter.format_error_context(&error);
        assert!(context.contains("Shrinkage process timeout"));
        assert!(context.contains("Iterations attempted: 1000"));
        assert!(context.contains("Last successful shrink: 42"));

        // Test ConfigError
        let error = PropertyError::ConfigError {
            message: "invalid iterations".to_string(),
            field: Some("iterations".to_string()),
        };
        let context = reporter.format_error_context(&error);
        assert!(context.contains("Configuration error"));
        assert!(context.contains("Field: iterations"));

        // Test TestCancelled
        let error = PropertyError::TestCancelled {
            reason: "user interrupt".to_string(),
        };
        let context = reporter.format_error_context(&error);
        assert!(context.contains("Test cancellation"));
        assert!(context.contains("Reason: user interrupt"));

        // Test InternalError
        let error = PropertyError::InternalError {
            message: "unexpected state".to_string(),
            source_message: Some("null pointer".to_string()),
        };
        let context = reporter.format_error_context(&error);
        assert!(context.contains("Internal framework error"));
        assert!(context.contains("Source: null pointer"));
    }

    #[test]
    fn test_error_reporter_shrink_progress_visualization() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
            context: None,
            iteration: None,
        };
        let config = TestConfig::default();
        let failure = TestFailure::new(
            error,
            100,
            Some(1),
            25,
            config,
            5,
            Duration::from_millis(200),
            Duration::from_millis(100),
        );

        let reporter = ErrorReporter::new().show_shrink_progress();
        let report = reporter.format_failure(&failure);

        assert!(report.contains("Shrink progress:"));
        assert!(report.contains("Shrink rate:"));
        // Should show progress bar for 25 steps
        assert!(report.contains("█"));
    }

    #[test]
    fn test_error_reporter_no_shrinking() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
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

        let reporter = ErrorReporter::new();
        let report = reporter.format_failure(&failure);

        assert!(report.contains("No shrinking performed"));
        assert!(report.contains("Consider implementing shrinking"));
    }

    #[test]
    fn test_error_reporter_format_for_test_output() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
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

        let reporter_verbose = ErrorReporter::new().verbose();
        let verbose_output = reporter_verbose.format_for_test_output(&failure);
        assert!(verbose_output.contains("PROPERTY TEST FAILURE"));

        let reporter_concise = ErrorReporter::new();
        let concise_output = reporter_concise.format_for_test_output(&failure);
        assert!(concise_output.contains("QUICK SUMMARY"));
        assert!(concise_output.len() < verbose_output.len());
    }
}
