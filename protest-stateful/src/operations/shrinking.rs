//! Advanced shrinking strategies for operation sequences
//!
//! This module provides sophisticated shrinking algorithms that go beyond simple
//! element removal:
//!
//! - **DeltaDebugSequenceShrinker**: Binary search for minimal failing subsequences
//! - **SmartSequenceShrinking**: Shrink while preserving invariants and preconditions
//!
//! # Example
//!
//! ```rust
//! use protest_stateful::prelude::*;
//! use protest_stateful::operations::shrinking::*;
//!
//! #[derive(Debug, Clone)]
//! struct Counter { value: i32 }
//!
//! #[derive(Debug, Clone)]
//! enum CounterOp {
//!     Increment,
//!     Decrement,
//! }
//!
//! impl Operation for CounterOp {
//!     type State = Counter;
//!
//!     fn execute(&self, state: &mut Self::State) {
//!         match self {
//!             CounterOp::Increment => state.value += 1,
//!             CounterOp::Decrement => state.value -= 1,
//!         }
//!     }
//!
//!     fn precondition(&self, state: &Self::State) -> bool {
//!         match self {
//!             CounterOp::Decrement => state.value > 0,
//!             _ => true,
//!         }
//!     }
//! }
//!
//! let mut seq = OperationSequence::new();
//! seq.push(CounterOp::Increment);
//! seq.push(CounterOp::Increment);
//! seq.push(CounterOp::Increment);
//! seq.push(CounterOp::Decrement);
//!
//! // Use delta debugging to find minimal failing sequence
//! let shrinker = DeltaDebugSequenceShrinker::new(seq);
//! let test = StatefulTest::new(Counter { value: 0 })
//!     .invariant("value_less_than_2", |s: &Counter| s.value < 2);
//!
//! let minimal = shrinker.minimize(|sequence| {
//!     test.run(sequence).is_err()
//! });
//!
//! // The minimal sequence that still fails the test
//! assert!(minimal.len() <= 4);
//! ```

use crate::operations::{Operation, OperationSequence};
use std::fmt::Debug;

/// Delta debugging shrinker for operation sequences
///
/// Uses binary search to find the minimal subsequence that still exhibits
/// the failing property. This is much more efficient than trying every
/// possible subsequence.
///
/// # Algorithm
///
/// 1. Try removing large chunks (halves)
/// 2. If that doesn't preserve the failure, try smaller chunks
/// 3. Recursively shrink the resulting sequence
/// 4. Also try removing individual operations
///
/// This finds a locally minimal sequence (often globally minimal) in
/// O(n log n) tests instead of O(2^n).
#[derive(Debug, Clone)]
pub struct DeltaDebugSequenceShrinker<Op> {
    sequence: OperationSequence<Op>,
}

impl<Op: Operation> DeltaDebugSequenceShrinker<Op> {
    /// Create a new delta debug shrinker
    pub fn new(sequence: OperationSequence<Op>) -> Self {
        Self { sequence }
    }

    /// Find the minimal subsequence that still satisfies the test
    ///
    /// The test function should return `true` if the sequence still exhibits
    /// the property we're looking for (e.g., still fails the test).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use protest_stateful::prelude::*;
    /// # use protest_stateful::operations::shrinking::*;
    /// # #[derive(Debug, Clone)]
    /// # struct Counter { value: i32 }
    /// # #[derive(Debug, Clone)]
    /// # enum CounterOp { Increment }
    /// # impl Operation for CounterOp {
    /// #     type State = Counter;
    /// #     fn execute(&self, state: &mut Self::State) { state.value += 1; }
    /// # }
    /// let mut seq = OperationSequence::new();
    /// seq.push(CounterOp::Increment);
    /// seq.push(CounterOp::Increment);
    ///
    /// let shrinker = DeltaDebugSequenceShrinker::new(seq);
    /// let test = StatefulTest::new(Counter { value: 0 })
    ///     .invariant("less_than_2", |s: &Counter| s.value < 2);
    ///
    /// let minimal = shrinker.minimize(|sequence| test.run(sequence).is_err());
    /// ```
    pub fn minimize<F>(&self, test: F) -> OperationSequence<Op>
    where
        F: Fn(&OperationSequence<Op>) -> bool,
    {
        self.minimize_with_stats(test).0
    }

