//! Declarative macros for ultra-ergonomic property testing.
//!
//! This module provides macros that further reduce boilerplate for common
//! property testing patterns.

/// Create and run a property test with minimal syntax
///
/// This macro accepts a closure and optional configuration, automatically
/// infers the generator, and runs the property test.
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use protest::property;
/// use protest::primitives::IntGenerator;
///
/// property!(IntGenerator::new(0, 100), |x: i32| x >= 0);
/// ```
///
/// With configuration:
/// ```rust
/// use protest::property;
/// use protest::primitives::IntGenerator;
///
/// property!(
///     IntGenerator::new(0, 100),
///     iterations = 1000,
///     seed = 42,
///     |x: i32| x >= 0
/// );
/// ```
#[macro_export]
macro_rules! property {
    // Basic form: property!(generator, closure)
    ($generator:expr, $closure:expr) => {{ $crate::ergonomic::check_with_closure($generator, $closure) }};

    // With iterations: property!(generator, iterations = N, closure)
    ($generator:expr, iterations = $n:expr, $closure:expr) => {{
        $crate::ergonomic::check_with_closure_config(
            $generator,
            $closure,
            $crate::TestConfig {
                iterations: $n,
                ..$crate::TestConfig::default()
            },
        )
    }};

    // With seed: property!(generator, seed = N, closure)
    ($generator:expr, seed = $seed:expr, $closure:expr) => {{
        $crate::ergonomic::check_with_closure_config(
            $generator,
            $closure,
            $crate::TestConfig {
                seed: Some($seed),
                ..$crate::TestConfig::default()
            },
        )
    }};

    // With both iterations and seed
    ($generator:expr, iterations = $n:expr, seed = $seed:expr, $closure:expr) => {{
        $crate::ergonomic::check_with_closure_config(
            $generator,
            $closure,
            $crate::TestConfig {
                iterations: $n,
                seed: Some($seed),
                ..$crate::TestConfig::default()
            },
        )
    }};
}

/// Create a generator for a given type
///
/// This macro provides syntactic sugar for creating generators.
///
/// # Examples
///
/// ```rust
/// use protest::{generator, IntGenerator};
///
/// // Create an integer generator
/// let my_generator = generator!(i32, 0, 100);
/// ```
#[macro_export]
macro_rules! generator {
    // Range syntax: generator!(i32, 0, 100)
    ($ty:ty, $start:expr, $end:expr) => {{ $crate::primitives::IntGenerator::<$ty>::new($start, $end) }}; // Type-based inference would require procedural macros
                                                                                                          // For now, users should use AutoGen trait directly
}

/// Assert that a property holds, panicking if it fails
///
/// This is useful for tests where you want assertion-style failures.
///
/// # Examples
///
/// ```rust
/// use protest::assert_property;
/// use protest::primitives::IntGenerator;
///
/// assert_property!(IntGenerator::new(1, 100), |x: i32| x > 0);
/// ```
///
/// With a custom message:
/// ```rust
/// use protest::assert_property;
/// use protest::primitives::IntGenerator;
///
/// assert_property!(
///     IntGenerator::new(1, 100),
///     |x: i32| x > 0,
///     "All positive numbers should be greater than zero"
/// );
/// ```
#[macro_export]
macro_rules! assert_property {
    ($generator:expr, $closure:expr) => {{
        let result = $crate::ergonomic::check_with_closure($generator, $closure);
        if let Err(failure) = result {
            panic!("Property assertion failed: {}", failure.error);
        }
    }};

    ($generator:expr, $closure:expr, $msg:expr) => {{
        let result = $crate::ergonomic::check_with_closure($generator, $closure);
        if let Err(failure) = result {
            panic!("{}: {}", $msg, failure.error);
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::generator::Generator;
    use crate::primitives::IntGenerator;

    #[test]
    fn test_property_macro_basic() {
        let result = property!(IntGenerator::new(1, 10), |x: i32| x > 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_macro_with_iterations() {
        let result = property!(IntGenerator::new(1, 10), iterations = 50, |x: i32| x > 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_macro_with_seed() {
        let result = property!(IntGenerator::new(1, 10), seed = 42, |x: i32| x > 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_macro_with_both() {
        let result = property!(
            IntGenerator::new(1, 10),
            iterations = 25,
            seed = 123,
            |x: i32| x > 0
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_generator_macro_range() {
        let my_generator = generator!(i32, 0, 100);
        let mut rng = rand::thread_rng();
        let config = crate::config::GeneratorConfig::default();
        let value = my_generator.generate(&mut rng, &config);
        assert!((0..=100).contains(&value));
    }

    #[test]
    fn test_assert_property_passing() {
        assert_property!(IntGenerator::new(1, 10), |x: i32| x > 0);
    }

    #[test]
    fn test_assert_property_with_message() {
        assert_property!(
            IntGenerator::new(1, 10),
            |x: i32| x > 0,
            "Positive numbers should be greater than zero"
        );
    }

    #[test]
    #[should_panic(expected = "Property assertion failed")]
    fn test_assert_property_failing() {
        assert_property!(IntGenerator::new(1, 100), |x: i32| x < 50);
    }
}
