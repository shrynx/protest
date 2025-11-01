//! Weighted operation generation based on operation weights
//!
//! This module provides a `WeightedGenerator` that generates random operations
//! according to their specified weights, with higher weights producing more
//! frequent operations.

use crate::operations::Operation;
use rand::Rng;
use std::marker::PhantomData;

/// A generator that produces random operations weighted by their frequency
///
/// Operations with higher weights will be generated more frequently.
/// Weights are specified using the `#[weight(N)]` attribute on operation variants.
///
/// # Example
///
/// ```
/// use protest_stateful::{Operation, operations::WeightedGenerator};
/// use rand::thread_rng;
///
/// #[derive(Debug, Clone, Operation)]
/// #[operation(state = "Vec<i32>")]
/// enum StackOp {
///     #[execute("state.push(42)")]
///     #[weight(5)]  // 5x more frequent than Clear
///     Push,
///
///     #[execute("state.pop()")]
///     #[precondition("!state.is_empty()")]
///     #[weight(3)]  // 3x more frequent than Clear
///     Pop,
///
///     #[execute("state.clear()")]
///     #[weight(1)]
///     Clear,
/// }
///
/// let mut rng = thread_rng();
/// let mut generator = WeightedGenerator::<StackOp, _>::new(
///     vec![StackOp::Push, StackOp::Pop, StackOp::Clear],
///     rng,
/// );
///
/// // Generate 10 weighted random operations
/// let ops = generator.generate(10);
/// // Push will appear ~5x more often than Clear
/// // Pop will appear ~3x more often than Clear
/// ```
pub struct WeightedGenerator<Op, R>
where
    Op: Operation,
    R: Rng,
{
    variants: Vec<Op>,
    weights: Vec<u32>,
    total_weight: u32,
    rng: R,
    _phantom: PhantomData<Op>,
}

