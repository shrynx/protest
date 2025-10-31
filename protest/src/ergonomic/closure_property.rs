//! Closure-based property definitions for ergonomic property testing.
//!
//! This module allows you to use closures directly as properties without
//! implementing the `Property` trait manually.

use crate::config::TestConfig;
use crate::error::PropertyError;
use crate::error::PropertyResult;
use crate::execution::check_with_config;
use crate::generator::Generator;
use crate::property::Property;

/// Trait for types that can act as property closures
///
/// This trait is automatically implemented for closures that return `bool`,
/// `Result<(), PropertyError>`, or `Result<T, PropertyError>`.
pub trait PropertyClosure<T> {
    /// The output type of the property
    type Output;

    /// Call the property closure with the given input
    fn call(&self, input: T) -> Result<Self::Output, PropertyError>;
}

// Implementation for closures that return bool
impl<F, T> PropertyClosure<T> for F
where
    F: Fn(T) -> bool,
{
    type Output = ();

    fn call(&self, input: T) -> Result<Self::Output, PropertyError> {
        if self(input) {
            Ok(())
        } else {
            Err(PropertyError::property_failed("Property returned false"))
        }
    }
}

/// Wrapper that converts a closure into a `Property` implementation
///
/// This struct wraps any type that implements `PropertyClosure` and makes it
/// usable with the standard property testing infrastructure.
pub struct ClosureProperty<F> {
    closure: F,
}

impl<F> ClosureProperty<F> {
    /// Create a new closure property from a closure
    pub fn new(closure: F) -> Self {
        Self { closure }
    }
}

impl<F, T> Property<T> for ClosureProperty<F>
where
    F: PropertyClosure<T>,
{
    type Output = F::Output;

    fn test(&self, input: T) -> Result<Self::Output, PropertyError> {
        self.closure.call(input)
    }
}

/// Convenience function to check a property using a closure
///
/// This function allows you to write property tests with minimal boilerplate
/// by passing a closure directly instead of implementing the `Property` trait.
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::check_with_closure;
/// use protest::range;
///
/// // Test that all positive integers are greater than zero
/// let result = check_with_closure(
///     range(1, 100),
///     |x: i32| x > 0
/// );
/// assert!(result.is_ok());
/// ```
///
/// ```rust
/// use protest::ergonomic::check_with_closure;
/// use protest::range;
///
/// // Test that doubling and halving returns the original value
/// let result = check_with_closure(
///     range(1, 100),
///     |x: i32| (x * 2) / 2 == x
/// );
/// assert!(result.is_ok());
/// ```
pub fn check_with_closure<T, G, F>(generator: G, closure: F) -> PropertyResult<T>
where
    T: Clone + std::fmt::Debug + PartialEq + 'static,
    G: Generator<T> + 'static,
    F: PropertyClosure<T>,
{
    let property = ClosureProperty::new(closure);
    check_with_config(generator, property, TestConfig::default())
}

/// Convenience function to check a property using a closure with custom configuration
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::check_with_closure_config;
/// use protest::{range, TestConfig};
///
/// let config = TestConfig {
///     iterations: 1000,
///     seed: Some(42),
///     ..TestConfig::default()
/// };
///
/// let result = check_with_closure_config(
///     range(1, 100),
///     |x: i32| x > 0,
///     config
/// );
/// assert!(result.is_ok());
/// ```
pub fn check_with_closure_config<T, G, F>(
    generator: G,
    closure: F,
    config: TestConfig,
) -> PropertyResult<T>
where
    T: Clone + std::fmt::Debug + PartialEq + 'static,
    G: Generator<T> + 'static,
    F: PropertyClosure<T>,
{
    let property = ClosureProperty::new(closure);
    check_with_config(generator, property, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::IntGenerator;

    #[test]
    fn test_closure_property_bool_return() {
        let result = check_with_closure(IntGenerator::new(1, 10), |x: i32| x > 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_closure_property_with_operations() {
        let result = check_with_closure(IntGenerator::new(-100, 100), |x: i32| x.abs() >= 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_closure_property_failing_case() {
        let result = check_with_closure(
            IntGenerator::new(1, 100),
            |x: i32| x < 50, // This will fail for values >= 50
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_closure_property_with_config() {
        let config = TestConfig {
            iterations: 10,
            seed: Some(42),
            ..TestConfig::default()
        };

        let result = check_with_closure_config(IntGenerator::new(1, 10), |x: i32| x > 0, config);
        assert!(result.is_ok());
    }
}
