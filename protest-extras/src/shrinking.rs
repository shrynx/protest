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

// ============================================================================
// Cascading Shrinking Strategy
// ============================================================================

/// Cascading shrinker that applies multiple shrinking strategies in sequence
///
/// This strategy tries multiple shrinking approaches and combines their results,
/// allowing for more thorough exploration of the shrink space.
///
/// # Examples
///
/// ```rust
/// use protest_extras::shrinking::CascadingShrinker;
///
/// let value = vec![10, 20, 30, 40, 50];
///
/// // Try removing elements, then halving, then targeted shrinking
/// let shrinker = CascadingShrinker::new(value);
/// let shrunk: Vec<_> = shrinker.shrink().take(20).collect();
///
/// // Will explore many different shrinking paths
/// assert!(!shrunk.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct CascadingShrinker<T> {
    value: T,
}

impl<T: Clone> CascadingShrinker<T> {
    /// Create a new cascading shrinker
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Get the original value
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T: Clone + Debug + 'static> CascadingShrinker<Vec<T>> {
    /// Shrink using multiple strategies
    pub fn shrink(&self) -> Box<dyn Iterator<Item = Vec<T>>> {
        let original = self.value.clone();
        let mut candidates = Vec::new();

        // Strategy 1: Remove single elements
        for i in 0..original.len() {
            let mut v = original.clone();
            v.remove(i);
            candidates.push(v);
        }

        // Strategy 2: Remove chunks (halves, thirds, quarters)
        if original.len() > 1 {
            // Remove first half
            candidates.push(original[original.len() / 2..].to_vec());
            // Remove second half
            candidates.push(original[..original.len() / 2].to_vec());

            // Remove first third
            if original.len() >= 3 {
                candidates.push(original[original.len() / 3..].to_vec());
            }

            // Remove middle third
            if original.len() >= 3 {
                let third = original.len() / 3;
                let mut v = original[..third].to_vec();
                v.extend_from_slice(&original[2 * third..]);
                candidates.push(v);
            }
        }

        // Strategy 3: Keep only sorted subsequences
        if original.len() > 2 {
            for i in 0..original.len() - 1 {
                candidates.push(original[i..i + 2].to_vec());
            }
        }

        // Strategy 4: Empty vector
        candidates.push(Vec::new());

        Box::new(candidates.into_iter())
    }
}

impl CascadingShrinker<i32> {
    /// Shrink using multiple strategies for integers
    pub fn shrink(&self) -> Box<dyn Iterator<Item = i32>> {
        let current = self.value;
        let mut candidates = Vec::new();

        if current == 0 {
            return Box::new(std::iter::empty());
        }

        // Strategy 1: Move toward zero
        let half = current / 2;
        if half != current {
            candidates.push(half);
        }

        // Strategy 2: Subtract powers of 2
        for i in 0..10 {
            let pow = 1 << i;
            if pow < current.abs() {
                let next = if current > 0 {
                    current - pow
                } else {
                    current + pow
                };
                if next != current {
                    candidates.push(next);
                }
            }
        }

        // Strategy 3: Zero
        candidates.push(0);

        // Deduplicate
        candidates.sort_unstable();
        candidates.dedup();

        Box::new(candidates.into_iter())
    }
}

// ============================================================================
// Guided Shrinking with Test Feedback
// ============================================================================

/// Guided shrinker that uses test feedback to prioritize shrinking directions
///
/// This shrinker runs the test on shrink candidates and uses pass/fail information
/// to guide the search toward minimal failing cases.
///
/// # Examples
///
/// ```rust
/// use protest_extras::shrinking::GuidedShrinker;
///
/// let value = vec![1, 2, 3, 4, 5, 6, 7, 8];
///
/// // Find minimal subset that sums to > 10
/// let shrinker = GuidedShrinker::new(value);
/// let minimal = shrinker.find_minimal(|v| v.iter().sum::<i32>() > 10);
///
/// // Will find a small subset that still sums to > 10
/// assert!(minimal.iter().sum::<i32>() > 10);
/// assert!(minimal.len() < 8);
/// ```
#[derive(Debug, Clone)]
pub struct GuidedShrinker<T> {
    value: T,
    max_iterations: usize,
}

