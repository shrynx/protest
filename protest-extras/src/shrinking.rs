//! Enhanced shrinking strategies for Protest
//!
//! This module provides advanced shrinking strategies that go beyond basic shrinking:
//! - **Smart Shrinking**: Preserve invariants while shrinking
//! - **Delta Debugging**: Binary search for minimal failing subsets
//! - **Targeted Shrinking**: Shrink toward specific target values
//!
//! # Examples
//!
//! ## Smart Shrinking with Invariants
//!
//! ```rust
//! use protest_extras::shrinking::SmartShrink;
//!
//! let mut sorted_vec = vec![1, 3, 5, 7, 9];
//!
//! // Shrink while keeping the vector sorted
//! let shrunk = sorted_vec.shrink_preserving(|v| {
//!     v.windows(2).all(|w| w[0] <= w[1])
//! });
//!
//! for candidate in shrunk {
//!     // All candidates remain sorted
//!     assert!(candidate.windows(2).all(|w| w[0] <= w[1]));
//! }
//! ```
//!
//! ## Delta Debugging
//!
//! ```rust
//! use protest_extras::shrinking::DeltaDebugShrinker;
//!
//! let operations = vec![1, 2, 3, 4, 5, 6, 7, 8];
//!
//! // Find minimal subset that fails a test
//! let shrinker = DeltaDebugShrinker::new(operations);
//! let minimal = shrinker.find_minimal(|ops| {
//!     // Only fails if contains both 3 and 7
//!     ops.contains(&3) && ops.contains(&7)
//! });
//!
//! assert_eq!(minimal, vec![3, 7]); // Minimal failing case
//! ```

use std::fmt::Debug;

// ============================================================================
// Smart Shrinking with Invariants
// ============================================================================

/// Trait for shrinking values while preserving invariants
///
/// This allows you to shrink a value while ensuring that certain properties
/// (invariants) are maintained throughout the shrinking process.
pub trait SmartShrink: Clone {
    /// Shrink this value while preserving the given invariant
    ///
    /// The invariant function should return `true` if the value is valid.
    /// Only shrunk values that satisfy the invariant will be yielded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use protest_extras::shrinking::SmartShrink;
    ///
    /// let mut vec = vec![2, 4, 6, 8];
    ///
    /// // Shrink while keeping all elements even
    /// let shrunk: Vec<_> = vec.shrink_preserving(|v| {
    ///     v.iter().all(|x| x % 2 == 0)
    /// }).collect();
    ///
    /// // All shrunk candidates have even elements
    /// for candidate in shrunk {
    ///     assert!(candidate.iter().all(|x| x % 2 == 0));
    /// }
    /// ```
    fn shrink_preserving<F>(&self, invariant: F) -> Box<dyn Iterator<Item = Self>>
    where
        F: Fn(&Self) -> bool + 'static;
}

// Implement SmartShrink for Vec
impl<T: Clone + Debug + 'static> SmartShrink for Vec<T> {
    fn shrink_preserving<F>(&self, invariant: F) -> Box<dyn Iterator<Item = Self>>
    where
        F: Fn(&Self) -> bool + 'static,
    {
        let original = self.clone();
        let invariant = std::rc::Rc::new(invariant);

        let mut candidates = Vec::new();

        // Try empty vector
        if !original.is_empty() {
            let v = Vec::new();
            if invariant(&v) {
                candidates.push(v);
            }
        }

        // Try removing elements from different positions
        for i in 0..original.len().min(3) {
            if original.len() > 1 {
                let mut v = original.clone();
                v.remove(i);
                if invariant(&v) {
                    candidates.push(v);
                }
            }
        }

        // Try half the length
        if original.len() > 1 {
            let v = original[..original.len() / 2].to_vec();
            if invariant(&v) {
                candidates.push(v);
            }
        }

        Box::new(candidates.into_iter())
    }
}

