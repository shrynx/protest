//! Fluent builder API for ergonomic property testing.
//!
//! This module provides a chainable builder interface that makes it easy to
//! configure and run property tests with minimal boilerplate.

#![allow(clippy::result_large_err)]

use std::marker::PhantomData;
use std::time::Duration;

use crate::config::TestConfig;
use crate::ergonomic::auto_gen::{AutoGen, InferredGenerator};
use crate::ergonomic::closure_property::{ClosureProperty, PropertyClosure};
use crate::error::PropertyResult;
use crate::execution::check_with_config;
use crate::generator::{BoxedGenerator, Generator};

/// Fluent builder for configuring and running property tests
///
/// This builder allows you to chain configuration methods and then execute
/// the property test with automatic generator inference.
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::builder::ErgonomicPropertyTest;
///
/// // Using the builder directly
/// let result = ErgonomicPropertyTest::<i32>::new()
///     .iterations(100)
///     .seed(42)
///     .run(|x: i32| x >= 0);
/// ```
pub struct ErgonomicPropertyTest<T> {
    config: TestConfig,
    generator: Option<BoxedGenerator<T>>,
    _phantom: PhantomData<T>,
}

impl<T> ErgonomicPropertyTest<T> {
    /// Create a new property test builder
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
            generator: None,
            _phantom: PhantomData,
        }
    }

    /// Set the number of test iterations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    ///
    /// let builder = ErgonomicPropertyTest::<i32>::new()
    ///     .iterations(1000);
    /// ```
    pub fn iterations(mut self, n: usize) -> Self {
        self.config.iterations = n;
        self
    }

    /// Set a specific random seed for reproducible tests
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    ///
    /// let builder = ErgonomicPropertyTest::<i32>::new()
    ///     .seed(42);
    /// ```
    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = Some(seed);
        self
    }

    /// Set the maximum number of shrink iterations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    ///
    /// let builder = ErgonomicPropertyTest::<i32>::new()
    ///     .max_shrink_iterations(500);
    /// ```
    pub fn max_shrink_iterations(mut self, n: usize) -> Self {
        self.config.max_shrink_iterations = n;
        self
    }

    /// Set the shrink timeout duration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    /// use std::time::Duration;
    ///
    /// let builder = ErgonomicPropertyTest::<i32>::new()
    ///     .shrink_timeout(Duration::from_secs(5));
    /// ```
    pub fn shrink_timeout(mut self, timeout: Duration) -> Self {
        self.config.shrink_timeout = timeout;
        self
    }

    /// Set the generator size hint
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    ///
    /// let builder = ErgonomicPropertyTest::<Vec<i32>>::new()
    ///     .size_hint(50);
    /// ```
    pub fn size_hint(mut self, hint: usize) -> Self {
        self.config.generator_config.size_hint = hint;
        self
    }

    /// Set the maximum generation depth for nested structures
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    ///
    /// let builder = ErgonomicPropertyTest::<Vec<Vec<i32>>>::new()
    ///     .max_depth(10);
    /// ```
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.generator_config.max_depth = depth;
        self
    }

    /// Provide a custom generator instead of using automatic inference
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    /// use protest::primitives::IntGenerator;
    ///
    /// let builder = ErgonomicPropertyTest::<i32>::new()
    ///     .with_generator(IntGenerator::new(0, 100));
    /// ```
    pub fn with_generator<G: Generator<T> + Send + Sync + 'static>(mut self, generator: G) -> Self {
        self.generator = Some(BoxedGenerator::new(generator));
        self
    }

    /// Run the property test with a closure property
    ///
    /// This method accepts any closure that implements `PropertyClosure` and
    /// automatically infers the generator if one wasn't provided.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    ///
    /// let result = ErgonomicPropertyTest::<i32>::new()
    ///     .iterations(100)
    ///     .run(|x: i32| x.abs() >= 0);
    /// ```
    pub fn run<F>(self, closure: F) -> PropertyResult<T>
    where
        T: Clone + std::fmt::Debug + PartialEq + AutoGen + Send + Sync + 'static,
        F: PropertyClosure<T>,
    {
        let property = ClosureProperty::new(closure);
        let generator = self
            .generator
            .unwrap_or_else(|| BoxedGenerator::new(InferredGenerator::<T>::new()));
        check_with_config(generator, property, self.config)
    }

    /// Run the property test with an explicit generator and closure
    ///
    /// This is useful when you want to use a custom generator but still
    /// use the builder pattern for configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest::ergonomic::builder::ErgonomicPropertyTest;
    /// use protest::primitives::IntGenerator;
    ///
    /// let result = ErgonomicPropertyTest::<i32>::new()
    ///     .iterations(100)
    ///     .run_with(IntGenerator::new(0, 50), |x: i32| x >= 0);
    /// ```
    pub fn run_with<G, F>(self, generator: G, closure: F) -> PropertyResult<T>
    where
        T: Clone + std::fmt::Debug + PartialEq + 'static,
        G: Generator<T> + 'static,
        F: PropertyClosure<T>,
    {
        let property = ClosureProperty::new(closure);
        check_with_config(generator, property, self.config)
    }
}

