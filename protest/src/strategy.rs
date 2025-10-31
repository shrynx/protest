//! Strategy-based generation system for composable test data creation.

use crate::config::GeneratorConfig;
use crate::generator::Generator;

/// A strategy for generating values of a specific type
///
/// Strategies are composable and can be combined to create more complex generation patterns.
/// They encapsulate both the generation logic and the shrinking behavior.
pub trait Strategy {
    /// The type of values this strategy generates
    type Value: 'static;

    /// Generate a value using this strategy
    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Self::Value;

    /// Create an iterator of shrunk values from the given value
    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>>;

    /// Map this strategy to produce values of a different type
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Value) -> U,
        U: 'static,
    {
        Map {
            strategy: self,
            mapper: f,
        }
    }

    /// Filter values produced by this strategy
    fn filter<F>(self, predicate: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Value) -> bool,
    {
        Filter {
            strategy: self,
            predicate,
        }
    }

    /// Combine this strategy with another to produce tuples
    fn zip<S>(self, other: S) -> Zip<Self, S>
    where
        Self: Sized,
        S: Strategy,
    {
        Zip {
            left: self,
            right: other,
        }
    }
}

/// A strategy that maps values from one type to another
pub struct Map<S, F> {
    strategy: S,
    mapper: F,
}

impl<S, F, U> Strategy for Map<S, F>
where
    S: Strategy,
    F: Fn(S::Value) -> U,
    U: 'static,
{
    type Value = U;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Self::Value {
        let value = self.strategy.generate(rng, config);
        (self.mapper)(value)
    }

    fn shrink(&self, _value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // For mapped strategies, we can't easily shrink the mapped value back to the original
        // This is a limitation that can be improved with more sophisticated shrinking
        Box::new(std::iter::empty())
    }
}

/// A strategy that filters values based on a predicate
pub struct Filter<S, F> {
    strategy: S,
    predicate: F,
}

impl<S, F> Strategy for Filter<S, F>
where
    S: Strategy,
    F: Fn(&S::Value) -> bool,
{
    type Value = S::Value;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Self::Value {
        // Try to generate a value that passes the filter
        // In a real implementation, we'd want to limit attempts to avoid infinite loops
        for _ in 0..1000 {
            // Limit attempts to avoid infinite loops
            let value = self.strategy.generate(rng, config);
            if (self.predicate)(&value) {
                return value;
            }
        }
        panic!("Filter strategy failed to generate a valid value after 1000 attempts");
    }

    fn shrink(&self, _value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Simplified shrinking for now to avoid lifetime issues
        Box::new(std::iter::empty())
    }
}

/// A strategy that combines two strategies to produce tuples
pub struct Zip<L, R> {
    left: L,
    right: R,
}

impl<L, R> Strategy for Zip<L, R>
where
    L: Strategy,
    R: Strategy,
    L::Value: Clone,
    R::Value: Clone,
{
    type Value = (L::Value, R::Value);

    fn generate<RNG: rand::Rng>(&self, rng: &mut RNG, config: &GeneratorConfig) -> Self::Value {
        let left_value = self.left.generate(rng, config);
        let right_value = self.right.generate(rng, config);
        (left_value, right_value)
    }

    fn shrink(&self, _value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // For now, return empty iterator to avoid complex lifetime issues
        // This can be improved with a more sophisticated shrinking implementation
        Box::new(std::iter::empty())
    }
}

/// A strategy that always produces the same value
#[derive(Debug, Clone)]
pub struct Just<T> {
    value: T,
}

impl<T: Clone + 'static> Strategy for Just<T> {
    type Value = T;

    fn generate<R: rand::Rng>(&self, _rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
        self.value.clone()
    }

    fn shrink(&self, _value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // A constant value cannot be shrunk
        Box::new(std::iter::empty())
    }
}