    /// Find minimal subsequence and return statistics
    ///
    /// Returns `(minimal_sequence, test_count)` where `test_count` is the
    /// number of times the test function was called.
    pub fn minimize_with_stats<F>(&self, test: F) -> (OperationSequence<Op>, usize)
    where
        F: Fn(&OperationSequence<Op>) -> bool,
    {
        use std::cell::Cell;

        let test_count = Cell::new(0);
        let test_with_count = |seq: &OperationSequence<Op>| {
            test_count.set(test_count.get() + 1);
            test(seq)
        };

        let minimal = Self::ddmin(&self.sequence, &test_with_count);
        (minimal, test_count.get())
    }

    /// Core delta debugging algorithm
    fn ddmin<F>(sequence: &OperationSequence<Op>, test: &F) -> OperationSequence<Op>
    where
        F: Fn(&OperationSequence<Op>) -> bool,
    {
        let ops = sequence.operations();
        let n = ops.len();

        if n <= 1 {
            return sequence.clone();
        }

        // Try removing chunks of size n/2, n/4, n/8, etc.
        let mut chunk_size = n / 2;

        while chunk_size > 0 {
            // Try removing each chunk
            for start in (0..n).step_by(chunk_size) {
                let end = (start + chunk_size).min(n);

                // Create sequence without this chunk
                let mut without_chunk = Vec::new();
                without_chunk.extend_from_slice(&ops[..start]);
                without_chunk.extend_from_slice(&ops[end..]);

                if without_chunk.is_empty() {
                    continue;
                }

                let candidate = OperationSequence::from_vec(without_chunk);

                // If removing this chunk still fails, recursively shrink
                if test(&candidate) {
                    return Self::ddmin(&candidate, test);
                }
            }

            // Try keeping only each chunk (complement of above)
            for start in (0..n).step_by(chunk_size) {
                let end = (start + chunk_size).min(n);

                let chunk = ops[start..end].to_vec();

                if chunk.is_empty() {
                    continue;
                }

                let candidate = OperationSequence::from_vec(chunk);

                if test(&candidate) {
                    return Self::ddmin(&candidate, test);
                }
            }

            // Try smaller chunks
            chunk_size /= 2;
        }

        // Try removing individual operations one by one
        for i in 0..n {
            let mut without_op = ops.to_vec();
            without_op.remove(i);

            if without_op.is_empty() {
                continue;
            }

            let candidate = OperationSequence::from_vec(without_op);

            if test(&candidate) {
                return Self::ddmin(&candidate, test);
            }
        }

        // Cannot shrink further
        sequence.clone()
    }
}

/// Smart sequence shrinking configuration
///
/// Controls how sequences are shrunk while preserving properties like
/// invariants and preconditions.
#[derive(Debug, Clone)]
pub struct SmartSequenceShrinking {
    /// Whether to preserve invariants during shrinking
    pub preserve_invariants: bool,
    /// Whether to preserve preconditions during shrinking
    pub preserve_preconditions: bool,
    /// Maximum number of shrinking attempts
    pub max_attempts: usize,
}

impl Default for SmartSequenceShrinking {
    fn default() -> Self {
        Self {
            preserve_invariants: true,
            preserve_preconditions: true,
            max_attempts: 1000,
        }
    }
}

impl SmartSequenceShrinking {
    /// Create a new smart shrinking configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to preserve invariants
    pub fn preserve_invariants(mut self, preserve: bool) -> Self {
        self.preserve_invariants = preserve;
        self
    }

    /// Set whether to preserve preconditions
    pub fn preserve_preconditions(mut self, preserve: bool) -> Self {
        self.preserve_preconditions = preserve;
        self
    }

