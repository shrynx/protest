//! Linearizability verification for concurrent operations
//!
//! This module implements linearizability checking for concurrent executions.
//! Linearizability ensures that concurrent operations appear to execute atomically
//! at some point between their invocation and response.
//!
//! # Background
//!
//! A concurrent history is linearizable if:
//! 1. Each operation appears to take effect instantaneously at some point (linearization point)
//!    between its invocation and response
//! 2. The linearization points respect the sequential specification
//! 3. The real-time ordering is preserved (if op1 completes before op2 starts, op1's
//!    linearization point must precede op2's)
//!
//! # Algorithm
//!
//! We implement a modified Wing & Gong algorithm that:
//! - Records invocation and response events for each operation
//! - Searches for valid linearization orders
//! - Uses backtracking to find a sequential history that matches the specification
//!
//! # Example
//!
//! ```rust
//! use protest_stateful::concurrent::linearizability::*;
//! use std::time::Instant;
//!
//! // Record a concurrent execution
//! let mut history = History::new();
//! let start = Instant::now();
//!
//! // Thread 1: enqueue(1)
//! let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
//! let op1 = history.record_response(op1, "ok".to_string(), start + std::time::Duration::from_millis(10));
//!
//! // Thread 2: dequeue() -> 1
//! let op2 = history.record_invocation(1, "dequeue()".to_string(), start + std::time::Duration::from_millis(5));
//! let op2 = history.record_response(op2, "1".to_string(), start + std::time::Duration::from_millis(15));
//!
//! // Check if the history is linearizable
//! // (Would need a model to check against)
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug, Display};
use std::time::Instant;

/// A unique identifier for an operation
pub type OperationId = usize;

/// A thread identifier
pub type ThreadId = usize;

/// An event in a concurrent execution history
#[derive(Debug, Clone, PartialEq)]
pub enum HistoryEvent {
    /// An operation was invoked
    Invocation {
        op_id: OperationId,
        thread_id: ThreadId,
        operation: String,
        time: Instant,
    },
    /// An operation returned
    Response {
        op_id: OperationId,
        thread_id: ThreadId,
        result: String,
        time: Instant,
    },
}

impl HistoryEvent {
    /// Get the operation ID for this event
    pub fn op_id(&self) -> OperationId {
        match self {
            HistoryEvent::Invocation { op_id, .. } => *op_id,
            HistoryEvent::Response { op_id, .. } => *op_id,
        }
    }

    /// Get the thread ID for this event
    pub fn thread_id(&self) -> ThreadId {
        match self {
            HistoryEvent::Invocation { thread_id, .. } => *thread_id,
            HistoryEvent::Response { thread_id, .. } => *thread_id,
        }
    }

    /// Get the timestamp for this event
    pub fn time(&self) -> Instant {
        match self {
            HistoryEvent::Invocation { time, .. } => *time,
            HistoryEvent::Response { time, .. } => *time,
        }
    }

    /// Check if this is an invocation event
    pub fn is_invocation(&self) -> bool {
        matches!(self, HistoryEvent::Invocation { .. })
    }

    /// Check if this is a response event
    pub fn is_response(&self) -> bool {
        matches!(self, HistoryEvent::Response { .. })
    }
}

/// A complete operation with invocation and response
#[derive(Debug, Clone, PartialEq)]
pub struct CompletedOperation {
    pub op_id: OperationId,
    pub thread_id: ThreadId,
    pub operation: String,
    pub result: String,
    pub invocation_time: Instant,
    pub response_time: Instant,
}

impl CompletedOperation {
    /// Check if this operation happens before another in real-time
    /// (this operation completes before the other starts)
    pub fn happens_before(&self, other: &CompletedOperation) -> bool {
        self.response_time < other.invocation_time
    }
}

/// A pending operation (invoked but not yet returned)
#[derive(Debug, Clone)]
struct PendingOperation {
    thread_id: ThreadId,
    operation: String,
    invocation_time: Instant,
}

/// A history of concurrent operations
#[derive(Debug, Clone)]
pub struct History {
    events: Vec<HistoryEvent>,
    next_op_id: OperationId,
    pending: HashMap<OperationId, PendingOperation>,
    completed: Vec<CompletedOperation>,
}