impl<T: Clone> GuidedShrinker<T> {
    /// Create a new guided shrinker with default max iterations
    pub fn new(value: T) -> Self {
        Self {
            value,
            max_iterations: 1000,
        }
    }

    /// Create a guided shrinker with custom max iterations
    pub fn with_max_iterations(value: T, max_iterations: usize) -> Self {
        Self {
            value,
            max_iterations,
        }
    }
}

impl<T: Clone + Debug + PartialEq + 'static> GuidedShrinker<Vec<T>> {
    /// Find minimal failing case using test feedback
    ///
    /// The test function should return `true` if the test fails (i.e., we want to keep shrinking).
    /// Returns the smallest value that still fails the test.
    pub fn find_minimal<F>(&self, test: F) -> Vec<T>
    where
        F: Fn(&Vec<T>) -> bool,
    {
        let mut current = self.value.clone();

        if !test(&current) {
            // Original doesn't fail, return as-is
            return current;
        }

        for _ in 0..self.max_iterations {
            let mut found_smaller = false;

            // Try removing each element
            for i in (0..current.len()).rev() {
                let mut candidate = current.clone();
                candidate.remove(i);

                if test(&candidate) {
                    // Still fails with this element removed
                    current = candidate;
                    found_smaller = true;
                    break;
                }
            }

            if !found_smaller {
                // Try removing halves
                if current.len() > 1 {
                    let half = current.len() / 2;

                    // Try first half
                    let first_half = current[..half].to_vec();
                    if test(&first_half) {
                        current = first_half;
                        found_smaller = true;
                    } else {
                        // Try second half
                        let second_half = current[half..].to_vec();
                        if test(&second_half) {
                            current = second_half;
                            found_smaller = true;
                        }
                    }
                }
            }

            if !found_smaller {
                // Can't shrink further
                break;
            }
        }

        current
    }

    /// Find minimal failing case with detailed feedback
    ///
    /// Returns both the minimal value and the number of iterations used.
    pub fn find_minimal_with_stats<F>(&self, test: F) -> (Vec<T>, usize)
    where
        F: Fn(&Vec<T>) -> bool,
    {
        let mut current = self.value.clone();
        let mut iterations = 0;

        if !test(&current) {
            return (current, iterations);
        }

        for _ in 0..self.max_iterations {
            iterations += 1;
            let mut found_smaller = false;

            for i in (0..current.len()).rev() {
                let mut candidate = current.clone();
                candidate.remove(i);

                if test(&candidate) {
                    current = candidate;
                    found_smaller = true;
                    break;
                }
            }

            if !found_smaller && current.len() > 1 {
                let half = current.len() / 2;

                let first_half = current[..half].to_vec();
                if test(&first_half) {
                    current = first_half;
                    found_smaller = true;
                } else {
                    let second_half = current[half..].to_vec();
                    if test(&second_half) {
                        current = second_half;
                        found_smaller = true;
                    }
                }
            }

            if !found_smaller {
                break;
            }
        }

        (current, iterations)
    }
}

// ============================================================================
// Breadth-First vs Depth-First Shrinking Control
// ============================================================================

/// Search strategy for shrinking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShrinkStrategy {
    /// Breadth-first: Try all shrinks at current depth before going deeper
    BreadthFirst,
    /// Depth-first: Follow first successful shrink as far as possible
    DepthFirst,
}

