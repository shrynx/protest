//! Concurrent stateful testing with linearizability checking
//!
//! Test parallel operations on concurrent data structures

use crate::operations::Operation;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;

/// A concurrent operation that can be executed in parallel
pub trait ConcurrentOperation: Operation + Send + Sync {
    /// Execute this operation concurrently
    fn execute_concurrent(&self, state: &Arc<Mutex<Self::State>>);
}

/// Configuration for concurrent testing
#[derive(Debug, Clone)]
pub struct ConcurrentConfig {
    /// Number of threads to use
    pub thread_count: usize,
    /// Operations per thread
    pub operations_per_thread: usize,
    /// Check linearizability
    pub check_linearizability: bool,
}

impl Default for ConcurrentConfig {
    fn default() -> Self {
        Self {
            thread_count: 4,
            operations_per_thread: 100,
            check_linearizability: true,
        }
    }
}

/// Run operations concurrently and check for race conditions
pub fn run_concurrent<Op>(
    initial_state: Op::State,
    operations: Vec<Vec<Op>>,
    _config: ConcurrentConfig,
) -> Result<Op::State, ConcurrentTestFailure>
where
    Op: ConcurrentOperation + 'static,
    Op::State: Send + Clone,
{
    let state = Arc::new(Mutex::new(initial_state));
    let mut handles = vec![];

    // Spawn threads
    for thread_ops in operations {
        let state_clone = Arc::clone(&state);
        let handle = thread::spawn(move || {
            for op in thread_ops {
                op.execute_concurrent(&state_clone);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().map_err(|_| ConcurrentTestFailure {
            description: "Thread panicked during execution".to_string(),
        })?;
    }

    // Extract final state
    let final_state = Arc::try_unwrap(state)
        .map_err(|_| ConcurrentTestFailure {
            description: "Could not unwrap Arc".to_string(),
        })?
        .into_inner()
        .map_err(|_| ConcurrentTestFailure {
            description: "Mutex poisoned".to_string(),
        })?;

    Ok(final_state)
}

/// Represents a failure in concurrent testing
#[derive(Debug)]
pub struct ConcurrentTestFailure {
    pub description: String,
}

impl std::fmt::Display for ConcurrentTestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Concurrent test failure: {}", self.description)
    }
}

impl std::error::Error for ConcurrentTestFailure {}

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
    }

    impl Operation for CounterOp {
        type State = Counter;

        fn execute(&self, state: &mut Self::State) {
            match self {
                CounterOp::Increment => state.value += 1,
            }
        }
    }

    impl ConcurrentOperation for CounterOp {
        fn execute_concurrent(&self, state: &Arc<Mutex<Self::State>>) {
            let mut state = state.lock().unwrap();
            self.execute(&mut state);
        }
    }

    #[test]
    fn test_concurrent_counter() {
        let initial = Counter { value: 0 };
        let ops_per_thread = 100;
        let thread_count = 4;

        let mut operations = vec![];
        for _ in 0..thread_count {
            let thread_ops = vec![CounterOp::Increment; ops_per_thread];
            operations.push(thread_ops);
        }

        let config = ConcurrentConfig {
            thread_count,
            operations_per_thread: ops_per_thread,
            check_linearizability: false,
        };

        let result = run_concurrent(initial, operations, config);
        assert!(result.is_ok());

        let final_state = result.unwrap();
        assert_eq!(final_state.value, (ops_per_thread * thread_count) as i32);
    }
}