// Also implement Generator for Just
impl<T> Generator<T> for Just<T>
where
    T: Clone + std::fmt::Debug + PartialEq + 'static,
{
    fn generate(&self, _rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> T {
        self.value.clone()
    }

    fn shrink(&self, _value: &T) -> Box<dyn Iterator<Item = T>> {
        Box::new(std::iter::empty())
    }
}

/// A strategy that chooses randomly from a collection of values
#[derive(Debug, Clone)]
pub struct OneOf<T> {
    values: Vec<T>,
}

impl<T: Clone + 'static> Strategy for OneOf<T> {
    type Value = T;

    fn generate<R: rand::Rng>(&self, rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
        if self.values.is_empty() {
            panic!("OneOf strategy cannot generate from empty collection");
        }
        let index = rng.gen_range(0..self.values.len());
        self.values[index].clone()
    }

    fn shrink(&self, _value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // For OneOf, we can try other values in the collection as shrinks
        // This is a simple approach - more sophisticated shrinking could be implemented
        // For now, return empty iterator to avoid lifetime issues
        Box::new(std::iter::empty())
    }
}

/// A strategy for generating values in a numeric range
#[derive(Debug, Clone)]
pub struct Range<T> {
    start: T,
    end: T,
}

impl<T> Strategy for Range<T>
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd + Copy + Clone + 'static,
{
    type Value = T;

    fn generate<R: rand::Rng>(&self, rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
        rng.gen_range(self.start..=self.end)
    }

    fn shrink(&self, _value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Numeric shrinking would move towards zero or the start of the range
        // This is a placeholder for now
        Box::new(std::iter::empty())
    }
}

// Also implement Generator for Range so it can be used with check_with_config
impl<T> Generator<T> for Range<T>
where
    T: rand::distributions::uniform::SampleUniform
        + PartialOrd
        + Copy
        + Clone
        + std::fmt::Debug
        + PartialEq
        + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> T {
        use rand::Rng;
        // Call generate directly using RngCore methods
        rng.gen_range(self.start..=self.end)
    }

    fn shrink(&self, value: &T) -> Box<dyn Iterator<Item = T>> {
        Strategy::shrink(self, value)
    }
}

/// Create a strategy that always produces the same value
pub fn just<T: Clone>(value: T) -> Just<T> {
    Just { value }
}

/// Create a strategy that chooses from a collection of values
pub fn one_of<T: Clone>(values: Vec<T>) -> OneOf<T> {
    OneOf { values }
}

/// Create a strategy for generating values in a range
pub fn range<T>(start: T, end: T) -> Range<T>
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd + Copy,
{
    Range { start, end }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_just_strategy() {
        let strategy = just(42);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = Strategy::generate(&strategy, &mut rng, &config);
        assert_eq!(value, 42);

        // Just strategy should not produce any shrinks
        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();
        assert!(shrinks.is_empty());
    }

    #[test]
    fn test_one_of_strategy() {
        let values = vec![1, 2, 3, 4, 5];
        let strategy = one_of(values.clone());
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = Strategy::generate(&strategy, &mut rng, &config);
        assert!(values.contains(&value));
    }

    #[test]
    fn test_range_strategy() {
        let strategy = range(1, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = Strategy::generate(&strategy, &mut rng, &config);
        assert!((1..=10).contains(&value));
    }

    #[test]
    fn test_strategy_map() {
        let strategy = just(5).map(|x| x * 2);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_strategy_zip() {
        let strategy = just(42).zip(range(1, 5));
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (left, right) = strategy.generate(&mut rng, &config);
        assert_eq!(left, 42);
        assert!((1..=5).contains(&right));
    }

    #[test]
    fn test_strategy_filter() {
        let strategy = range(1, 100).filter(|x| x % 2 == 0);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert!((1..=100).contains(&value));
        assert_eq!(value % 2, 0); // Should be even
    }

    #[test]
    fn test_strategy_composition() {
        // Test complex composition: map a range to strings, then zip with a constant
        let strategy = range(1, 5).map(|x| format!("value_{}", x)).zip(just(true));

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (string_val, bool_val) = strategy.generate(&mut rng, &config);
        assert!(string_val.starts_with("value_"));
        assert!(bool_val);
    }
}