/// Configurable shrinker with search strategy control
///
/// Allows choosing between breadth-first and depth-first search strategies
/// when exploring the shrink space.
///
/// # Examples
///
/// ```rust
/// use protest_extras::shrinking::{ConfigurableShrinker, ShrinkStrategy};
///
/// let value = vec![1, 2, 3, 4, 5];
///
/// // Depth-first: aggressive shrinking
/// let shrinker = ConfigurableShrinker::new(value.clone(), ShrinkStrategy::DepthFirst);
/// let depth_result = shrinker.find_minimal(|v| v.len() >= 2);
///
/// // Breadth-first: explore more evenly
/// let shrinker = ConfigurableShrinker::new(value, ShrinkStrategy::BreadthFirst);
/// let breadth_result = shrinker.find_minimal(|v| v.len() >= 2);
///
/// assert!(depth_result.len() >= 2);
/// assert!(breadth_result.len() >= 2);
/// ```
#[derive(Debug, Clone)]
pub struct ConfigurableShrinker<T> {
    value: T,
    strategy: ShrinkStrategy,
    max_depth: usize,
}

impl<T: Clone> ConfigurableShrinker<T> {
    /// Create a new configurable shrinker
    pub fn new(value: T, strategy: ShrinkStrategy) -> Self {
        Self {
            value,
            strategy,
            max_depth: 100,
        }
    }

    /// Set maximum search depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Get the current strategy
    pub fn strategy(&self) -> ShrinkStrategy {
        self.strategy
    }
}