impl History {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_op_id: 0,
            pending: HashMap::new(),
            completed: Vec::new(),
        }
    }

    /// Record an operation invocation and return its ID
    pub fn record_invocation(
        &mut self,
        thread_id: ThreadId,
        operation: String,
        time: Instant,
    ) -> OperationId {
        let op_id = self.next_op_id;
        self.next_op_id += 1;

        let event = HistoryEvent::Invocation {
            op_id,
            thread_id,
            operation: operation.clone(),
            time,
        };

        self.events.push(event);
        self.pending.insert(
            op_id,
            PendingOperation {
                thread_id,
                operation,
                invocation_time: time,
            },
        );

        op_id
    }

    /// Record an operation response
    pub fn record_response(
        &mut self,
        op_id: OperationId,
        result: String,
        time: Instant,
    ) -> OperationId {
        if let Some(pending_op) = self.pending.remove(&op_id) {
            let event = HistoryEvent::Response {
                op_id,
                thread_id: pending_op.thread_id,
                result: result.clone(),
                time,
            };

            self.events.push(event);

            let completed = CompletedOperation {
                op_id,
                thread_id: pending_op.thread_id,
                operation: pending_op.operation,
                result,
                invocation_time: pending_op.invocation_time,
                response_time: time,
            };

            self.completed.push(completed);
        }

        op_id
    }

    /// Get all completed operations
    pub fn completed_operations(&self) -> &[CompletedOperation] {
        &self.completed
    }

    /// Get all events
    pub fn events(&self) -> &[HistoryEvent] {
        &self.events
    }

    /// Check if all operations have completed
    pub fn all_completed(&self) -> bool {
        self.pending.is_empty()
    }

    /// Get the number of pending operations
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Visualize the history as a timeline
    pub fn visualize(&self) -> String {
        let mut output = String::new();
        output.push_str("Timeline Visualization:\n");
        output.push_str(&"=".repeat(70));
        output.push('\n');

        // Group operations by thread
        let mut ops_by_thread: HashMap<ThreadId, Vec<&CompletedOperation>> = HashMap::new();
        for op in &self.completed {
            ops_by_thread.entry(op.thread_id).or_default().push(op);
        }

        // Get min/max times for scaling
        if self.completed.is_empty() {
            output.push_str("(empty history)\n");
            return output;
        }

        let min_time = self
            .completed
            .iter()
            .map(|op| op.invocation_time)
            .min()
            .unwrap();
        let max_time = self
            .completed
            .iter()
            .map(|op| op.response_time)
            .max()
            .unwrap();
        let duration = max_time.duration_since(min_time);

        // Display each thread's operations
        let mut thread_ids: Vec<_> = ops_by_thread.keys().collect();
        thread_ids.sort();

        for &thread_id in &thread_ids {
            output.push_str(&format!("Thread {}: ", thread_id));

            if let Some(ops) = ops_by_thread.get(thread_id) {
                for (i, op) in ops.iter().enumerate() {
                    if i > 0 {
                        output.push_str(&" ".repeat(10));
                    }

                    let start_offset = op.invocation_time.duration_since(min_time);
                    let end_offset = op.response_time.duration_since(min_time);

                    output.push_str(&format!(
                        "{}({}) [{:.1}ms - {:.1}ms] -> {}\n",
                        op.operation.split('(').next().unwrap_or(&op.operation),
                        op.operation
                            .split('(')
                            .nth(1)
                            .map(|s| s.trim_end_matches(')'))
                            .unwrap_or(""),
                        start_offset.as_secs_f64() * 1000.0,
                        end_offset.as_secs_f64() * 1000.0,
                        op.result
                    ));

                    if i == 0 {
                        output.push_str(&" ".repeat(10));
                    }
                }
            }
            output.push('\n');
        }

        output.push_str(&"=".repeat(70));
        output.push('\n');
        output.push_str(&format!(
            "Total duration: {:.1}ms\n",
            duration.as_secs_f64() * 1000.0
        ));

        output
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a linearizability check
#[derive(Debug, Clone)]
pub enum LinearizabilityResult {
    /// The history is linearizable with the given order
    Linearizable {
        /// A valid sequential ordering of operations
        order: Vec<OperationId>,
    },
    /// The history is not linearizable
    NotLinearizable {
        /// Description of why it's not linearizable
        reason: String,
        /// Conflicting operations that violate linearizability
        conflict: Option<(CompletedOperation, CompletedOperation)>,
    },
}

impl Display for LinearizabilityResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinearizabilityResult::Linearizable { order } => {
                write!(f, "Linearizable (order: {:?})", order)
            }
            LinearizabilityResult::NotLinearizable { reason, conflict } => {
                write!(f, "Not linearizable: {}", reason)?;
                if let Some((op1, op2)) = conflict {
                    write!(
                        f,
                        "\nConflict between:\n  Op {}: {} -> {}\n  Op {}: {} -> {}",
                        op1.op_id, op1.operation, op1.result, op2.op_id, op2.operation, op2.result
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl LinearizabilityResult {
    /// Visualize the result with detailed information
    pub fn visualize(&self, history: &History) -> String {
        let mut output = String::new();

        match self {
            LinearizabilityResult::Linearizable { order } => {
                output.push_str("✓ LINEARIZABLE\n");
                output.push_str(&"=".repeat(70));
                output.push('\n');
                output.push_str("\nValid sequential ordering:\n");

                for (i, &op_id) in order.iter().enumerate() {
                    if let Some(op) = history
                        .completed_operations()
                        .iter()
                        .find(|o| o.op_id == op_id)
                    {
                        output.push_str(&format!(
                            "  {}. {} -> {}\n",
                            i + 1,
                            op.operation,
                            op.result
                        ));
                    }
                }

                output.push('\n');
                output.push_str("This means the concurrent execution is equivalent to\n");
                output.push_str("executing these operations sequentially in the above order.\n");
            }
            LinearizabilityResult::NotLinearizable { reason, conflict } => {
                output.push_str("✗ NOT LINEARIZABLE\n");
                output.push_str(&"=".repeat(70));
                output.push('\n');
                output.push_str(&format!("\nReason: {}\n", reason));

                if let Some((op1, op2)) = conflict {
                    output.push_str("\nConflicting operations:\n");
                    output.push_str(&format!(
                        "  Op {} (Thread {}): {} -> {}\n",
                        op1.op_id, op1.thread_id, op1.operation, op1.result
                    ));
                    output.push_str(&format!(
                        "    Invoked at:  {:.1}ms\n",
                        op1.invocation_time.elapsed().as_secs_f64() * 1000.0
                    ));
                    output.push_str(&format!(
                        "    Returned at: {:.1}ms\n",
                        op1.response_time.elapsed().as_secs_f64() * 1000.0
                    ));
                    output.push('\n');
                    output.push_str(&format!(
                        "  Op {} (Thread {}): {} -> {}\n",
                        op2.op_id, op2.thread_id, op2.operation, op2.result
                    ));
                    output.push_str(&format!(
                        "    Invoked at:  {:.1}ms\n",
                        op2.invocation_time.elapsed().as_secs_f64() * 1000.0
                    ));
                    output.push_str(&format!(
                        "    Returned at: {:.1}ms\n",
                        op2.response_time.elapsed().as_secs_f64() * 1000.0
                    ));

                    if op1.happens_before(op2) {
                        output.push_str(&format!(
                            "\n⚠ Op {} completed before Op {} started (happens-before)\n",
                            op1.op_id, op2.op_id
                        ));
                        output.push_str("   but the results violate sequential consistency!\n");
                    }
                }

                output.push('\n');
                output
                    .push_str("This indicates a potential bug in the concurrent data structure.\n");
            }
        }

        output
    }
}

/// A model for checking sequential correctness
///
/// This trait defines the sequential behavior that concurrent operations
/// should be equivalent to.
pub trait SequentialSpec {
    /// Apply an operation to the model and return the expected result
    fn apply(&mut self, operation: &str) -> String;

    /// Check if a result matches the expected result
    fn matches(&self, expected: &str, actual: &str) -> bool {
        expected == actual
    }

    /// Reset the model to initial state
    fn reset(&mut self);
}

/// Linearizability checker
pub struct LinearizabilityChecker<S: SequentialSpec> {
    spec: S,
}

impl<S: SequentialSpec> LinearizabilityChecker<S> {
    /// Create a new linearizability checker with the given specification
    pub fn new(spec: S) -> Self {
        Self { spec }
    }

    /// Check if a history is linearizable
    pub fn check(&mut self, history: &History) -> LinearizabilityResult {
        let operations = history.completed_operations();

        if operations.is_empty() {
            return LinearizabilityResult::Linearizable { order: vec![] };
        }

        // Build the happens-before graph
        let happens_before = self.build_happens_before_graph(operations);

        // Try to find a valid linearization
        let mut visited = HashSet::new();
        let mut current_order = Vec::new();

        if self.find_linearization(
            operations,
            &happens_before,
            &mut visited,
            &mut current_order,
        ) {
            LinearizabilityResult::Linearizable {
                order: current_order,
            }
        } else {
            // Find a conflict to report
            let conflict = self.find_conflict(operations);
            LinearizabilityResult::NotLinearizable {
                reason: "No valid linearization found".to_string(),
                conflict,
            }
        }
    }

    /// Build the happens-before graph
    /// Returns a map where each operation ID maps to the set of operations that must come BEFORE it
    fn build_happens_before_graph(
        &self,
        operations: &[CompletedOperation],
    ) -> HashMap<OperationId, HashSet<OperationId>> {
        let mut graph: HashMap<OperationId, HashSet<OperationId>> = HashMap::new();

        for op1 in operations {
            for op2 in operations {
                if op1.op_id != op2.op_id && op1.happens_before(op2) {
                    // op1 happens before op2, so op2 depends on op1
                    graph.entry(op2.op_id).or_default().insert(op1.op_id);
                }
            }
        }

        graph
    }

    /// Try to find a valid linearization using backtracking
    fn find_linearization(
        &mut self,
        operations: &[CompletedOperation],
        happens_before: &HashMap<OperationId, HashSet<OperationId>>,
        visited: &mut HashSet<OperationId>,
        current_order: &mut Vec<OperationId>,
    ) -> bool {
        // Base case: all operations placed
        if visited.len() == operations.len() {
            return self.verify_sequential_correctness(operations, current_order);
        }

        // Try each unvisited operation
        for op in operations {
            if visited.contains(&op.op_id) {
                continue;
            }

            // Check if all operations that must happen before this one are already placed
            if let Some(predecessors) = happens_before.get(&op.op_id)
                && !predecessors.iter().all(|pred| visited.contains(pred))
            {
                continue;
            }

            // Try placing this operation
            visited.insert(op.op_id);
            current_order.push(op.op_id);

            if self.find_linearization(operations, happens_before, visited, current_order) {
                return true;
            }

            // Backtrack
            current_order.pop();
            visited.remove(&op.op_id);
        }

        false
    }

    /// Verify that an ordering is sequentially correct
    fn verify_sequential_correctness(
        &mut self,
        operations: &[CompletedOperation],
        order: &[OperationId],
    ) -> bool {
        self.spec.reset();

        let op_map: HashMap<OperationId, &CompletedOperation> =
            operations.iter().map(|op| (op.op_id, op)).collect();

        for &op_id in order {
            if let Some(op) = op_map.get(&op_id) {
                let expected_result = self.spec.apply(&op.operation);
                if !self.spec.matches(&expected_result, &op.result) {
                    return false;
                }
            }
        }

        true
    }

    /// Find a pair of conflicting operations
    fn find_conflict(
        &self,
        operations: &[CompletedOperation],
    ) -> Option<(CompletedOperation, CompletedOperation)> {
        // Find first pair that violates real-time ordering
        for i in 0..operations.len() {
            for j in (i + 1)..operations.len() {
                let op1 = &operations[i];
                let op2 = &operations[j];

                if op1.happens_before(op2) {
                    return Some((op1.clone(), op2.clone()));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::time::Duration;

    // Simple FIFO queue model for testing
    #[derive(Debug)]
    struct FifoQueueModel {
        queue: VecDeque<i32>,
    }

    impl FifoQueueModel {
        fn new() -> Self {
            Self {
                queue: VecDeque::new(),
            }
        }
    }

    impl SequentialSpec for FifoQueueModel {
        fn apply(&mut self, operation: &str) -> String {
            if let Some(enq_val) = operation.strip_prefix("enqueue(") {
                let val: i32 = enq_val.trim_end_matches(')').parse().unwrap();
                self.queue.push_back(val);
                "ok".to_string()
            } else if operation == "dequeue()" {
                self.queue
                    .pop_front()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "empty".to_string())
            } else {
                "unknown".to_string()
            }
        }

        fn reset(&mut self) {
            self.queue.clear();
        }
    }

    #[test]
    fn test_history_recording() {
        let mut history = History::new();
        let start = Instant::now();

        let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
        let _op1 =
            history.record_response(op1, "ok".to_string(), start + Duration::from_millis(10));

        assert_eq!(history.completed_operations().len(), 1);
        assert!(history.all_completed());
    }

    #[test]
    fn test_linearizable_simple() {
        let mut history = History::new();
        let start = Instant::now();

        // Thread 0: enqueue(1)
        let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
        history.record_response(op1, "ok".to_string(), start + Duration::from_millis(10));

        // Thread 1: dequeue() -> 1
        let op2 = history.record_invocation(
            1,
            "dequeue()".to_string(),
            start + Duration::from_millis(20),
        );
        history.record_response(op2, "1".to_string(), start + Duration::from_millis(30));

        let model = FifoQueueModel::new();
        let mut checker = LinearizabilityChecker::new(model);

        let result = checker.check(&history);
        assert!(matches!(result, LinearizabilityResult::Linearizable { .. }));
    }

    #[test]
    fn test_linearizable_concurrent() {
        let mut history = History::new();
        let start = Instant::now();

        // Thread 0: enqueue(1) [0-10ms]
        let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
        history.record_response(op1, "ok".to_string(), start + Duration::from_millis(10));

        // Thread 1: enqueue(2) [5-15ms] (overlaps with op1)
        let op2 = history.record_invocation(
            1,
            "enqueue(2)".to_string(),
            start + Duration::from_millis(5),
        );
        history.record_response(op2, "ok".to_string(), start + Duration::from_millis(15));

        // Thread 0: dequeue() -> 1 [20-30ms]
        let op3 = history.record_invocation(
            0,
            "dequeue()".to_string(),
            start + Duration::from_millis(20),
        );
        history.record_response(op3, "1".to_string(), start + Duration::from_millis(30));

        let model = FifoQueueModel::new();
        let mut checker = LinearizabilityChecker::new(model);

        let result = checker.check(&history);
        assert!(matches!(result, LinearizabilityResult::Linearizable { .. }));
    }

    #[test]
    fn test_not_linearizable() {
        let mut history = History::new();
        let start = Instant::now();

        // Thread 0: enqueue(1) [0-10ms]
        let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
        history.record_response(op1, "ok".to_string(), start + Duration::from_millis(10));

        // Thread 1: dequeue() -> 2 [20-30ms] (should be 1, not 2!)
        let op2 = history.record_invocation(
            1,
            "dequeue()".to_string(),
            start + Duration::from_millis(20),
        );
        history.record_response(op2, "2".to_string(), start + Duration::from_millis(30));

        let model = FifoQueueModel::new();
        let mut checker = LinearizabilityChecker::new(model);

        let result = checker.check(&history);
        assert!(matches!(
            result,
            LinearizabilityResult::NotLinearizable { .. }
        ));
    }

    #[test]
    fn test_happens_before() {
        let start = Instant::now();

        let op1 = CompletedOperation {
            op_id: 1,
            thread_id: 0,
            operation: "enqueue(1)".to_string(),
            result: "ok".to_string(),
            invocation_time: start,
            response_time: start + Duration::from_millis(10),
        };

        let op2 = CompletedOperation {
            op_id: 2,
            thread_id: 1,
            operation: "dequeue()".to_string(),
            result: "1".to_string(),
            invocation_time: start + Duration::from_millis(20),
            response_time: start + Duration::from_millis(30),
        };

        assert!(op1.happens_before(&op2));
        assert!(!op2.happens_before(&op1));
    }
}
