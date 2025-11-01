//! Linearizability Verification Example
//!
//! This example demonstrates how to use the linearizability checker to verify
//! that concurrent operations on a data structure are linearizable.
//!
//! Linearizability ensures that concurrent operations appear to execute atomically
//! at some point between their invocation and response, and that this execution
//! is consistent with some sequential specification.
//!
//! Run with: cargo run --example linearizability_verification

use protest_stateful::concurrent::linearizability::*;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Linearizability Verification Demo ===\n");

    example_1_linearizable_queue();
    println!("\n{}\n", "=".repeat(70));

    example_2_not_linearizable_queue();
    println!("\n{}\n", "=".repeat(70));

    example_3_concurrent_enqueue_dequeue();
    println!("\n{}\n", "=".repeat(70));

    example_4_complex_history();
}

// FIFO Queue Sequential Specification
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

/// Example 1: Linearizable concurrent queue operations
fn example_1_linearizable_queue() {
    println!("Example 1: Linearizable Queue Operations");
    println!("{}", "-".repeat(70));

    let mut history = History::new();
    let start = Instant::now();

    println!("Concurrent execution:");
    println!("  Thread 0: enqueue(1) [0ms - 10ms]");
    println!("  Thread 1: enqueue(2) [5ms - 15ms]  (overlaps with thread 0)");
    println!("  Thread 2: dequeue() -> 1 [20ms - 30ms]");
    println!("  Thread 3: dequeue() -> 2 [25ms - 35ms]  (overlaps with thread 2)");

    // Thread 0: enqueue(1)
    let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
    history.record_response(op1, "ok".to_string(), start + Duration::from_millis(10));

    // Thread 1: enqueue(2) (overlaps with thread 0)
    let op2 = history.record_invocation(
        1,
        "enqueue(2)".to_string(),
        start + Duration::from_millis(5),
    );
    history.record_response(op2, "ok".to_string(), start + Duration::from_millis(15));

    // Thread 2: dequeue() -> 1
    let op3 = history.record_invocation(
        2,
        "dequeue()".to_string(),
        start + Duration::from_millis(20),
    );
    history.record_response(op3, "1".to_string(), start + Duration::from_millis(30));

    // Thread 3: dequeue() -> 2 (overlaps with thread 2)
    let op4 = history.record_invocation(
        3,
        "dequeue()".to_string(),
        start + Duration::from_millis(25),
    );
    history.record_response(op4, "2".to_string(), start + Duration::from_millis(35));

    let model = FifoQueueModel::new();
    let mut checker = LinearizabilityChecker::new(model);

    println!("\nChecking linearizability...");
    let result = checker.check(&history);

    match result {
        LinearizabilityResult::Linearizable { ref order } => {
            println!("✓ History is LINEARIZABLE");
            println!("\nValid linearization order: {:?}", order);
            println!(
                "This means operations can be reordered to: enqueue(1), enqueue(2), dequeue()->1, dequeue()->2"
            );
        }
        LinearizabilityResult::NotLinearizable { ref reason, .. } => {
            println!("✗ History is NOT linearizable");
            println!("Reason: {}", reason);
        }
    }
}

/// Example 2: Non-linearizable queue operations
fn example_2_not_linearizable_queue() {
    println!("Example 2: Non-Linearizable Queue Operations");
    println!("{}", "-".repeat(70));

    let mut history = History::new();
    let start = Instant::now();

    println!("Concurrent execution:");
    println!("  Thread 0: enqueue(1) [0ms - 10ms]");
    println!("  Thread 1: enqueue(2) [20ms - 30ms]  (AFTER thread 0)");
    println!("  Thread 2: dequeue() -> 2 [40ms - 50ms]  (returns 2 instead of 1!)");

    // Thread 0: enqueue(1)
    let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
    history.record_response(op1, "ok".to_string(), start + Duration::from_millis(10));

    // Thread 1: enqueue(2) (clearly after thread 0)
    let op2 = history.record_invocation(
        1,
        "enqueue(2)".to_string(),
        start + Duration::from_millis(20),
    );
    history.record_response(op2, "ok".to_string(), start + Duration::from_millis(30));

    // Thread 2: dequeue() -> 2 (should be 1!)
    let op3 = history.record_invocation(
        2,
        "dequeue()".to_string(),
        start + Duration::from_millis(40),
    );
    history.record_response(op3, "2".to_string(), start + Duration::from_millis(50));

    let model = FifoQueueModel::new();
    let mut checker = LinearizabilityChecker::new(model);

    println!("\nChecking linearizability...");
    let result = checker.check(&history);

    match &result {
        LinearizabilityResult::Linearizable { .. } => {
            println!("✓ History is linearizable (unexpected!)");
        }
        LinearizabilityResult::NotLinearizable { reason, conflict } => {
            println!("✗ History is NOT LINEARIZABLE");
            println!("\nReason: {}", reason);
            println!("\nThis violates FIFO semantics:");
            println!("  - enqueue(1) completed before enqueue(2) started");
            println!("  - Therefore, dequeue() must return 1, not 2");
            println!("  - This indicates a bug in the queue implementation!");

            if let Some((op1, op2)) = conflict {
                println!("\nConflict details:");
                println!("  Op {}: {} -> {}", op1.op_id, op1.operation, op1.result);
                println!("  Op {}: {} -> {}", op2.op_id, op2.operation, op2.result);
            }
        }
    }

    println!("\n--- Detailed Visualization ---\n");
    println!("{}", history.visualize());
    println!("\n{}", result.visualize(&history));
}