impl<T> Default for ErgonomicPropertyTest<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a property test builder with type inference
///
/// This function creates a builder that will automatically infer the appropriate
/// generator based on the closure's parameter type.
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::builder::property;
///
/// // Type is inferred from the closure
/// let result = property(|x: i32| x >= 0)
///     .iterations(100)
///     .seed(42)
///     .run();
/// ```
pub fn property<T, F>(_closure: F) -> ErgonomicPropertyTestWithClosure<T, F>
where
    F: PropertyClosure<T>,
{
    ErgonomicPropertyTestWithClosure {
        closure: _closure,
        config: TestConfig::default(),
        generator: None,
        _phantom: PhantomData,
    }
}

/// Builder that holds both configuration and the closure
///
/// This is returned by the `property()` function and allows for a more
/// ergonomic API where the closure is provided upfront.
pub struct ErgonomicPropertyTestWithClosure<T, F> {
    closure: F,
    config: TestConfig,
    generator: Option<BoxedGenerator<T>>,
    _phantom: PhantomData<T>,
}

impl<T, F> ErgonomicPropertyTestWithClosure<T, F>
where
    F: PropertyClosure<T>,
{
    /// Set the number of test iterations
    pub fn iterations(mut self, n: usize) -> Self {
        self.config.iterations = n;
        self
    }

    /// Set a specific random seed for reproducible tests
    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = Some(seed);
        self
    }

    /// Set the maximum number of shrink iterations
    pub fn max_shrink_iterations(mut self, n: usize) -> Self {
        self.config.max_shrink_iterations = n;
        self
    }

    /// Set the shrink timeout duration
    pub fn shrink_timeout(mut self, timeout: Duration) -> Self {
        self.config.shrink_timeout = timeout;
        self
    }

    /// Set the generator size hint
    pub fn size_hint(mut self, hint: usize) -> Self {
        self.config.generator_config.size_hint = hint;
        self
    }

    /// Set the maximum generation depth for nested structures
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.generator_config.max_depth = depth;
        self
    }

    /// Provide a custom generator instead of using automatic inference
    pub fn with_generator<G: Generator<T> + Send + Sync + 'static>(mut self, generator: G) -> Self {
        self.generator = Some(BoxedGenerator::new(generator));
        self
    }

    /// Run the property test with automatic generator inference
    pub fn run(self) -> PropertyResult<T>
    where
        T: Clone + std::fmt::Debug + PartialEq + AutoGen + Send + Sync + 'static,
    {
        let property = ClosureProperty::new(self.closure);
        let generator = self
            .generator
            .unwrap_or_else(|| BoxedGenerator::new(InferredGenerator::<T>::new()));
        check_with_config(generator, property, self.config)
    }

    /// Run the property test with an explicit generator
    pub fn run_with<G>(self, generator: G) -> PropertyResult<T>
    where
        T: Clone + std::fmt::Debug + PartialEq + 'static,
        G: Generator<T> + 'static,
    {
        let property = ClosureProperty::new(self.closure);
        check_with_config(generator, property, self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::IntGenerator;

    #[test]
    fn test_builder_basic() {
        let result = ErgonomicPropertyTest::<i32>::new()
            .iterations(10)
            .run_with(IntGenerator::new(1, 100), |x: i32| x > 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_seed() {
        let result = ErgonomicPropertyTest::<i32>::new()
            .iterations(10)
            .seed(42)
            .run_with(IntGenerator::new(1, 100), |x: i32| x > 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_custom_generator() {
        let result = ErgonomicPropertyTest::<i32>::new()
            .iterations(10)
            .with_generator(IntGenerator::new(5, 10))
            .run_with(IntGenerator::new(5, 10), |x: i32| (5..=10).contains(&x));
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_function_basic() {
        let result = property(|x: i32| x >= 0)
            .iterations(10)
            .run_with(IntGenerator::new(0, 100));
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_function_with_seed() {
        let result = property(|x: i32| x.abs() >= 0)
            .iterations(10)
            .seed(123)
            .run_with(IntGenerator::new(-100, 100));
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_config_chaining() {
        let result = ErgonomicPropertyTest::<i32>::new()
            .iterations(20)
            .seed(42)
            .max_shrink_iterations(100)
            .size_hint(5)
            .max_depth(3)
            .run_with(IntGenerator::new(1, 50), |x: i32| x > 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_failing_case() {
        let result = property(|x: i32| x < 50)
            .iterations(10)
            .run_with(IntGenerator::new(1, 100));
        assert!(result.is_err());
    }
}
