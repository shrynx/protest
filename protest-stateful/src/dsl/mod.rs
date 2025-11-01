//! Domain-specific language for stateful property testing

use crate::invariants::{InvariantSet, InvariantViolation};
use crate::operations::{Operation, OperationSequence};
use std::fmt::Debug;

/// A stateful property test
pub struct StatefulTest<State, Op>
where
    State: Clone + Debug + 'static,
    Op: Operation<State = State>,
{
    initial_state: State,
    invariants: InvariantSet<State>,
    _phantom: std::marker::PhantomData<Op>,
}

impl<State, Op> StatefulTest<State, Op>
where
    State: Clone + Debug + 'static,
    Op: Operation<State = State>,
{
    /// Create a new stateful test with an initial state
    pub fn new(initial_state: State) -> Self {
        Self {
            initial_state,
            invariants: InvariantSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add an invariant to check after each operation
    pub fn invariant<F>(mut self, name: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn(&State) -> bool + 'static,
    {
        self.invariants.add_fn(name, check_fn);
        self
    }

    /// Execute a sequence of operations and check invariants
    pub fn run(&self, sequence: &OperationSequence<Op>) -> Result<State, StatefulTestFailure> {
        let mut state = self.initial_state.clone();

        // Check initial invariants
        if let Err(violation) = self.invariants.check_all(&state) {
            return Err(StatefulTestFailure {
                operation_index: None,
                operation: None,
                state_before: None,
                violation,
            });
        }

        // Execute operations one by one
        for (idx, op) in sequence.operations().iter().enumerate() {
            let state_before = state.clone();

            // Check precondition
            if !op.precondition(&state) {
                return Err(StatefulTestFailure {
                    operation_index: Some(idx),
                    operation: Some(format!("{:?}", op)),
                    state_before: Some(format!("{:?}", state_before)),
                    violation: InvariantViolation {
                        description: format!("Precondition failed for operation: {:?}", op),
                    },
                });
            }

            // Execute operation
            op.execute(&mut state);

            // Check invariants
            if let Err(violation) = self.invariants.check_all(&state) {
                return Err(StatefulTestFailure {
                    operation_index: Some(idx),
                    operation: Some(format!("{:?}", op)),
                    state_before: Some(format!("{:?}", state_before)),
                    violation,
                });
            }
        }

        Ok(state)
    }

    /// Execute with detailed trace
    pub fn run_with_trace(
        &self,
        sequence: &OperationSequence<Op>,
    ) -> Result<ExecutionTrace<State>, StatefulTestFailure> {
        let mut trace = ExecutionTrace::new(self.initial_state.clone());
        let mut state = self.initial_state.clone();

        for (idx, op) in sequence.operations().iter().enumerate() {
            let state_before = state.clone();

            if !op.precondition(&state) {
                return Err(StatefulTestFailure {
                    operation_index: Some(idx),
                    operation: Some(format!("{:?}", op)),
                    state_before: Some(format!("{:?}", state_before)),
                    violation: InvariantViolation {
                        description: format!("Precondition failed for operation: {:?}", op),
                    },
                });
            }

            op.execute(&mut state);

            trace.add_step(op.description(), state.clone());

            if let Err(violation) = self.invariants.check_all(&state) {
                return Err(StatefulTestFailure {
                    operation_index: Some(idx),
                    operation: Some(format!("{:?}", op)),
                    state_before: Some(format!("{:?}", state_before)),
                    violation,
                });
            }
        }

        Ok(trace)
    }
}

/// Represents a failure in a stateful test
#[derive(Debug)]
pub struct StatefulTestFailure {
    pub operation_index: Option<usize>,
    pub operation: Option<String>,
    pub state_before: Option<String>,
    pub violation: InvariantViolation,
}

impl std::fmt::Display for StatefulTestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stateful test failed: {}", self.violation)?;
        if let Some(idx) = self.operation_index {
            write!(f, "\n  At operation index: {}", idx)?;
        }
        if let Some(ref op) = self.operation {
            write!(f, "\n  Operation: {}", op)?;
        }
        if let Some(ref state) = self.state_before {
            write!(f, "\n  State before: {}", state)?;
        }
        Ok(())
    }
}

impl std::error::Error for StatefulTestFailure {}

/// A trace of execution showing state at each step
#[derive(Debug, Clone)]
pub struct ExecutionTrace<State> {
    initial_state: State,
    steps: Vec<(String, State)>, // (operation description, resulting state)
}

impl<State: Clone> ExecutionTrace<State> {
    /// Create a new trace
    pub fn new(initial_state: State) -> Self {
        Self {
            initial_state,
            steps: Vec::new(),
        }
    }

    /// Add a step to the trace
    pub fn add_step(&mut self, operation: String, state: State) {
        self.steps.push((operation, state));
    }

    /// Get the initial state
    pub fn initial_state(&self) -> &State {
        &self.initial_state
    }

    /// Get all steps
    pub fn steps(&self) -> &[(String, State)] {
        &self.steps
    }

    /// Get the final state
    pub fn final_state(&self) -> Option<&State> {
        self.steps.last().map(|(_, state)| state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct Counter {
        value: i32,
    }

    #[derive(Debug, Clone)]
    enum CounterOp {
        Increment,
        Decrement,
        #[allow(dead_code)]
        Reset,
    }

    impl Operation for CounterOp {
        type State = Counter;

        fn execute(&self, state: &mut Self::State) {
            match self {
                CounterOp::Increment => state.value += 1,
                CounterOp::Decrement => state.value -= 1,
                CounterOp::Reset => state.value = 0,
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
    fn test_stateful_test_success() {
        let test = StatefulTest::new(Counter { value: 0 })
            .invariant("non_negative", |state: &Counter| state.value >= 0);

        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Decrement);

        let result = test.run(&seq);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 1);
    }

    #[test]
    fn test_stateful_test_invariant_failure() {
        let test = StatefulTest::new(Counter { value: 0 })
            .invariant("less_than_2", |state: &Counter| state.value < 2);

        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment); // This will violate invariant

        let result = test.run(&seq);
        assert!(result.is_err());
    }

    #[test]
    fn test_execution_trace() {
        let test = StatefulTest::new(Counter { value: 0 });

        let mut seq = OperationSequence::new();
        seq.push(CounterOp::Increment);
        seq.push(CounterOp::Increment);

        let trace = test.run_with_trace(&seq).unwrap();
        assert_eq!(trace.steps().len(), 2);
        assert_eq!(trace.final_state().unwrap().value, 2);
    }
}
