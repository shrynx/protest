//! Property-Based Snapshot Testing with Insta
//!
//! This crate provides integration between [Protest](https://crates.io/crates/protest)
//! property-based testing and [Insta](https://crates.io/crates/insta) snapshot testing.
//!
//! # Overview
//!
//! Property-based snapshot testing allows you to:
//! - Test with **diverse, generated inputs** while keeping visual regression testing
//! - **Detect unexpected changes** in serialization output
//! - **Document behavior** through automatically captured snapshots
//! - **Review changes** using Insta's review workflow
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use protest::{Generator, primitives::IntGenerator, config::GeneratorConfig};
//! use protest_insta::PropertySnapshots;
//! use serde::Serialize;
//! use rand::SeedableRng;
//! use rand::rngs::StdRng;
//!
//! #[derive(Serialize)]
//! struct Point { x: i32, y: i32 }
//!
//! #[test]
//! fn test_point_serialization() {
//!     let mut rng = StdRng::seed_from_u64(42);
//!     let config = GeneratorConfig::default();
//!     let generator = IntGenerator::new(0, 100);
//!
//!     let mut snapshots = PropertySnapshots::new("point_serialization");
//!
//!     for _ in 0..5 {
//!         let x = generator.generate(&mut rng, &config);
//!         let y = generator.generate(&mut rng, &config);
//!         let point = Point { x, y };
//!         snapshots.assert_json_snapshot(&point);
//!     }
//! }
//! ```
//!
//! # Features
//!
//! - **Generator Integration**: Use any Protest generator for snapshot inputs
//! - **Automatic Naming**: Sequential snapshot names for property-based tests
//! - **JSON Support**: Built-in JSON serialization for snapshots
//! - **Debug Support**: Snapshot any Debug type
//! - **Insta Workflow**: Full compatibility with Insta's review tools
//!
//! # Examples
//!
//! ## JSON Snapshot Testing
//!
//! ```rust,no_run
//! use protest::{Generator, primitives::VecGenerator, primitives::IntGenerator};
//! use protest::{config::GeneratorConfig};
//! use protest_insta::PropertySnapshots;
//! use serde::Serialize;
//! use rand::SeedableRng;
//! use rand::rngs::StdRng;
//!
//! #[derive(Serialize)]
//! struct Data {
//!     values: Vec<i32>,
//!     sum: i32,
//! }
//!
//! #[test]
//! fn test_data_processing() {
//!     let mut rng = StdRng::seed_from_u64(123);
//!     let config = GeneratorConfig::default();
//!     let generator = VecGenerator::new(IntGenerator::new(0, 10), 1, 5);
//!
//!     let mut snapshots = PropertySnapshots::new("data_processing");
//!
//!     for _ in 0..3 {
//!         let values = generator.generate(&mut rng, &config);
//!         let sum: i32 = values.iter().sum();
//!         let data = Data { values, sum };
//!         snapshots.assert_json_snapshot(&data);
//!     }
//! }
//! ```
//!
//! ## Debug Snapshot Testing
//!
//! ```rust,no_run
//! use protest::{Generator, primitives::IntGenerator, config::GeneratorConfig};
//! use protest_insta::PropertySnapshots;
//! use rand::SeedableRng;
//! use rand::rngs::StdRng;
//!
//! #[test]
//! fn test_computation_output() {
//!     let mut rng = StdRng::seed_from_u64(456);
//!     let config = GeneratorConfig::default();
//!     let generator = IntGenerator::new(1, 100);
//!
//!     let mut snapshots = PropertySnapshots::new("computation");
//!
//!     for _ in 0..5 {
//!         let n = generator.generate(&mut rng, &config);
//!         let result = (1..=n).sum::<i32>();
//!         snapshots.assert_debug_snapshot(&result);
//!     }
//! }
//! ```

use serde::Serialize;
use std::fmt::Debug;

/// Helper for managing property-based snapshots
///
/// This struct provides convenient methods for creating multiple snapshots
/// in property-based tests. Each snapshot gets an automatic sequential name.
///
/// # Example
///
/// ```rust,no_run
/// use protest::{Generator, primitives::IntGenerator, config::GeneratorConfig};
/// use protest_insta::PropertySnapshots;
/// use rand::SeedableRng;
/// use rand::rngs::StdRng;
///
/// #[test]
/// fn test_with_property_snapshots() {
///     let mut rng = StdRng::seed_from_u64(42);
///     let config = GeneratorConfig::default();
///     let generator = IntGenerator::new(0, 100);
///
///     let mut snapshots = PropertySnapshots::new("test_case");
///
///     for _ in 0..5 {
///         let value = generator.generate(&mut rng, &config);
///         snapshots.assert_debug_snapshot(&value);
///     }
/// }
/// ```
pub struct PropertySnapshots {
    /// Base name for all snapshots in this group
    base_name: String,
    /// Counter for sequential snapshot naming
    counter: usize,
}

impl PropertySnapshots {
    /// Create a new PropertySnapshots helper with the given base name
    ///
    /// # Arguments
    ///
    /// * `base_name` - The base name for all snapshots in this group
    ///
    /// # Example
    ///
    /// ```rust
    /// use protest_insta::PropertySnapshots;
    ///
    /// let mut snapshots = PropertySnapshots::new("my_test");
    /// // Will create snapshots named: my_test_0, my_test_1, my_test_2, ...
    /// ```
    pub fn new(base_name: impl Into<String>) -> Self {
        Self {
            base_name: base_name.into(),
            counter: 0,
        }
    }

