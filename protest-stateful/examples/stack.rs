//! Example: Testing a Stack implementation with stateful property testing

use protest_stateful::prelude::*;

/// A simple stack implementation
#[derive(Debug, Clone)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Operations that can be performed on the stack
#[derive(Debug, Clone)]
enum StackOp {
    Push(i32),
    Pop,
    Peek,
}

impl Operation for StackOp {
    type State = Stack<i32>;

    fn execute(&self, state: &mut Self::State) {
        match self {
            StackOp::Push(value) => state.push(*value),
            StackOp::Pop => {
                state.pop();
            }
            StackOp::Peek => {
                state.peek();
            }
        }
    }

    fn precondition(&self, state: &Self::State) -> bool {
        match self {
            StackOp::Pop | StackOp::Peek => !state.is_empty(),
            StackOp::Push(_) => true,
        }
    }

    fn description(&self) -> String {
        match self {
            StackOp::Push(v) => format!("Push({})", v),
            StackOp::Pop => "Pop".to_string(),
            StackOp::Peek => "Peek".to_string(),
        }
    }
}

fn main() {
    println!("Testing Stack with Stateful Property Testing\n");

    // Create a stateful test
    let test = StatefulTest::new(Stack::new())
        .invariant("length_non_negative", |_state: &Stack<i32>| true)
        .invariant("empty_has_zero_length", |state: &Stack<i32>| {
            if state.is_empty() {
                state.is_empty()
            } else {
                true
            }
        });

    // Test sequence 1: Push and Pop
    println!("Test 1: Push and Pop sequence");
    let mut seq1 = OperationSequence::new();
    seq1.push(StackOp::Push(10));
    seq1.push(StackOp::Push(20));
    seq1.push(StackOp::Push(30));
    seq1.push(StackOp::Pop);
    seq1.push(StackOp::Pop);

    match test.run(&seq1) {
        Ok(final_state) => {
            println!("  ✓ Test passed!");
            println!("  Final stack length: {}", final_state.len());
            println!("  Final stack: {:?}\n", final_state.items);
        }
        Err(e) => {
            println!("  ✗ Test failed: {}\n", e);
        }
    }

    // Test sequence 2: With trace
    println!("Test 2: Execution trace");
    let mut seq2 = OperationSequence::new();
    seq2.push(StackOp::Push(5));
    seq2.push(StackOp::Push(10));
    seq2.push(StackOp::Peek);
    seq2.push(StackOp::Pop);

    match test.run_with_trace(&seq2) {
        Ok(trace) => {
            println!("  ✓ Test passed with trace:");
            for (i, (op, state)) in trace.steps().iter().enumerate() {
                println!("    Step {}: {} -> {:?}", i + 1, op, state.items);
            }
            println!();
        }
        Err(e) => {
            println!("  ✗ Test failed: {}\n", e);
        }
    }

    // Test sequence 3: Push many items
    println!("Test 3: Push many items");
    let mut seq3 = OperationSequence::new();
    for i in 0..10 {
        seq3.push(StackOp::Push(i));
    }

    match test.run(&seq3) {
        Ok(final_state) => {
            println!("  ✓ Test passed!");
            println!("  Final stack length: {}", final_state.len());
            println!("  Items: {:?}\n", final_state.items);
        }
        Err(e) => {
            println!("  ✗ Test failed: {}\n", e);
        }
    }

    println!("All stack tests completed!");
}
