//! Example: Testing a concurrent queue

use protest_stateful::concurrent::*;
use protest_stateful::operations::Operation;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A thread-safe queue
#[derive(Debug, Clone)]
struct ConcurrentQueue<T> {
    items: VecDeque<T>,
}

impl<T> ConcurrentQueue<T> {
    fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

/// Queue operations
#[derive(Debug, Clone)]
enum QueueOp {
    Enqueue(i32),
    Dequeue,
}

impl Operation for QueueOp {
    type State = ConcurrentQueue<i32>;

    fn execute(&self, state: &mut Self::State) {
        match self {
            QueueOp::Enqueue(value) => state.enqueue(*value),
            QueueOp::Dequeue => {
                state.dequeue();
            }
        }
    }
}

impl ConcurrentOperation for QueueOp {
    fn execute_concurrent(&self, state: &Arc<Mutex<Self::State>>) {
        let mut queue = state.lock().unwrap();
        self.execute(&mut queue);
    }
}

fn main() {
    println!("Testing Concurrent Queue\n");

    let initial = ConcurrentQueue::new();
    let thread_count: usize = 4;
    let ops_per_thread: usize = 50;

    // Create operations for each thread
    let mut operations = vec![];
    for thread_id in 0..thread_count {
        let mut thread_ops = vec![];

        // Each thread enqueues its own values
        for i in 0..ops_per_thread {
            let value = (thread_id * 1000 + i) as i32;
            thread_ops.push(QueueOp::Enqueue(value));
        }

        // Then dequeues some
        for _ in 0..ops_per_thread / 2 {
            thread_ops.push(QueueOp::Dequeue);
        }

        operations.push(thread_ops);
    }

    let config = ConcurrentConfig {
        thread_count,
        operations_per_thread: ops_per_thread + ops_per_thread / 2,
        check_linearizability: false,
    };

    println!(
        "Running {} threads, each performing {} operations...",
        thread_count, config.operations_per_thread
    );

    match run_concurrent(initial, operations, config) {
        Ok(final_state) => {
            println!("  ✓ Concurrent test passed!");
            println!("  Final queue length: {}", final_state.len());

            // Expected: thread_count * ops_per_thread enqueues
            //          - thread_count * (ops_per_thread / 2) dequeues
            let expected_len =
                (thread_count * ops_per_thread) - (thread_count * ops_per_thread / 2);
            println!("  Expected length: {}", expected_len);

            if final_state.len() == expected_len {
                println!("  ✓ Length matches expected value!");
            } else {
                println!("  ✗ Length mismatch!");
            }
        }
        Err(e) => {
            println!("  ✗ Concurrent test failed: {}", e);
        }
    }

    println!("\nConcurrent queue test completed!");
}
