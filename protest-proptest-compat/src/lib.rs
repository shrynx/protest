//! Proptest Migration Helpers for Protest
//!
//! This crate provides utilities, adapters, and guidance for migrating from
//! [proptest](https://crates.io/crates/proptest) to [Protest](https://crates.io/crates/protest).
//!
//! # Overview
//!
//! Rather than attempting to be a drop-in replacement, this crate focuses on making
//! migration smooth by providing:
//!
//! - **Strategy adapters** - Convert between proptest and Protest concepts
//! - **Helper functions** - Common migration patterns
//! - **Side-by-side examples** - See proptest vs Protest equivalents
//! - **Migration guide** - Step-by-step instructions
//!
//! # Quick Migration Guide
//!
//! ## Pattern 1: Simple Range-Based Properties
//!
//! ### Before (Proptest)
//! ```rust,ignore
//! use proptest::prelude::*;
//!
//! proptest! {
//!     #[test]
//!     fn test_addition(a in 0..100i32, b in 0..100i32) {
//!         assert!(a + b >= a);
//!         assert!(a + b >= b);
//!     }
//! }
//! ```
//!
//! ### After (Protest)
//! ```rust,no_run
//! use protest::*;
//!
//! #[test]
//! fn test_addition() {
//!     property!(generator!(i32, 0, 100), |(a, b)| {
//!         a + b >= a && a + b >= b
//!     });
//! }
//! ```
//!
//! ## Pattern 2: Vector Properties
//!
//! ### Before (Proptest)
//! ```rust,ignore
//! proptest! {
//!     #[test]
//!     fn reverse_twice_is_identity(v: Vec<i32>) {
//!         let mut v2 = v.clone();
//!         v2.reverse();
//!         v2.reverse();
//!         assert_eq!(v, v2);
//!     }
//! }
//! ```
//!
//! ### After (Protest)
//! ```rust,no_run
//! use protest::ergonomic::*;
//!
//! #[test]
//! fn reverse_twice_is_identity() {
//!     property(|mut v: Vec<i32>| {
//!         let original = v.clone();
//!         v.reverse();
//!         v.reverse();
//!         v == original
//!     })
//!     .iterations(100)
//!     .run()
//!     .expect("property should hold");
//! }
//! ```
//!
//! ## Pattern 3: Custom Strategies
//!
//! ### Before (Proptest)
//! ```rust,ignore
//! use proptest::strategy::Strategy;
//!
//! fn my_strategy() -> impl Strategy<Value = MyType> {
//!     // ...
//! }
//! ```
//!
//! ### After (Protest)
//! ```rust,ignore
//! use protest::Generator;
//!
//! struct MyGenerator;
//! impl Generator<MyType> for MyGenerator {
//!     fn generate(&self, rng: &mut dyn RngCore, config: &GeneratorConfig) -> MyType {
//!         // ...
//!     }
//! }
//! ```

use protest::{Generator, config::GeneratorConfig};
use rand::RngCore;

/// A helper function to convert a proptest-style range into a Protest generator
///
/// This makes migration easier for simple integer range strategies.
///
/// # Example
///
/// ```rust
/// use protest_proptest_compat::range_to_generator;
///
/// // Proptest style: 0..100i32
/// // Protest style:
/// let generator = range_to_generator(0, 100);
/// ```
pub fn range_to_generator<T>(start: T, end: T) -> protest::primitives::IntGenerator<T>
where
    T: num_traits::PrimInt,
{
    protest::primitives::IntGenerator::new(start, end)
}

/// Adapter to wrap a Protest generator with proptest-like interface
///
/// This can help when you have code that expects proptest's Strategy trait.
pub struct GeneratorAdapter<G, T> {
    generator: G,
    _phantom: std::marker::PhantomData<T>,
}

