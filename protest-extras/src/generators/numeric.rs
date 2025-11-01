//! Constrained numeric generators
//!
//! This module provides generators for numbers with specific constraints:
//! - Positive integers
//! - Even numbers
//! - Prime numbers
//! - Percentage values (0.0-100.0)
//!
//! All generators use std library only.

use protest::{Generator, GeneratorConfig};
use rand::Rng;

// ============================================================================
// Positive Integer Generator
// ============================================================================

/// Generator for positive integers (> 0)
///
/// Generic over any unsigned integer type
#[derive(Debug, Clone)]
pub struct PositiveIntGenerator<T> {
    min: T,
    max: T,
}

impl<T> PositiveIntGenerator<T>
where
    T: num_traits::PrimInt + num_traits::Unsigned + rand::distributions::uniform::SampleUniform,
{
    /// Create a new positive integer generator
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> Generator<T> for PositiveIntGenerator<T>
where
    T: num_traits::PrimInt
        + num_traits::Unsigned
        + rand::distributions::uniform::SampleUniform
        + Clone
        + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> T {
        rng.r#gen_range(self.min..=self.max)
    }

    fn shrink(&self, value: &T) -> Box<dyn Iterator<Item = T>> {
        let mut shrinks = Vec::new();
        let one = T::one();

        // Try shrinking toward min
        if *value > self.min {
            shrinks.push(self.min);
        }

        // Try shrinking toward 1 (smallest positive)
        if *value > one && one >= self.min {
            shrinks.push(one);
        }

        // Try half
        let two = one + one;
        if *value > two {
            shrinks.push(*value / two);
        }

        // Try value - 1
        if *value > self.min {
            shrinks.push(*value - one);
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Even Number Generator
// ============================================================================

/// Generator for even numbers
#[derive(Debug, Clone)]
pub struct EvenNumberGenerator<T> {
    min: T,
    max: T,
}

impl<T> EvenNumberGenerator<T>
where
    T: num_traits::PrimInt,
{
    /// Create a new even number generator
    ///
    /// Note: min and max will be adjusted to the nearest even numbers if needed
    pub fn new(min: T, max: T) -> Self {
        let two = T::one() + T::one();

        // Adjust min to nearest even number >= min
        let adjusted_min = if min % two == T::zero() {
            min
        } else {
            min + T::one()
        };

        // Adjust max to nearest even number <= max
        let adjusted_max = if max % two == T::zero() {
            max
        } else {
            max - T::one()
        };

        Self {
            min: adjusted_min,
            max: adjusted_max,
        }
    }
}

impl<T> Generator<T> for EvenNumberGenerator<T>
where
    T: num_traits::PrimInt + rand::distributions::uniform::SampleUniform + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> T {
        let two = T::one() + T::one();
        let range = (self.max - self.min) / two + T::one();
        let offset = rng.r#gen_range(T::zero()..range);
        self.min + (offset * two)
    }

    fn shrink(&self, value: &T) -> Box<dyn Iterator<Item = T>> {
        let mut shrinks = Vec::new();
        let two = T::one() + T::one();

        // Try zero if in range
        if T::zero() >= self.min && T::zero() <= self.max && *value != T::zero() {
            shrinks.push(T::zero());
        }

        // Try min
        if *value > self.min {
            shrinks.push(self.min);
        }

        // Try half (rounded to even)
        if *value > two {
            let half = *value / two;
            let half_even = if half % two == T::zero() {
                half
            } else {
                half - T::one()
            };
            if half_even >= self.min {
                shrinks.push(half_even);
            }
        }

        // Try value - 2
        if *value > self.min + two {
            shrinks.push(*value - two);
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Prime Number Generator
// ============================================================================

/// Generator for prime numbers
#[derive(Debug, Clone)]
pub struct PrimeNumberGenerator {
    min: u64,
    #[allow(dead_code)]
    max: u64,
    primes_cache: Vec<u64>,
}

impl PrimeNumberGenerator {
    /// Create a new prime number generator
    pub fn new(min: u64, max: u64) -> Self {
        let primes = Self::generate_primes_in_range(min, max);
        Self {
            min,
            max,
            primes_cache: primes,
        }
    }

    fn is_prime(n: u64) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 {
            return true;
        }
        if n.is_multiple_of(2) {
            return false;
        }

        let sqrt = (n as f64).sqrt() as u64;
        for i in (3..=sqrt).step_by(2) {
            if n.is_multiple_of(i) {
                return false;
            }
        }
        true
    }

    fn generate_primes_in_range(min: u64, max: u64) -> Vec<u64> {
        let start = if min <= 2 { 2 } else { min };
        (start..=max).filter(|&n| Self::is_prime(n)).collect()
    }
}

impl Generator<u64> for PrimeNumberGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> u64 {
        if self.primes_cache.is_empty() {
            // No primes in range, return min
            self.min
        } else {
            let idx = rng.r#gen_range(0..self.primes_cache.len());
            self.primes_cache[idx]
        }
    }

    fn shrink(&self, value: &u64) -> Box<dyn Iterator<Item = u64>> {
        let mut shrinks = Vec::new();

        // Try smaller primes
        for &prime in &self.primes_cache {
            if prime < *value {
                shrinks.push(prime);
                if shrinks.len() >= 5 {
                    break;
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Percentage Generator
// ============================================================================

/// Generator for percentage values (0.0 to 100.0)
#[derive(Debug, Clone, Copy)]
pub struct PercentageGenerator {
    allow_zero: bool,
    allow_hundred: bool,
}

impl PercentageGenerator {
    /// Create a new percentage generator (0.0 to 100.0 inclusive)
    pub fn new() -> Self {
        Self {
            allow_zero: true,
            allow_hundred: true,
        }
    }

    /// Create a percentage generator excluding 0.0
    pub fn positive() -> Self {
        Self {
            allow_zero: false,
            allow_hundred: true,
        }
    }

    /// Create a percentage generator with custom bounds
    pub fn with_bounds(allow_zero: bool, allow_hundred: bool) -> Self {
        Self {
            allow_zero,
            allow_hundred,
        }
    }
}

impl Default for PercentageGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator<f64> for PercentageGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> f64 {
        let min = if self.allow_zero { 0.0 } else { 0.01 };
        let max = if self.allow_hundred { 100.0 } else { 99.99 };

        rng.r#gen_range(min..=max)
    }

    fn shrink(&self, value: &f64) -> Box<dyn Iterator<Item = f64>> {
        let mut shrinks = Vec::new();

        // Try common values
        if self.allow_zero && *value > 0.0 {
            shrinks.push(0.0);
        }

        if *value > 1.0 {
            shrinks.push(1.0);
        }

        if *value > 50.0 {
            shrinks.push(50.0);
        }

        if self.allow_hundred && *value < 100.0 {
            shrinks.push(100.0);
        }

        // Try half
        if *value > 1.0 {
            shrinks.push(*value / 2.0);
        }

        Box::new(shrinks.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_positive_int_generator() {
        let generator = PositiveIntGenerator::new(1u32, 100u32);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..20 {
            let n = generator.generate(&mut rng, &config);
            assert!((1..=100).contains(&n));
            assert!(n > 0);
        }
    }

    #[test]
    fn test_even_number_generator() {
        let generator = EvenNumberGenerator::new(0i32, 100i32);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..20 {
            let n = generator.generate(&mut rng, &config);
            assert!((0..=100).contains(&n));
            assert_eq!(n % 2, 0, "Generated number {} should be even", n);
        }
    }

    #[test]
    fn test_even_number_generator_odd_bounds() {
        // Test that odd bounds get adjusted correctly
        let generator = EvenNumberGenerator::new(1i32, 99i32);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..20 {
            let n = generator.generate(&mut rng, &config);
            assert!((2..=98).contains(&n)); // Adjusted to 2..98
            assert_eq!(n % 2, 0);
        }
    }

    #[test]
    fn test_prime_number_generator() {
        let generator = PrimeNumberGenerator::new(2, 50);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..20 {
            let n = generator.generate(&mut rng, &config);
            assert!((2..=50).contains(&n));
            assert!(PrimeNumberGenerator::is_prime(n));
        }
    }

    #[test]
    fn test_prime_number_is_prime() {
        assert!(PrimeNumberGenerator::is_prime(2));
        assert!(PrimeNumberGenerator::is_prime(3));
        assert!(PrimeNumberGenerator::is_prime(5));
        assert!(PrimeNumberGenerator::is_prime(7));
        assert!(PrimeNumberGenerator::is_prime(11));
        assert!(PrimeNumberGenerator::is_prime(13));
        assert!(PrimeNumberGenerator::is_prime(97));

        assert!(!PrimeNumberGenerator::is_prime(0));
        assert!(!PrimeNumberGenerator::is_prime(1));
        assert!(!PrimeNumberGenerator::is_prime(4));
        assert!(!PrimeNumberGenerator::is_prime(6));
        assert!(!PrimeNumberGenerator::is_prime(8));
        assert!(!PrimeNumberGenerator::is_prime(9));
        assert!(!PrimeNumberGenerator::is_prime(100));
    }

    #[test]
    fn test_percentage_generator() {
        let generator = PercentageGenerator::new();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..20 {
            let p = generator.generate(&mut rng, &config);
            assert!((0.0..=100.0).contains(&p));
        }
    }

    #[test]
    fn test_percentage_generator_positive() {
        let generator = PercentageGenerator::positive();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..20 {
            let p = generator.generate(&mut rng, &config);
            assert!(p > 0.0 && p <= 100.0);
        }
    }
}