    /// Get the next snapshot name and increment the counter
    fn next_name(&mut self) -> String {
        let name = format!("{}_{}", self.base_name, self.counter);
        self.counter += 1;
        name
    }

    /// Assert a JSON snapshot for the given value
    ///
    /// This serializes the value to pretty-printed JSON and creates a snapshot.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot (must implement Serialize)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use protest_insta::PropertySnapshots;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { port: u16, host: String }
    ///
    /// # fn test() {
    /// let mut snapshots = PropertySnapshots::new("config");
    /// let cfg = Config { port: 8080, host: "localhost".to_string() };
    /// snapshots.assert_json_snapshot(&cfg);
    /// # }
    /// ```
    pub fn assert_json_snapshot<T: Serialize>(&mut self, value: &T) {
        let name = self.next_name();
        insta::assert_json_snapshot!(name, value);
    }

    /// Assert a debug snapshot for the given value
    ///
    /// This uses the Debug trait to format the value and creates a snapshot.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot (must implement Debug)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use protest_insta::PropertySnapshots;
    ///
    /// # fn test() {
    /// let mut snapshots = PropertySnapshots::new("numbers");
    /// let values = vec![1, 2, 3, 4, 5];
    /// snapshots.assert_debug_snapshot(&values);
    /// # }
    /// ```
    pub fn assert_debug_snapshot<T: Debug>(&mut self, value: &T) {
        let name = self.next_name();
        insta::assert_debug_snapshot!(name, value);
    }

    /// Assert a YAML snapshot for the given value
    ///
    /// This serializes the value to YAML and creates a snapshot.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to snapshot (must implement Serialize)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use protest_insta::PropertySnapshots;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Settings { debug: bool, timeout: u64 }
    ///
    /// # fn test() {
    /// let mut snapshots = PropertySnapshots::new("settings");
    /// let s = Settings { debug: true, timeout: 30 };
    /// snapshots.assert_yaml_snapshot(&s);
    /// # }
    /// ```
    pub fn assert_yaml_snapshot<T: Serialize>(&mut self, value: &T) {
        let name = self.next_name();
        insta::assert_yaml_snapshot!(name, value);
    }

    /// Reset the counter back to 0
    ///
    /// Useful if you want to reuse the same PropertySnapshots instance
    /// with a different set of tests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use protest_insta::PropertySnapshots;
    ///
    /// let mut snapshots = PropertySnapshots::new("test");
    /// // ... use snapshots ...
    /// snapshots.reset();
    /// // Counter is back to 0
    /// ```
    pub fn reset(&mut self) {
        self.counter = 0;
    }

    /// Get the current counter value
    ///
    /// # Example
    ///
    /// ```rust
    /// use protest_insta::PropertySnapshots;
    ///
    /// let mut snapshots = PropertySnapshots::new("test");
    /// assert_eq!(snapshots.count(), 0);
    /// ```
    pub fn count(&self) -> usize {
        self.counter
    }
}

/// Helper function to run a property-based test with snapshot assertions
///
/// This function generates multiple inputs using the provided generator
/// and calls the test function for each input. The test function should
/// use Insta's snapshot assertions.
///
/// # Arguments
///
/// * `test_name` - Base name for the snapshots
/// * `generator` - Generator for creating test inputs
/// * `sample_count` - Number of samples to generate
/// * `seed` - RNG seed for reproducibility
/// * `test_fn` - Function to test each generated value
///
/// # Example
///
/// ```rust,no_run
/// use protest::{Generator, primitives::IntGenerator, config::GeneratorConfig};
/// use protest_insta::property_snapshot_test;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Square { value: i32, squared: i32 }
///
/// #[test]
/// fn test_squaring() {
///     property_snapshot_test(
///         "square_function",
///         IntGenerator::new(1, 10),
///         5,
///         42,
///         |value, snapshots| {
///             let squared = value * value;
///             let result = Square { value, squared };
///             snapshots.assert_json_snapshot(&result);
///         }
///     );
/// }
/// ```
pub fn property_snapshot_test<T, G, F>(
    test_name: &str,
    generator: G,
    sample_count: usize,
    seed: u64,
    mut test_fn: F,
) where
    G: protest::Generator<T>,
    F: FnMut(T, &mut PropertySnapshots),
{
    use protest::config::GeneratorConfig;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(seed);
    let config = GeneratorConfig::default();
    let mut snapshots = PropertySnapshots::new(test_name);

    for _ in 0..sample_count {
        let value = generator.generate(&mut rng, &config);
        test_fn(value, &mut snapshots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_naming() {
        let mut snapshots = PropertySnapshots::new("test");
        assert_eq!(snapshots.next_name(), "test_0");
        assert_eq!(snapshots.next_name(), "test_1");
        assert_eq!(snapshots.next_name(), "test_2");
    }

    #[test]
    fn test_snapshot_reset() {
        let mut snapshots = PropertySnapshots::new("test");
        snapshots.next_name();
        snapshots.next_name();
        assert_eq!(snapshots.count(), 2);
        snapshots.reset();
        assert_eq!(snapshots.count(), 0);
        assert_eq!(snapshots.next_name(), "test_0");
    }

    #[test]
    fn test_snapshot_count() {
        let mut snapshots = PropertySnapshots::new("test");
        assert_eq!(snapshots.count(), 0);
        snapshots.next_name();
        assert_eq!(snapshots.count(), 1);
        snapshots.next_name();
        assert_eq!(snapshots.count(), 2);
    }
}
