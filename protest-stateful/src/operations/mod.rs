//! Operation execution and sequence generation for stateful testing

pub mod sequence;

use std::fmt::Debug;

/// Represents a single operation that can be applied to a state
pub trait Operation: Debug + Clone {
    /// The state type this operation modifies
    type State;

    /// Execute this operation on the given state
    fn execute(&self, state: &mut Self::State);

    /// Optional: Check if this operation can be executed in the current state
    fn precondition(&self, _state: &Self::State) -> bool {
        true // By default, all operations are always valid
    }

    /// Optional: Get a human-readable description
    fn description(&self) -> String {
        format!("{:?}", self)
    }
}

/// A sequence of operations to be executed
#[derive(Debug, Clone)]
pub struct OperationSequence<Op> {
    operations: Vec<Op>,
}

impl<Op: Operation> OperationSequence<Op> {
    /// Create a new empty sequence
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Create from a vector of operations
    pub fn from_vec(operations: Vec<Op>) -> Self {
        Self { operations }
    }

    /// Add an operation to the sequence
    pub fn push(&mut self, op: Op) {
        self.operations.push(op);
    }

    /// Get the operations
    pub fn operations(&self) -> &[Op] {
        &self.operations
    }

    /// Get the number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Execute all operations in sequence
    pub fn execute_all(&self, state: &mut Op::State) {
        for op in &self.operations {
            op.execute(state);
        }
    }

    /// Execute operations with precondition checking
    pub fn execute_with_preconditions(&self, state: &mut Op::State) -> Result<(), String> {
        for (idx, op) in self.operations.iter().enumerate() {
            if !op.precondition(state) {
                return Err(format!(
                    "Precondition failed for operation {} at index {}: {:?}",
                    idx, idx, op
                ));
            }
            op.execute(state);
        }
        Ok(())
    }

    /// Shrink the sequence by removing operations
    pub fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();

        // Try removing each operation
        for i in 0..self.operations.len() {
            let mut ops = self.operations.clone();
            ops.remove(i);
            if !ops.is_empty() {
                shrunk.push(Self::from_vec(ops));
            }
        }

        // Try removing half the operations
        if self.operations.len() > 2 {
            let half = self.operations.len() / 2;
            shrunk.push(Self::from_vec(self.operations[..half].to_vec()));
            shrunk.push(Self::from_vec(self.operations[half..].to_vec()));
        }

        // Try removing first/last operation
        if self.operations.len() > 1 {
            shrunk.push(Self::from_vec(self.operations[1..].to_vec()));
            shrunk.push(Self::from_vec(
                self.operations[..self.operations.len() - 1].to_vec(),
            ));
        }

        shrunk
    }
}

impl<Op: Operation> Default for OperationSequence<Op> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum TestOp {
        Increment,
        Decrement,
        Reset,
    }

    impl Operation for TestOp {
        type State = i32;

        fn execute(&self, state: &mut Self::State) {
            match self {
                TestOp::Increment => *state += 1,
                TestOp::Decrement => *state -= 1,
                TestOp::Reset => *state = 0,
            }
        }

        fn precondition(&self, state: &Self::State) -> bool {
            match self {
                TestOp::Decrement => *state > 0, // Can't decrement below 0
                _ => true,
            }
        }
    }

    #[test]
    fn test_operation_sequence_execution() {
        let mut state = 0;
        let mut seq = OperationSequence::new();
        seq.push(TestOp::Increment);
        seq.push(TestOp::Increment);
        seq.push(TestOp::Decrement);

        seq.execute_all(&mut state);
        assert_eq!(state, 1);
    }

    #[test]
    fn test_precondition_checking() {
        let mut state = 0;
        let mut seq = OperationSequence::new();
        seq.push(TestOp::Decrement); // Should fail precondition

        let result = seq.execute_with_preconditions(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_sequence_shrinking() {
        let mut seq = OperationSequence::new();
        seq.push(TestOp::Increment);
        seq.push(TestOp::Increment);
        seq.push(TestOp::Decrement);
        seq.push(TestOp::Reset);

        let shrunk = seq.shrink();
        assert!(!shrunk.is_empty());

        // All shrunk sequences should be smaller
        for s in shrunk {
            assert!(s.len() < seq.len());
        }
    }
}