// Implement SmartShrink for integers
impl SmartShrink for i32 {
    fn shrink_preserving<F>(&self, invariant: F) -> Box<dyn Iterator<Item = Self>>
    where
        F: Fn(&Self) -> bool + 'static,
    {
        let current = *self;
        let invariant = std::rc::Rc::new(invariant);

        let candidates: Vec<i32> = (0..current)
            .rev()
            .filter(|x| invariant(x))
            .take(10)
            .collect();

        Box::new(candidates.into_iter())
    }
}

impl SmartShrink for u32 {
    fn shrink_preserving<F>(&self, invariant: F) -> Box<dyn Iterator<Item = Self>>
    where
        F: Fn(&Self) -> bool + 'static,
    {
        let current = *self;
        let invariant = std::rc::Rc::new(invariant);

        let candidates: Vec<u32> = (0..current)
            .rev()
            .filter(|x| invariant(x))
            .take(10)
            .collect();

        Box::new(candidates.into_iter())
    }
}

// ============================================================================
// Delta Debugging Shrinker
// ============================================================================

/// Delta debugging shrinker for finding minimal failing subsets
///
/// Uses binary search to efficiently find the smallest subset of a collection
/// that still fails a test. This is particularly useful for reducing sequences
/// of operations or large inputs to their minimal failing case.
///
/// # Examples
///
/// ```rust
/// use protest_extras::shrinking::DeltaDebugShrinker;
///
/// let items = vec!["a", "b", "c", "d", "e"];
/// let shrinker = DeltaDebugShrinker::new(items);
///
/// // Find minimal subset that contains "b" and "d"
/// let minimal = shrinker.find_minimal(|subset| {
///     subset.contains(&"b") && subset.contains(&"d")
/// });
///
/// assert_eq!(minimal, vec!["b", "d"]);
/// ```
#[derive(Debug, Clone)]
pub struct DeltaDebugShrinker<T> {
    items: Vec<T>,
}

impl<T: Clone + Debug + PartialEq> DeltaDebugShrinker<T> {
    /// Create a new delta debugging shrinker
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    /// Find the minimal subset that satisfies the predicate
    ///
    /// The predicate should return `true` for failing test cases.
    pub fn find_minimal<F>(self, predicate: F) -> Vec<T>
    where
        F: Fn(&[T]) -> bool,
    {
        let mut current = self.items;

        // If the full set doesn't fail, return it
        if !predicate(&current) {
            return current;
        }

        let mut changed = true;

        while changed {
            changed = false;
            let n = current.len();

            if n <= 1 {
                break;
            }

            // Try removing chunks of size n/2
            let chunk_size = n / 2;

            for i in 0..=(n - chunk_size) {
                let candidate: Vec<T> = current
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx < i || *idx >= i + chunk_size)
                    .map(|(_, item)| item.clone())
                    .collect();

                if !candidate.is_empty() && predicate(&candidate) {
                    current = candidate;
                    changed = true;
                    break;
                }
            }

            // If no chunk removal worked, try removing individual elements
            if !changed {
                for i in 0..current.len() {
                    let mut candidate = current.clone();
                    candidate.remove(i);

                    if !candidate.is_empty() && predicate(&candidate) {
                        current = candidate;
                        changed = true;
                        break;
                    }
                }
            }
        }

        current
    }

    /// Shrink to minimal subset, yielding intermediate results
    ///
    /// This is useful if you want to see the shrinking process or
    /// stop early.
    pub fn shrink_steps<F>(self, predicate: F) -> DeltaDebugIterator<T, F>
    where
        F: Fn(&[T]) -> bool,
    {
        DeltaDebugIterator {
            current: self.items,
            predicate,
            phase: 0,
            chunk_size: 0,
            index: 0,
        }
    }
}

/// Iterator that yields shrinking steps
pub struct DeltaDebugIterator<T, F>
where
    F: Fn(&[T]) -> bool,
{
    current: Vec<T>,
    predicate: F,
    phase: usize,
    chunk_size: usize,
    index: usize,
}