impl<G, T> GeneratorAdapter<G, T> {
    /// Create a new adapter wrapping a Protest generator
    pub fn new(generator: G) -> Self {
        Self {
            generator,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Generate a value using the wrapped generator
    pub fn generate(&self, rng: &mut dyn RngCore) -> T
    where
        G: Generator<T>,
    {
        let config = GeneratorConfig::default();
        self.generator.generate(rng, &config)
    }
}

/// Helper to create a vector generator (common migration pattern)
///
/// # Example
///
/// ```rust
/// use protest_proptest_compat::vec_generator;
/// use protest::primitives::IntGenerator;
///
/// // Create a generator for Vec<i32> with 0-100 elements
/// let generator = vec_generator(IntGenerator::new(0, 100), 0, 100);
/// ```
pub fn vec_generator<T, G>(
    element_gen: G,
    min_size: usize,
    max_size: usize,
) -> protest::primitives::VecGenerator<T, G>
where
    G: Generator<T>,
{
    protest::primitives::VecGenerator::new(element_gen, min_size, max_size)
}

/// Helper to create an option generator (common migration pattern)
///
/// # Example
///
/// ```rust
/// use protest_proptest_compat::option_generator;
/// use protest::primitives::IntGenerator;
///
/// // 50% chance of Some(value), 50% chance of None
/// let generator = option_generator(IntGenerator::new(0, 100), 0.5);
/// ```
pub fn option_generator<T, G>(
    inner: G,
    some_probability: f64,
) -> protest::primitives::OptionGenerator<T, G>
where
    G: Generator<T>,
{
    protest::primitives::OptionGenerator::with_probability(inner, some_probability)
}

/// Migration checklist and common patterns
///
/// Use this as a reference when migrating tests:
///
/// ## 1. Replace imports
/// - `use proptest::prelude::*;` → `use protest::*;` or `use protest::ergonomic::*;`
///
/// ## 2. Replace proptest! macro
/// - `proptest! { #[test] fn ... }` → Regular `#[test]` with `property!` or `property()` inside
///
/// ## 3. Replace strategies
/// - `0..100i32` → `generator!(i32, 0, 100)` or `IntGenerator::new(0, 100)`
/// - `Vec<i32>` → `VecGenerator::new(IntGenerator::new(...), min, max)`
/// - `prop::option::of(...)` → `OptionGenerator::new(..., probability)`
///
/// ## 4. Replace assertions
/// - Properties return `bool` or use `assert!` inside
/// - Proptest style: `prop_assert!` → Protest style: `assert!`
///
/// ## 5. Configuration
/// - Proptest: `.prop_map()` → Protest: `.map()` on generators
/// - Proptest: `.prop_filter()` → Protest: `.filter()` or preconditions in property
///
/// ## 6. Shrinking
/// - Protest has automatic shrinking built-in, no special configuration needed
///
/// # See Also
///
/// - Full migration examples in the `examples/` directory
/// - Comprehensive migration guide in README.md
pub struct MigrationGuide;

#[cfg(test)]
mod tests {
    use super::*;
    use protest::primitives::IntGenerator;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_range_to_generator() {
        let generator = range_to_generator(0, 100);
        let mut rng = StdRng::seed_from_u64(42);
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let value = generator.generate(&mut rng, &config);
            assert!((0..100).contains(&value));
        }
    }

    #[test]
    fn test_generator_adapter() {
        let generator = IntGenerator::new(0, 100);
        let adapter = GeneratorAdapter::new(generator);
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..10 {
            let value = adapter.generate(&mut rng);
            assert!((0..100).contains(&value));
        }
    }

    #[test]
    fn test_vec_generator_helper() {
        let generator = vec_generator(IntGenerator::new(0, 10), 5, 10);
        let mut rng = StdRng::seed_from_u64(42);
        let config = GeneratorConfig::default();

        let vec = generator.generate(&mut rng, &config);
        assert!(vec.len() >= 5 && vec.len() <= 10);
        for &val in &vec {
            assert!((0..=10).contains(&val));
        }
    }

    #[test]
    fn test_option_generator_helper() {
        let generator = option_generator(IntGenerator::new(0, 100), 0.5);
        let mut rng = StdRng::seed_from_u64(42);
        let config = GeneratorConfig::default();

        let mut some_count = 0;
        let mut none_count = 0;

        for _ in 0..100 {
            match generator.generate(&mut rng, &config) {
                Some(val) => {
                    assert!((0..100).contains(&val));
                    some_count += 1;
                }
                None => none_count += 1,
            }
        }

        // Should be roughly 50/50
        assert!(some_count > 20 && some_count < 80);
        assert!(none_count > 20 && none_count < 80);
    }
}
