#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_pattern_matching)]

//! # Protest - Property-Based Testing for Rust
//!
//! Protest is a property-based testing library that provides an intuitive API for generating
//! random test data, executing property tests with configurable parameters, and shrinking
//! failing cases to minimal examples.
//!
//! ## Quick Start
//!
//! ```rust
//! use protest::{just, range, Strategy};
//! use rand::thread_rng;
//!
//! // Create a simple strategy
//! let strategy = range(1, 100);
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate a value
//! let value = strategy.generate(&mut rng, &config);
//! assert!(value >= 1 && value <= 100);
//! ```

// Public modules
pub mod arbitrary;
pub mod config;
#[cfg(feature = "persistence")]
pub mod coverage;
pub mod ergonomic;
pub mod error;
pub mod execution;
pub mod generator;
pub mod performance;
#[cfg(feature = "persistence")]
pub mod persistence;
pub mod primitives;
pub mod property;
#[cfg(feature = "persistence")]
pub mod regression;
pub mod rng;
pub mod shrink;
pub mod statistics;
pub mod strategy;
pub mod test_runner;

// Re-export the main public API
pub use arbitrary::Arbitrary;
pub use config::{
    ConfigError, ConfigManager, GeneratorConfig, GlobalConfig, TestConfig, create_test_config,
    create_test_config_with_overrides, get_global_config, set_global_config,
};
pub use error::PropertyError;
pub use execution::{
    AsyncPropertyTest, PropertyTest, PropertyTestBuilder, check, check_async,
    check_async_with_config, check_with_config,
};
pub use generator::{
    BoxedGenerator, ConstantGenerator, Generator, GeneratorRegistry, OneOfGenerator,
};
pub use performance::{
    LazyGenerator, ParallelConfig, ParallelPropertyTest, StreamingShrinkStrategy, check_parallel,
    lazy,
};
// Note: ParallelAsyncPropertyTest and check_async_parallel have been removed
// to keep the library runtime-agnostic. Use check_async with your own async runtime instead.
#[cfg(feature = "persistence")]
pub use coverage::{
    CoverageCorpus, CoverageCorpusConfig, CoverageStats, CoverageTracker, path_hash,
};
#[cfg(feature = "persistence")]
pub use persistence::{CorpusCase, FailureCase, FailureSnapshot, PersistenceConfig, TestCorpus};
pub use primitives::*;
pub use property::{AsyncProperty, Property};
#[cfg(feature = "persistence")]
pub use regression::{RegressionConfig, RegressionGenerator};
pub use rng::{DefaultRngProvider, RngManager, RngProvider, create_rng, create_seeded_rng};
pub use shrink::{AsyncShrinkEngine, ShrinkConfig, ShrinkEngine, ShrinkResult, Shrinkable};
pub use statistics::{CoverageThresholdsBuilder, StatisticsCollector};
pub use strategy::Strategy;
pub use test_runner::{
    DefaultFormatter, JsonFormatter, TestContext, TestOutputFormatter, TestResult, TestRunner,
    VerboseFormatter,
};

// Re-export common types
pub use error::{PropertyResult, TestFailure, TestSuccess};

// Re-export strategy builders for convenience
pub use strategy::{just, one_of, range};

// Re-export derive macro from separate crate when derive feature is enabled
#[cfg(feature = "derive")]
pub use protest_derive::Generator;

// Re-export property test macro when derive feature is enabled
#[cfg(feature = "derive")]
pub use protest_derive::property_test;

// Re-export test builder macro when derive feature is enabled
#[cfg(feature = "derive")]
pub use protest_derive::test_builder;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_config_defaults() {
        let test_config = TestConfig::default();
        assert_eq!(test_config.iterations, 100);
        assert_eq!(test_config.max_shrink_iterations, 1000);
        assert_eq!(test_config.shrink_timeout, Duration::from_secs(10));
        assert!(test_config.seed.is_none());
    }

    #[test]
    fn test_generator_config_defaults() {
        let gen_config = GeneratorConfig::default();
        assert_eq!(gen_config.size_hint, 10);
        assert_eq!(gen_config.max_depth, 5);
        assert!(gen_config.custom_ranges.is_empty());
    }

    #[test]
    fn test_global_config_defaults() {
        let global_config = GlobalConfig::default();
        assert_eq!(global_config.default_iterations, 100);
        assert!(global_config.default_seed.is_none());
    }

    #[test]
    fn test_property_error_display() {
        let error = PropertyError::PropertyFailed {
            message: "test failed".to_string(),
            context: None,
            iteration: None,
        };
        assert_eq!(format!("{}", error), "Property failed: test failed");

        let error = PropertyError::ShrinkageTimeout {
            iterations: 500,
            last_successful_shrink: None,
        };
        assert_eq!(
            format!("{}", error),
            "Shrinkage timeout after 500 iterations"
        );
    }

    #[test]
    fn test_public_api_integration() {
        use rand::thread_rng;

        // Test that the public API works as expected
        let strategy = just(42).zip(range(1, 10));
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (left, right) = strategy.generate(&mut rng, &config);
        assert_eq!(left, 42);
        assert!((1..=10).contains(&right));
    }

    #[test]
    fn test_strategy_composition_public_api() {
        use rand::thread_rng;

        // Test complex strategy composition through public API
        let strategy = range(1, 5)
            .map(|x| x * 2)
            .filter(|&x| x > 4)
            .zip(just("test"));

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (number, text) = strategy.generate(&mut rng, &config);
        assert!(number > 4);
        assert!(number <= 10);
        assert!(number % 2 == 0);
        assert_eq!(text, "test");
    }
}