impl<T: Clone + Debug, F: Fn(&[T]) -> bool> Iterator for DeltaDebugIterator<T, F> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.len() <= 1 {
            return None;
        }

        loop {
            match self.phase {
                0 => {
                    // Initialize chunk size
                    self.chunk_size = self.current.len() / 2;
                    self.index = 0;
                    self.phase = 1;
                }
                1 => {
                    // Try removing chunks
                    if self.index + self.chunk_size <= self.current.len() {
                        let candidate: Vec<T> = self
                            .current
                            .iter()
                            .enumerate()
                            .filter(|(idx, _)| {
                                *idx < self.index || *idx >= self.index + self.chunk_size
                            })
                            .map(|(_, item)| item.clone())
                            .collect();

                        self.index += 1;

                        if !candidate.is_empty() && (self.predicate)(&candidate) {
                            self.current = candidate.clone();
                            self.phase = 0; // Restart with new current
                            return Some(candidate);
                        }
                    } else {
                        self.phase = 2;
                        self.index = 0;
                    }
                }
                2 => {
                    // Try removing individual elements
                    if self.index < self.current.len() {
                        let mut candidate = self.current.clone();
                        candidate.remove(self.index);
                        self.index += 1;

                        if !candidate.is_empty() && (self.predicate)(&candidate) {
                            self.current = candidate.clone();
                            self.phase = 0; // Restart
                            return Some(candidate);
                        }
                    } else {
                        return None; // No more shrinking possible
                    }
                }
                _ => return None,
            }
        }
    }
}

// ============================================================================
// Targeted Shrinking
// ============================================================================

/// Shrinker that shrinks toward a specific target value
///
/// Instead of shrinking toward a "simple" value (like 0 or empty),
/// this shrinker tries to shrink toward a specific target value.
///
/// # Examples
///
/// ```rust
/// use protest_extras::shrinking::TargetedShrinker;
///
/// // Shrink toward 42
/// let shrinker = TargetedShrinker::new_int(100, 42);
///
/// let shrunk: Vec<_> = shrinker.shrink().take(5).collect();
/// // Yields values getting closer to 42
/// assert!(shrunk.iter().all(|&x| x >= 42 && x <= 100));
/// ```
#[derive(Debug, Clone)]
pub struct TargetedShrinker<T> {
    current: T,
    target: T,
}

impl TargetedShrinker<i32> {
    /// Create a new targeted shrinker for integers
    pub fn new_int(current: i32, target: i32) -> Self {
        Self { current, target }
    }

    /// Generate shrinking steps toward the target
    pub fn shrink(self) -> TargetedShrinkIterator<i32> {
        TargetedShrinkIterator {
            current: self.current,
            target: self.target,
            step: 0,
        }
    }
}

impl TargetedShrinker<f64> {
    /// Create a new targeted shrinker for floats
    pub fn new_float(current: f64, target: f64) -> Self {
        Self { current, target }
    }

    /// Generate shrinking steps toward the target
    pub fn shrink(self) -> TargetedShrinkIterator<f64> {
        TargetedShrinkIterator {
            current: self.current,
            target: self.target,
            step: 0,
        }
    }
}

/// Iterator for targeted shrinking
pub struct TargetedShrinkIterator<T> {
    current: T,
    target: T,
    step: usize,
}

impl Iterator for TargetedShrinkIterator<i32> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.target {
            return None;
        }

        self.step += 1;

        // Binary search toward target
        let diff = self.current - self.target;
        let step_size = (diff.abs() / 2).max(1);

        self.current = if diff > 0 {
            (self.current - step_size).max(self.target)
        } else {
            (self.current + step_size).min(self.target)
        };

        Some(self.current)
    }
}