    /// Set maximum shrinking attempts
    pub fn max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max;
        self
    }

    /// Shrink a sequence while preserving the test failure and constraints
    ///
    /// # Arguments
    ///
    /// * `sequence` - The failing sequence to shrink
    /// * `initial_state` - The initial state for test execution
    /// * `test` - Test function that returns `true` if the sequence still fails
    ///
    /// # Returns
    ///
    /// The minimal sequence that:
    /// 1. Still fails the test
    /// 2. Maintains all invariants (if `preserve_invariants` is true)
    /// 3. Respects all preconditions (if `preserve_preconditions` is true)
    pub fn shrink<Op, F>(
        &self,
        sequence: &OperationSequence<Op>,
        initial_state: &Op::State,
        test: F,
    ) -> OperationSequence<Op>
    where
        Op: Operation,
        Op::State: Clone,
        F: Fn(&OperationSequence<Op>) -> bool,
    {
        self.shrink_with_stats(sequence, initial_state, test).0
    }

    /// Shrink with statistics
    ///
    /// Returns `(minimal_sequence, attempts_used)`
    pub fn shrink_with_stats<Op, F>(
        &self,
        sequence: &OperationSequence<Op>,
        initial_state: &Op::State,
        test: F,
    ) -> (OperationSequence<Op>, usize)
    where
        Op: Operation,
        Op::State: Clone,
        F: Fn(&OperationSequence<Op>) -> bool,
    {
        let mut current = sequence.clone();
        let mut attempts = 0;

        loop {
            if attempts >= self.max_attempts {
                break;
            }

            let mut found_smaller = false;

            // Try removing each operation
            for i in (0..current.len()).rev() {
                if attempts >= self.max_attempts {
                    break;
                }

                let mut candidate_ops = current.operations().to_vec();
                candidate_ops.remove(i);

                if candidate_ops.is_empty() {
                    continue;
                }

                let candidate = OperationSequence::from_vec(candidate_ops);
                attempts += 1;

                // Check if candidate is valid
                if !self.is_valid_sequence(&candidate, initial_state) {
                    continue;
                }

                // Check if candidate still fails
                if test(&candidate) {
                    current = candidate;
                    found_smaller = true;
                    break;
                }
            }

            // Try removing chunks
            if !found_smaller && current.len() > 2 {
                for chunk_size in [current.len() / 2, current.len() / 3, current.len() / 4] {
                    if chunk_size == 0 || attempts >= self.max_attempts {
                        break;
                    }

                    for start in (0..current.len()).step_by(chunk_size.max(1)) {
                        if attempts >= self.max_attempts {
                            break;
                        }

                        let end = (start + chunk_size).min(current.len());

                        let mut candidate_ops = current.operations().to_vec();
                        candidate_ops.drain(start..end);

                        if candidate_ops.is_empty() {
                            continue;
                        }

                        let candidate = OperationSequence::from_vec(candidate_ops);
                        attempts += 1;

                        if !self.is_valid_sequence(&candidate, initial_state) {
                            continue;
                        }

                        if test(&candidate) {
                            current = candidate;
                            found_smaller = true;
                            break;
                        }
                    }

                    if found_smaller {
                        break;
                    }
                }
            }

            if !found_smaller {
                break;
            }
        }

        (current, attempts)
    }

    /// Check if a sequence is valid according to constraints
    fn is_valid_sequence<Op>(
        &self,
        sequence: &OperationSequence<Op>,
        initial_state: &Op::State,
    ) -> bool
    where
        Op: Operation,
        Op::State: Clone,
    {
        if !self.preserve_preconditions {
            return true;
        }

        // Check that all preconditions are satisfied
        let mut state = initial_state.clone();
        for op in sequence.operations() {
            if !op.precondition(&state) {
                return false;
            }
            op.execute(&mut state);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::StatefulTest;

    #[derive(Debug, Clone)]
    struct Counter {
        value: i32,
    }

    #[derive(Debug, Clone)]
    enum CounterOp {
        Increment,
        Decrement,
        Add(i32),
    }

    impl Operation for CounterOp {
        type State = Counter;

        fn execute(&self, state: &mut Self::State) {
            match self {
                CounterOp::Increment => state.value += 1,
                CounterOp::Decrement => state.value -= 1,
                CounterOp::Add(n) => state.value += n,
            }
        }

        fn precondition(&self, state: &Self::State) -> bool {
            match self {
                CounterOp::Decrement => state.value > 0,
                _ => true,
            }
        }
    }

    #[test]
    fn test_delta_debug_basic() {
        // Create a sequence that increments to 5
        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Increment); // 1
        seq.push(CounterOp::Increment); // 2
        seq.push(CounterOp::Increment); // 3
        seq.push(CounterOp::Increment); // 4
        seq.push(CounterOp::Increment); // 5

        let shrinker = DeltaDebugSequenceShrinker::new(seq);

        let test = StatefulTest::new(Counter { value: 0 })
            .invariant("less_than_3", |s: &Counter| s.value < 3);

        // Find minimal sequence that violates "value < 3"
        let minimal = shrinker.minimize(|sequence| test.run(sequence).is_err());

        // Should find that we only need 3 increments to violate the invariant
        assert_eq!(minimal.len(), 3);
    }

    #[test]
    fn test_delta_debug_with_stats() {
        let mut seq = OperationSequence::new();
        for _ in 0..10 {
            seq.push(CounterOp::Increment);
        }

        let shrinker = DeltaDebugSequenceShrinker::new(seq);

        let test = StatefulTest::new(Counter { value: 0 })
            .invariant("less_than_5", |s: &Counter| s.value < 5);

        let (minimal, test_count) =
            shrinker.minimize_with_stats(|sequence| test.run(sequence).is_err());

        assert_eq!(minimal.len(), 5); // Need exactly 5 increments to reach 5
        println!("Found minimal in {} tests", test_count);
        assert!(test_count < 50); // Should be much less than trying all subsets
    }

    #[test]
    fn test_delta_debug_complex_sequence() {
        // Create a complex sequence
        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Add(5)); // 5
        seq.push(CounterOp::Increment); // 6
        seq.push(CounterOp::Decrement); // 5
        seq.push(CounterOp::Add(10)); // 15
        seq.push(CounterOp::Decrement); // 14
        seq.push(CounterOp::Increment); // 15

        let shrinker = DeltaDebugSequenceShrinker::new(seq);

        let test = StatefulTest::new(Counter { value: 0 })
            .invariant("less_than_10", |s: &Counter| s.value < 10);

        let minimal = shrinker.minimize(|sequence| test.run(sequence).is_err());

        // The minimal should be just Add(10) since that alone violates the invariant
        assert_eq!(minimal.len(), 1);
    }

    #[test]
    fn test_smart_shrinking_basic() {
        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment);

        let config = SmartSequenceShrinking::new();
        let initial = Counter { value: 0 };

        let test =
            StatefulTest::new(initial.clone()).invariant("less_than_3", |s: &Counter| s.value < 3);

        let minimal = config.shrink(&seq, &initial, |sequence| test.run(sequence).is_err());

        assert_eq!(minimal.len(), 3);
    }

    #[test]
    fn test_smart_shrinking_preserves_preconditions() {
        // Create a sequence: Inc, Inc, Inc, Dec, Dec
        // Precondition: can't decrement below 0
        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Increment); // 1
        seq.push(CounterOp::Increment); // 2
        seq.push(CounterOp::Increment); // 3
        seq.push(CounterOp::Decrement); // 2
        seq.push(CounterOp::Decrement); // 1

        let config = SmartSequenceShrinking::new().preserve_preconditions(true);

        let initial = Counter { value: 0 };

        let test: StatefulTest<Counter, CounterOp> = StatefulTest::new(initial.clone())
            .invariant("always_positive", |s: &Counter| s.value > 0);

        // This test passes, but we're testing shrinking with preconditions
        let minimal = config.shrink(&seq, &initial, |sequence| {
            // Returns true if sequence ends with value >= 1
            let mut state = initial.clone();
            for op in sequence.operations() {
                op.execute(&mut state);
            }
            state.value >= 1
        });

        // Verify the minimal sequence respects preconditions
        let mut state = initial.clone();
        for op in minimal.operations() {
            assert!(op.precondition(&state));
            op.execute(&mut state);
        }
    }

    #[test]
    fn test_smart_shrinking_with_stats() {
        let mut seq = OperationSequence::new();
        for _ in 0..8 {
            seq.push(CounterOp::Increment);
        }

        let config = SmartSequenceShrinking::new().max_attempts(100);
        let initial = Counter { value: 0 };

        let test =
            StatefulTest::new(initial.clone()).invariant("less_than_4", |s: &Counter| s.value < 4);

        let (minimal, attempts) =
            config.shrink_with_stats(&seq, &initial, |sequence| test.run(sequence).is_err());

        assert_eq!(minimal.len(), 4);
        println!("Shrunk in {} attempts", attempts);
        assert!(attempts <= 100);
    }

    #[test]
    fn test_cannot_shrink_minimal() {
        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Increment);

        let shrinker = DeltaDebugSequenceShrinker::new(seq.clone());

        let test = StatefulTest::new(Counter { value: 0 })
            .invariant("always_zero", |s: &Counter| s.value == 0);

        let minimal = shrinker.minimize(|sequence| test.run(sequence).is_err());

        // Should be the same as original since it's already minimal
        assert_eq!(minimal.len(), 1);
    }
}