/// Example 3: Concurrent enqueue and dequeue
fn example_3_concurrent_enqueue_dequeue() {
    println!("Example 3: Concurrent Enqueue and Dequeue");
    println!("{}", "-".repeat(70));

    let mut history = History::new();
    let start = Instant::now();

    println!("Concurrent execution:");
    println!("  Thread 0: enqueue(10) [0ms - 15ms]");
    println!("  Thread 1: dequeue() [5ms - 20ms]  (overlaps with enqueue)");
    println!();
    println!("Question: Can dequeue() return 10?");
    println!("Answer: Yes! The linearization point of enqueue(10) could be");
    println!("        at 6ms, before dequeue's linearization point.");

    // Thread 0: enqueue(10)
    let op1 = history.record_invocation(0, "enqueue(10)".to_string(), start);
    history.record_response(op1, "ok".to_string(), start + Duration::from_millis(15));

    // Thread 1: dequeue() -> 10 (overlaps with enqueue)
    let op2 =
        history.record_invocation(1, "dequeue()".to_string(), start + Duration::from_millis(5));
    history.record_response(op2, "10".to_string(), start + Duration::from_millis(20));

    let model = FifoQueueModel::new();
    let mut checker = LinearizabilityChecker::new(model);

    println!("\nChecking linearizability...");
    let result = checker.check(&history);

    match result {
        LinearizabilityResult::Linearizable { ref order } => {
            println!("✓ History is LINEARIZABLE");
            println!("\nValid linearization order: {:?}", order);
            println!("Linearization: enqueue(10) happens before dequeue()");
            println!("This is valid because operations overlapped in time!");
        }
        LinearizabilityResult::NotLinearizable { ref reason, .. } => {
            println!("✗ History is NOT linearizable");
            println!("Reason: {}", reason);
        }
    }
}

/// Example 4: Complex history with multiple threads
fn example_4_complex_history() {
    println!("Example 4: Complex Multi-Thread History");
    println!("{}", "-".repeat(70));

    let mut history = History::new();
    let start = Instant::now();

    println!("Concurrent execution:");
    println!("  Thread 0: enqueue(1) [0-5ms]");
    println!("  Thread 1: enqueue(2) [2-7ms]");
    println!("  Thread 2: enqueue(3) [4-9ms]");
    println!("  Thread 3: dequeue() [10-15ms] -> 1");
    println!("  Thread 4: dequeue() [12-17ms] -> 2");
    println!("  Thread 5: dequeue() [14-19ms] -> 3");

    // Three concurrent enqueues
    let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
    history.record_response(op1, "ok".to_string(), start + Duration::from_millis(5));

    let op2 = history.record_invocation(
        1,
        "enqueue(2)".to_string(),
        start + Duration::from_millis(2),
    );
    history.record_response(op2, "ok".to_string(), start + Duration::from_millis(7));

    let op3 = history.record_invocation(
        2,
        "enqueue(3)".to_string(),
        start + Duration::from_millis(4),
    );
    history.record_response(op3, "ok".to_string(), start + Duration::from_millis(9));

    // Three overlapping dequeues
    let op4 = history.record_invocation(
        3,
        "dequeue()".to_string(),
        start + Duration::from_millis(10),
    );
    history.record_response(op4, "1".to_string(), start + Duration::from_millis(15));

    let op5 = history.record_invocation(
        4,
        "dequeue()".to_string(),
        start + Duration::from_millis(12),
    );
    history.record_response(op5, "2".to_string(), start + Duration::from_millis(17));

    let op6 = history.record_invocation(
        5,
        "dequeue()".to_string(),
        start + Duration::from_millis(14),
    );
    history.record_response(op6, "3".to_string(), start + Duration::from_millis(19));

    let model = FifoQueueModel::new();
    let mut checker = LinearizabilityChecker::new(model);

    println!("\nChecking linearizability...");
    let result = checker.check(&history);

    match result {
        LinearizabilityResult::Linearizable { ref order } => {
            println!("✓ History is LINEARIZABLE");
            println!("\nValid linearization order: {:?}", order);
            println!("\nKey insight:");
            println!("  Even with 6 concurrent operations across 6 threads,");
            println!("  the checker found a valid sequential ordering that");
            println!("  respects both the real-time ordering and FIFO semantics!");
        }
        LinearizabilityResult::NotLinearizable { ref reason, .. } => {
            println!("✗ History is NOT linearizable");
            println!("Reason: {}", reason);
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("Summary:");
    println!("  - Linearizability checks if concurrent operations are");
    println!("    equivalent to some sequential execution");
    println!("  - Operations that don't overlap in time must respect");
    println!("    their real-time ordering");
    println!("  - Overlapping operations can be reordered arbitrarily");
    println!("  - This is essential for verifying concurrent data structures!");
}