impl Iterator for TargetedShrinkIterator<f64> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.current - self.target).abs() < f64::EPSILON {
            return None;
        }

        self.step += 1;

        // Binary search toward target
        let diff = self.current - self.target;
        self.current = self.target + diff / 2.0;

        Some(self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_shrink_vec_preserving_sorted() {
        let vec = vec![1, 3, 5, 7, 9];

        let shrunk: Vec<_> = vec
            .shrink_preserving(|v| v.windows(2).all(|w| w[0] <= w[1]))
            .collect();

        // All shrunk vectors should be sorted
        for candidate in shrunk {
            assert!(candidate.windows(2).all(|w| w[0] <= w[1]));
        }
    }

    #[test]
    fn test_smart_shrink_vec_preserving_even() {
        let vec = vec![2, 4, 6, 8];

        let shrunk: Vec<_> = vec
            .shrink_preserving(|v| v.iter().all(|x| x % 2 == 0))
            .collect();

        // All shrunk vectors should have even elements
        for candidate in shrunk {
            assert!(candidate.iter().all(|x| x % 2 == 0));
        }
    }

    #[test]
    fn test_smart_shrink_int_preserving_positive() {
        let num = 100;

        let shrunk: Vec<_> = num.shrink_preserving(|x| *x > 0).take(10).collect();

        // All shrunk values should be positive
        for candidate in shrunk {
            assert!(candidate > 0);
        }
    }

    #[test]
    fn test_delta_debug_minimal_subset() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let shrinker = DeltaDebugShrinker::new(items);

        // Find minimal subset containing 3 and 7
        let minimal = shrinker.find_minimal(|subset| subset.contains(&3) && subset.contains(&7));

        assert!(minimal.contains(&3));
        assert!(minimal.contains(&7));
        assert!(minimal.len() <= 2);
    }

    #[test]
    fn test_delta_debug_single_element() {
        let items = vec![1, 2, 3, 4, 5];
        let shrinker = DeltaDebugShrinker::new(items);

        // Find minimal subset containing 3
        let minimal = shrinker.find_minimal(|subset| subset.contains(&3));

        assert_eq!(minimal, vec![3]);
    }

    #[test]
    fn test_delta_debug_iterator() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let shrinker = DeltaDebugShrinker::new(items);

        let steps: Vec<_> = shrinker
            .shrink_steps(|subset| subset.contains(&2) && subset.contains(&5))
            .collect();

        // Should yield progressively smaller subsets
        assert!(!steps.is_empty());
        let last = steps.last().unwrap();
        assert!(last.contains(&2));
        assert!(last.contains(&5));
    }

    #[test]
    fn test_targeted_shrink_int_toward_zero() {
        let shrinker = TargetedShrinker::new_int(100, 0);
        let shrunk: Vec<_> = shrinker.shrink().collect();

        assert!(!shrunk.is_empty());
        assert_eq!(*shrunk.last().unwrap(), 0);

        // Should be monotonically decreasing
        for window in shrunk.windows(2) {
            assert!(window[0] > window[1]);
        }
    }

    #[test]
    fn test_targeted_shrink_int_toward_target() {
        let shrinker = TargetedShrinker::new_int(100, 42);
        let shrunk: Vec<_> = shrinker.shrink().collect();

        assert!(!shrunk.is_empty());
        assert_eq!(*shrunk.last().unwrap(), 42);

        // Should be getting closer to 42
        for value in shrunk {
            assert!((42..=100).contains(&value));
        }
    }

    #[test]
    fn test_targeted_shrink_float() {
        let shrinker = TargetedShrinker::new_float(100.0, 42.0);
        let shrunk: Vec<_> = shrinker.shrink().take(10).collect();

        assert!(!shrunk.is_empty());

        // Should be getting closer to 42.0
        let mut prev = 100.0_f64;
        for value in shrunk {
            let prev_diff = (prev - 42.0_f64).abs();
            let curr_diff = (value - 42.0_f64).abs();
            assert!(curr_diff < prev_diff);
            prev = value;
        }
    }
}