impl<Op, R> WeightedGenerator<Op, R>
where
    Op: Operation,
    R: Rng,
{
    /// Create a new weighted generator from a list of operation variants
    ///
    /// The generator extracts weights from each operation using the `weight()` method,
    /// which is automatically implemented by the `#[derive(Operation)]` macro based on
    /// `#[weight(N)]` attributes.
    ///
    /// # Arguments
    ///
    /// * `variants` - Example instances of each operation variant
    /// * `rng` - Random number generator to use
    ///
    /// # Example
    ///
    /// ```
    /// use protest_stateful::{Operation, operations::WeightedGenerator};
    /// use rand::thread_rng;
    ///
    /// # #[derive(Debug, Clone, Operation)]
    /// # #[operation(state = "Vec<i32>")]
    /// # enum StackOp {
    /// #     #[execute("state.push(42)")]
    /// #     #[weight(5)]
    /// #     Push,
    /// #     #[execute("state.pop()")]
    /// #     #[weight(3)]
    /// #     Pop,
    /// # }
    /// let generator = WeightedGenerator::new(
    ///     vec![StackOp::Push, StackOp::Pop],
    ///     thread_rng(),
    /// );
    /// ```
    pub fn new(variants: Vec<Op>, rng: R) -> Self {
        let weights: Vec<u32> = variants.iter().map(|op| op.weight()).collect();
        let total_weight: u32 = weights.iter().sum();

        Self {
            variants,
            weights,
            total_weight,
            rng,
            _phantom: PhantomData,
        }
    }

    /// Generate a single random operation according to weights
    ///
    /// # Returns
    ///
    /// A randomly selected operation variant, with probability proportional to its weight
    pub fn generate_one(&mut self) -> Op {
        if self.variants.is_empty() {
            panic!("Cannot generate from empty variant list");
        }

        if self.total_weight == 0 {
            // If all weights are 0, use uniform distribution
            let idx = self.rng.gen_range(0..self.variants.len());
            return self.variants[idx].clone();
        }

        let mut roll = self.rng.gen_range(0..self.total_weight);

        for (idx, &weight) in self.weights.iter().enumerate() {
            if roll < weight {
                return self.variants[idx].clone();
            }
            roll -= weight;
        }

        // Fallback (shouldn't reach here)
        self.variants[0].clone()
    }

    /// Generate multiple random operations according to weights
    ///
    /// # Arguments
    ///
    /// * `count` - Number of operations to generate
    ///
    /// # Returns
    ///
    /// A vector of randomly generated operations
    ///
    /// # Example
    ///
    /// ```
    /// # use protest_stateful::{Operation, operations::WeightedGenerator};
    /// # use rand::thread_rng;
    /// # #[derive(Debug, Clone, Operation)]
    /// # #[operation(state = "Vec<i32>")]
    /// # enum StackOp {
    /// #     #[execute("state.push(42)")]
    /// #     #[weight(5)]
    /// #     Push,
    /// #     #[execute("state.pop()")]
    /// #     #[weight(1)]
    /// #     Pop,
    /// # }
    /// let mut generator = WeightedGenerator::new(
    ///     vec![StackOp::Push, StackOp::Pop],
    ///     thread_rng(),
    /// );
    ///
    /// let operations = generator.generate(100);
    /// assert_eq!(operations.len(), 100);
    /// ```
    pub fn generate(&mut self, count: usize) -> Vec<Op> {
        (0..count).map(|_| self.generate_one()).collect()
    }

    /// Get the weight of a specific variant by index
    pub fn weight_at(&self, index: usize) -> Option<u32> {
        self.weights.get(index).copied()
    }

    /// Get the total weight of all variants
    pub fn total_weight(&self) -> u32 {
        self.total_weight
    }

    /// Get statistics about the weight distribution
    ///
    /// Returns a vector of (variant_index, weight, percentage) tuples
    pub fn weight_distribution(&self) -> Vec<(usize, u32, f64)> {
        self.weights
            .iter()
            .enumerate()
            .map(|(idx, &weight)| {
                let percentage = if self.total_weight > 0 {
                    (weight as f64 / self.total_weight as f64) * 100.0
                } else {
                    100.0 / self.variants.len() as f64
                };
                (idx, weight, percentage)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[derive(Debug, Clone)]
    enum TestOp {
        Heavy,  // weight 5
        Medium, // weight 3
        Light,  // weight 1
        Zero,   // weight 0
    }

    impl Operation for TestOp {
        type State = ();

        fn execute(&self, _state: &mut Self::State) {}

        fn weight(&self) -> u32 {
            match self {
                TestOp::Heavy => 5,
                TestOp::Medium => 3,
                TestOp::Light => 1,
                TestOp::Zero => 0,
            }
        }
    }

    #[test]
    fn test_weighted_generation_distribution() {
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut generator =
            WeightedGenerator::new(vec![TestOp::Heavy, TestOp::Medium, TestOp::Light], rng);

        // Generate many operations and check distribution
        let count = 10000;
        let operations = generator.generate(count);

        let mut heavy_count = 0;
        let mut medium_count = 0;
        let mut light_count = 0;

        for op in operations {
            match op {
                TestOp::Heavy => heavy_count += 1,
                TestOp::Medium => medium_count += 1,
                TestOp::Light => light_count += 1,
                TestOp::Zero => {}
            }
        }

        // Total weights: 5 + 3 + 1 = 9
        // Expected: Heavy ~55.5%, Medium ~33.3%, Light ~11.1%
        let heavy_pct = heavy_count as f64 / count as f64;
        let medium_pct = medium_count as f64 / count as f64;
        let light_pct = light_count as f64 / count as f64;

        // Allow 5% deviation
        assert!((heavy_pct - 0.555).abs() < 0.05, "Heavy: {}", heavy_pct);
        assert!((medium_pct - 0.333).abs() < 0.05, "Medium: {}", medium_pct);
        assert!((light_pct - 0.111).abs() < 0.05, "Light: {}", light_pct);
    }

    #[test]
    fn test_weight_distribution() {
        let rng = ChaCha8Rng::seed_from_u64(42);
        let generator =
            WeightedGenerator::new(vec![TestOp::Heavy, TestOp::Medium, TestOp::Light], rng);

        let dist = generator.weight_distribution();
        assert_eq!(dist.len(), 3);

        assert_eq!(dist[0].1, 5); // Heavy weight
        assert!((dist[0].2 - 55.555).abs() < 0.01); // ~55.55%

        assert_eq!(dist[1].1, 3); // Medium weight
        assert!((dist[1].2 - 33.333).abs() < 0.01); // ~33.33%

        assert_eq!(dist[2].1, 1); // Light weight
        assert!((dist[2].2 - 11.111).abs() < 0.01); // ~11.11%
    }

    #[test]
    fn test_total_weight() {
        let rng = ChaCha8Rng::seed_from_u64(42);
        let generator =
            WeightedGenerator::new(vec![TestOp::Heavy, TestOp::Medium, TestOp::Light], rng);

        assert_eq!(generator.total_weight(), 9);
    }

    #[test]
    fn test_zero_weights_uniform_distribution() {
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut generator =
            WeightedGenerator::new(vec![TestOp::Zero, TestOp::Zero, TestOp::Zero], rng);

        // With all zero weights, should fall back to uniform distribution
        let operations = generator.generate(1000);
        assert_eq!(operations.len(), 1000);
    }

    #[test]
    fn test_single_operation() {
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut generator = WeightedGenerator::new(vec![TestOp::Heavy], rng);

        let op = generator.generate_one();
        assert!(matches!(op, TestOp::Heavy));
    }
}