impl<T: Clone + Debug + PartialEq + 'static> ConfigurableShrinker<Vec<T>> {
    /// Find minimal case using the configured strategy
    pub fn find_minimal<F>(&self, test: F) -> Vec<T>
    where
        F: Fn(&Vec<T>) -> bool,
    {
        match self.strategy {
            ShrinkStrategy::DepthFirst => self.find_minimal_dfs(&test, self.value.clone(), 0),
            ShrinkStrategy::BreadthFirst => self.find_minimal_bfs(&test),
        }
    }

    fn find_minimal_dfs<F>(&self, test: &F, current: Vec<T>, depth: usize) -> Vec<T>
    where
        F: Fn(&Vec<T>) -> bool,
    {
        if depth >= self.max_depth || !test(&current) {
            return current;
        }

        // Try removing elements one by one (depth-first)
        for i in (0..current.len()).rev() {
            let mut candidate = current.clone();
            candidate.remove(i);

            if test(&candidate) {
                // Recursively shrink this candidate
                return self.find_minimal_dfs(test, candidate, depth + 1);
            }
        }

        // Try halving
        if current.len() > 1 {
            let half = current.len() / 2;

            let first_half = current[..half].to_vec();
            if test(&first_half) {
                return self.find_minimal_dfs(test, first_half, depth + 1);
            }

            let second_half = current[half..].to_vec();
            if test(&second_half) {
                return self.find_minimal_dfs(test, second_half, depth + 1);
            }
        }

        current
    }

    fn find_minimal_bfs<F>(&self, test: &F) -> Vec<T>
    where
        F: Fn(&Vec<T>) -> bool,
    {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        queue.push_back((self.value.clone(), 0));

        let mut best = self.value.clone();
        let mut visited = std::collections::HashSet::new();

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= self.max_depth {
                continue;
            }

            if !test(&current) {
                continue;
            }

            // This is a valid failing case
            if current.len() < best.len() {
                best = current.clone();
            }

            // Generate candidates for next level
            for i in 0..current.len() {
                let mut candidate = current.clone();
                candidate.remove(i);

                // Use a simple hash to avoid revisiting
                let hash = format!("{:?}", candidate);
                if visited.insert(hash) {
                    queue.push_back((candidate, depth + 1));
                }
            }

            // Add halving candidates
            if current.len() > 1 {
                let half = current.len() / 2;

                let first_half = current[..half].to_vec();
                let hash = format!("{:?}", first_half);
                if visited.insert(hash.clone()) {
                    queue.push_back((first_half, depth + 1));
                }

                let second_half = current[half..].to_vec();
                let hash = format!("{:?}", second_half);
                if visited.insert(hash) {
                    queue.push_back((second_half, depth + 1));
                }
            }
        }

        best
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

    #[test]
    fn test_cascading_shrinker_vec() {
        let value = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let shrinker = CascadingShrinker::new(value);

        let shrunk: Vec<_> = shrinker.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should include various shrinking strategies
        // - Single element removals
        // - Halves
        // - Empty vec
        assert!(shrunk.contains(&vec![]));

        // Check that some element removals are present
        assert!(shrunk.iter().any(|v| v.len() == 7));
    }

    #[test]
    fn test_cascading_shrinker_int() {
        let shrinker = CascadingShrinker::new(100);
        let shrunk: Vec<_> = shrinker.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should include zero
        assert!(shrunk.contains(&0));

        // Should include half
        assert!(shrunk.contains(&50));

        // All values should be smaller than original
        for value in &shrunk {
            assert!(value.abs() <= 100);
        }
    }

    #[test]
    fn test_guided_shrinker_finds_minimal() {
        let value = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let shrinker = GuidedShrinker::new(value);

        // Find minimal subset that sums to > 15
        let minimal = shrinker.find_minimal(|v| v.iter().sum::<i32>() > 15);

        // Should find something smaller than original
        assert!(minimal.len() < 10);

        // Should still sum to > 15
        assert!(minimal.iter().sum::<i32>() > 15);

        // Should be reasonably small (e.g., sum of 16 would be minimal)
        assert!(minimal.len() <= 6); // At most [10, 6] or similar
    }

    #[test]
    fn test_guided_shrinker_with_stats() {
        let value = vec![1, 2, 3, 4, 5];
        let shrinker = GuidedShrinker::new(value);

        let (minimal, iterations) = shrinker.find_minimal_with_stats(|v| v.len() >= 2);

        // Should find something of length 2
        assert_eq!(minimal.len(), 2);

        // Should have taken some iterations
        assert!(iterations > 0);
        assert!(iterations < 100); // Reasonable bound
    }

    #[test]
    fn test_guided_shrinker_no_shrink_if_passes() {
        let value = vec![1, 2, 3];
        let shrinker = GuidedShrinker::new(value.clone());

        // Test always fails (returns false), so no shrinking
        let minimal = shrinker.find_minimal(|_v| false);

        // Should return original since test never fails
        assert_eq!(minimal, value);
    }

    #[test]
    fn test_configurable_shrinker_depth_first() {
        let value = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let shrinker = ConfigurableShrinker::new(value, ShrinkStrategy::DepthFirst);

        let minimal = shrinker.find_minimal(|v| v.len() >= 3);

        // Should find something of length 3
        assert_eq!(minimal.len(), 3);
    }

    #[test]
    fn test_configurable_shrinker_breadth_first() {
        let value = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let shrinker = ConfigurableShrinker::new(value, ShrinkStrategy::BreadthFirst);

        let minimal = shrinker.find_minimal(|v| v.len() >= 3);

        // Should find something of length 3
        assert_eq!(minimal.len(), 3);
    }

    #[test]
    fn test_configurable_shrinker_with_max_depth() {
        let value = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let shrinker =
            ConfigurableShrinker::new(value, ShrinkStrategy::DepthFirst).with_max_depth(5);

        let minimal = shrinker.find_minimal(|v| v.len() >= 2);

        // With max_depth=5, DFS can remove at most 5 elements: 10 - 5 = 5
        // This demonstrates that max_depth limits how many steps we can take
        assert_eq!(minimal.len(), 5);
    }

    #[test]
    fn test_shrink_strategy_enum() {
        // Test enum properties
        assert_eq!(ShrinkStrategy::DepthFirst, ShrinkStrategy::DepthFirst);
        assert_eq!(ShrinkStrategy::BreadthFirst, ShrinkStrategy::BreadthFirst);
        assert_ne!(ShrinkStrategy::DepthFirst, ShrinkStrategy::BreadthFirst);
    }

    #[test]
    fn test_cascading_shrinker_aggressive() {
        let value = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let shrinker = CascadingShrinker::new(value);

        let shrunk: Vec<_> = shrinker.shrink().collect();

        // Should produce many candidates
        assert!(shrunk.len() > 15);

        // Should include thirds (remove first third)
        assert!(shrunk.iter().any(|v| v.len() == 8));

        // Should include adjacent pairs
        assert!(shrunk.iter().any(|v| v.len() == 2));
    }
}
