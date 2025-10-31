//! Specialized collection generators
//!
//! This module provides generators for collections with specific properties:
//! - Non-empty vectors (guaranteed length >= 1)
//! - Sorted collections
//! - Unique element collections
//! - Bounded size maps
//!
//! All generators use std library only.

use protest::{Generator, GeneratorConfig};
use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;

// ============================================================================
// NonEmpty Vec Generator
// ============================================================================

/// Generator for non-empty vectors
///
/// Guarantees that generated vectors always have at least one element
#[derive(Debug, Clone)]
pub struct NonEmptyVecGenerator<T, G> {
    element_generator: G,
    min_len: usize,
    max_len: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G> NonEmptyVecGenerator<T, G>
where
    G: Generator<T>,
{
    /// Create a new non-empty vec generator
    ///
    /// min_len will be adjusted to at least 1
    pub fn new(element_generator: G, min_len: usize, max_len: usize) -> Self {
        Self {
            element_generator,
            min_len: min_len.max(1),
            max_len: max_len.max(1),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G> Generator<Vec<T>> for NonEmptyVecGenerator<T, G>
where
    T: Clone + 'static,
    G: Generator<T> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Vec<T> {
        let len = rng.r#gen_range(self.min_len..=self.max_len);
        (0..len)
            .map(|_| self.element_generator.generate(rng, config))
            .collect()
    }

    fn shrink(&self, value: &Vec<T>) -> Box<dyn Iterator<Item = Vec<T>>> {
        let mut shrinks = Vec::new();

        // Try minimal size (1 element)
        if value.len() > 1 {
            shrinks.push(vec![value[0].clone()]);
        }

        // Try removing elements from end
        if value.len() > self.min_len {
            shrinks.push(value[..self.min_len].to_vec());
        }

        // Try removing one element at a time
        if value.len() > self.min_len {
            for i in 0..value.len().min(3) {
                if i < value.len() {
                    let mut shrunk = value.clone();
                    shrunk.remove(i);
                    if shrunk.len() >= self.min_len {
                        shrinks.push(shrunk);
                    }
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Sorted Vec Generator
// ============================================================================

/// Generator for sorted vectors
///
/// Generates vectors that are already sorted in ascending order
#[derive(Debug, Clone)]
pub struct SortedVecGenerator<T, G> {
    element_generator: G,
    min_len: usize,
    max_len: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G> SortedVecGenerator<T, G>
where
    G: Generator<T>,
{
    /// Create a new sorted vec generator
    pub fn new(element_generator: G, min_len: usize, max_len: usize) -> Self {
        Self {
            element_generator,
            min_len,
            max_len,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G> Generator<Vec<T>> for SortedVecGenerator<T, G>
where
    T: Clone + Ord + 'static,
    G: Generator<T> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Vec<T> {
        let len = rng.r#gen_range(self.min_len..=self.max_len);
        let mut vec: Vec<T> = (0..len)
            .map(|_| self.element_generator.generate(rng, config))
            .collect();
        vec.sort();
        vec
    }

    fn shrink(&self, value: &Vec<T>) -> Box<dyn Iterator<Item = Vec<T>>> {
        let mut shrinks = Vec::new();

        // Try empty if allowed
        if !value.is_empty() && self.min_len == 0 {
            shrinks.push(vec![]);
        }

        // Try shrinking to min length
        if value.len() > self.min_len {
            shrinks.push(value[..self.min_len].to_vec());
        }

        // Try removing elements
        if value.len() > self.min_len {
            for i in 0..value.len().min(3) {
                if i < value.len() {
                    let mut shrunk = value.clone();
                    shrunk.remove(i);
                    if shrunk.len() >= self.min_len {
                        // Should still be sorted after removal
                        shrinks.push(shrunk);
                    }
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Unique Vec Generator
// ============================================================================

/// Generator for vectors with unique elements (no duplicates)
#[derive(Debug, Clone)]
pub struct UniqueVecGenerator<T, G> {
    element_generator: G,
    min_len: usize,
    max_len: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G> UniqueVecGenerator<T, G>
where
    G: Generator<T>,
{
    /// Create a new unique vec generator
    pub fn new(element_generator: G, min_len: usize, max_len: usize) -> Self {
        Self {
            element_generator,
            min_len,
            max_len,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G> Generator<Vec<T>> for UniqueVecGenerator<T, G>
where
    T: Clone + Eq + Hash + 'static,
    G: Generator<T> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Vec<T> {
        let target_len = rng.r#gen_range(self.min_len..=self.max_len);
        let mut vec = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Try to generate unique elements
        let mut attempts = 0;
        let max_attempts = target_len * 10;

        while vec.len() < target_len && attempts < max_attempts {
            let elem = self.element_generator.generate(rng, config);
            if seen.insert(elem.clone()) {
                vec.push(elem);
            }
            attempts += 1;
        }

        vec
    }

    fn shrink(&self, value: &Vec<T>) -> Box<dyn Iterator<Item = Vec<T>>> {
        let mut shrinks = Vec::new();

        // Try empty if allowed
        if !value.is_empty() && self.min_len == 0 {
            shrinks.push(vec![]);
        }

        // Try shrinking to min length
        if value.len() > self.min_len {
            shrinks.push(value[..self.min_len].to_vec());
        }

        // Try removing one element at a time
        if value.len() > self.min_len {
            for i in 0..value.len().min(3) {
                if i < value.len() {
                    let mut shrunk = value.clone();
                    shrunk.remove(i);
                    if shrunk.len() >= self.min_len {
                        shrinks.push(shrunk);
                    }
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Bounded Map Generator
// ============================================================================

/// Generator for HashMap with size bounds
#[derive(Debug, Clone)]
pub struct BoundedMapGenerator<K, V, KG, VG> {
    key_generator: KG,
    value_generator: VG,
    min_size: usize,
    max_size: usize,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V, KG, VG> BoundedMapGenerator<K, V, KG, VG>
where
    KG: Generator<K>,
    VG: Generator<V>,
{
    /// Create a new bounded map generator
    pub fn new(key_generator: KG, value_generator: VG, min_size: usize, max_size: usize) -> Self {
        Self {
            key_generator,
            value_generator,
            min_size,
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K, V, KG, VG> Generator<HashMap<K, V>> for BoundedMapGenerator<K, V, KG, VG>
where
    K: Clone + Eq + Hash + 'static,
    V: Clone + 'static,
    KG: Generator<K> + Clone + 'static,
    VG: Generator<V> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> HashMap<K, V> {
        use rand::Rng;
        let target_size = rng.r#gen_range(self.min_size..=self.max_size);
        let mut map = HashMap::new();

        // Try to generate unique keys
        let mut attempts = 0;
        let max_attempts = target_size * 10;

        while map.len() < target_size && attempts < max_attempts {
            let key = self.key_generator.generate(rng, config);
            let value = self.value_generator.generate(rng, config);
            map.insert(key, value);
            attempts += 1;
        }

        map
    }

    fn shrink(&self, value: &HashMap<K, V>) -> Box<dyn Iterator<Item = HashMap<K, V>>> {
        let mut shrinks = Vec::new();

        // Try empty if allowed
        if !value.is_empty() && self.min_size == 0 {
            shrinks.push(HashMap::new());
        }

        // Try removing entries
        if value.len() > self.min_size {
            let keys: Vec<_> = value.keys().cloned().collect();
            for key in keys.iter().take(3) {
                let mut shrunk = value.clone();
                shrunk.remove(key);
                if shrunk.len() >= self.min_size {
                    shrinks.push(shrunk);
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protest::IntGenerator;
    use rand::thread_rng;

    #[test]
    fn test_non_empty_vec_generator() {
        let gen = NonEmptyVecGenerator::new(IntGenerator::new(1, 100), 1, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let vec = gen.generate(&mut rng, &config);
            assert!(!vec.is_empty(), "Vec should not be empty");
            assert!(vec.len() >= 1 && vec.len() <= 10);
            for &elem in &vec {
                assert!(elem >= 1 && elem <= 100);
            }
        }
    }

    #[test]
    fn test_non_empty_vec_min_size() {
        // Even with min_len = 0, should have at least 1 element
        let gen = NonEmptyVecGenerator::new(IntGenerator::new(1, 100), 0, 5);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let vec = gen.generate(&mut rng, &config);
            assert!(!vec.is_empty());
        }
    }

    #[test]
    fn test_sorted_vec_generator() {
        let gen = SortedVecGenerator::new(IntGenerator::new(1, 100), 3, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let vec = gen.generate(&mut rng, &config);
            assert!(vec.len() >= 3 && vec.len() <= 10);

            // Check if sorted
            for window in vec.windows(2) {
                assert!(window[0] <= window[1], "Vec should be sorted: {:?}", vec);
            }
        }
    }

    #[test]
    fn test_unique_vec_generator() {
        let gen = UniqueVecGenerator::new(IntGenerator::new(1, 50), 3, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let vec = gen.generate(&mut rng, &config);
            assert!(vec.len() >= 3 && vec.len() <= 10);

            // Check for uniqueness
            let mut seen = std::collections::HashSet::new();
            for elem in &vec {
                assert!(seen.insert(*elem), "Duplicate found: {} in {:?}", elem, vec);
            }
        }
    }

    #[test]
    fn test_bounded_map_generator() {
        let gen =
            BoundedMapGenerator::new(IntGenerator::new(1, 100), IntGenerator::new(1, 100), 2, 8);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let map = gen.generate(&mut rng, &config);
            assert!(map.len() >= 2 && map.len() <= 8);

            for (key, value) in &map {
                assert!(*key >= 1 && *key <= 100);
                assert!(*value >= 1 && *value <= 100);
            }
        }
    }
}
