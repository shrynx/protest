//! Model-based testing - compare system against reference implementation

use crate::operations::{Operation, OperationSequence};
use std::fmt::Debug;

/// A model that represents the expected behavior of a system
pub trait Model: Clone + Debug {
    /// The type of the actual system state
    type SystemState: Debug;

    /// The type of operations
    type Operation: Operation<State = Self::SystemState>;

    /// Execute an operation on the model
    fn execute_model(&mut self, op: &Self::Operation);

    /// Check if the system state matches the model
    fn matches(&self, system_state: &Self::SystemState) -> bool;

    /// Get a description of the mismatch (if any)
    fn mismatch_description(&self, system_state: &Self::SystemState) -> Option<String> {
        if self.matches(system_state) {
            None
        } else {
            Some(format!(
                "Model state {:?} does not match system state {:?}",
                self, system_state
            ))
        }
    }
}

/// Model-based testing runner
pub struct ModelBasedTest<M>
where
    M: Model,
{
    initial_model: M,
    initial_system: M::SystemState,
}

impl<M> ModelBasedTest<M>
where
    M: Model + Debug,
    M::SystemState: Clone + Debug,
{
    /// Create a new model-based test
    pub fn new(initial_model: M, initial_system: M::SystemState) -> Self {
        Self {
            initial_model,
            initial_system,
        }
    }

    /// Run a sequence of operations on both model and system
    pub fn run(&self, sequence: &OperationSequence<M::Operation>) -> Result<(), ModelMismatch> {
        let mut model = self.initial_model.clone();
        let mut system = self.initial_system.clone();

        // Check initial state
        if !model.matches(&system) {
            return Err(ModelMismatch {
                operation_index: None,
                operation: None,
                description: model
                    .mismatch_description(&system)
                    .unwrap_or_else(|| "Initial state mismatch".to_string()),
            });
        }

        // Execute each operation on both model and system
        for (idx, op) in sequence.operations().iter().enumerate() {
            // Execute on model
            model.execute_model(op);

            // Execute on system
            op.execute(&mut system);

            // Check equivalence
            if !model.matches(&system) {
                return Err(ModelMismatch {
                    operation_index: Some(idx),
                    operation: Some(format!("{:?}", op)),
                    description: model
                        .mismatch_description(&system)
                        .unwrap_or_else(|| "State mismatch after operation".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Run with detailed trace
    pub fn run_with_trace(
        &self,
        sequence: &OperationSequence<M::Operation>,
    ) -> Result<ModelTrace<M, M::SystemState>, ModelMismatch> {
        let mut trace = ModelTrace::new(self.initial_model.clone(), self.initial_system.clone());
        let mut model = self.initial_model.clone();
        let mut system = self.initial_system.clone();

        for (idx, op) in sequence.operations().iter().enumerate() {
            model.execute_model(op);
            op.execute(&mut system);

            trace.add_step(op.description(), model.clone(), system.clone());

            if !model.matches(&system) {
                return Err(ModelMismatch {
                    operation_index: Some(idx),
                    operation: Some(format!("{:?}", op)),
                    description: model
                        .mismatch_description(&system)
                        .unwrap_or_else(|| "State mismatch".to_string()),
                });
            }
        }

        Ok(trace)
    }
}

/// Represents a mismatch between model and system
#[derive(Debug)]
pub struct ModelMismatch {
    pub operation_index: Option<usize>,
    pub operation: Option<String>,
    pub description: String,
}

impl std::fmt::Display for ModelMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model mismatch: {}", self.description)?;
        if let Some(idx) = self.operation_index {
            write!(f, "\n  At operation index: {}", idx)?;
        }
        if let Some(ref op) = self.operation {
            write!(f, "\n  Operation: {}", op)?;
        }
        Ok(())
    }
}

impl std::error::Error for ModelMismatch {}

/// Trace of model-based execution
#[derive(Debug, Clone)]
pub struct ModelTrace<M, S> {
    _initial_model: M,
    _initial_system: S,
    steps: Vec<(String, M, S)>, // (operation, model state, system state)
}

impl<M: Clone, S: Clone> ModelTrace<M, S> {
    /// Create a new trace
    pub fn new(initial_model: M, initial_system: S) -> Self {
        Self {
            _initial_model: initial_model,
            _initial_system: initial_system,
            steps: Vec::new(),
        }
    }

    /// Add a step
    pub fn add_step(&mut self, operation: String, model: M, system: S) {
        self.steps.push((operation, model, system));
    }

    /// Get the steps
    pub fn steps(&self) -> &[(String, M, S)] {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple key-value store system
    #[derive(Debug, Clone)]
    struct KVStore {
        data: std::collections::HashMap<String, String>,
    }

    #[derive(Debug, Clone)]
    enum KVOp {
        Set(String, String),
        #[allow(dead_code)]
        Get(String),
        Delete(String),
    }

    impl Operation for KVOp {
        type State = KVStore;

        fn execute(&self, state: &mut Self::State) {
            match self {
                KVOp::Set(k, v) => {
                    state.data.insert(k.clone(), v.clone());
                }
                KVOp::Get(_k) => {
                    // Read-only, no state change
                }
                KVOp::Delete(k) => {
                    state.data.remove(k);
                }
            }
        }
    }

    // Simple model (just a HashMap)
    #[derive(Debug, Clone)]
    struct KVModel {
        data: std::collections::HashMap<String, String>,
    }

    impl Model for KVModel {
        type SystemState = KVStore;
        type Operation = KVOp;

        fn execute_model(&mut self, op: &Self::Operation) {
            match op {
                KVOp::Set(k, v) => {
                    self.data.insert(k.clone(), v.clone());
                }
                KVOp::Get(_) => {} // Read-only
                KVOp::Delete(k) => {
                    self.data.remove(k);
                }
            }
        }

        fn matches(&self, system_state: &Self::SystemState) -> bool {
            self.data == system_state.data
        }
    }

    #[test]
    fn test_model_based_testing() {
        let model = KVModel {
            data: std::collections::HashMap::new(),
        };
        let system = KVStore {
            data: std::collections::HashMap::new(),
        };

        let test = ModelBasedTest::new(model, system);

        let mut seq = OperationSequence::new();
        seq.push(KVOp::Set("key1".to_string(), "value1".to_string()));
        seq.push(KVOp::Set("key2".to_string(), "value2".to_string()));
        seq.push(KVOp::Delete("key1".to_string()));

        let result = test.run(&seq);
        assert!(result.is_ok());
    }
}
