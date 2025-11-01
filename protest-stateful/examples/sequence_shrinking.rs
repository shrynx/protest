//! Advanced Sequence Shrinking Example
//!
//! This example demonstrates the two advanced shrinking strategies for operation sequences:
//! 1. DeltaDebugSequenceShrinker - Binary search for minimal failing subsequences
//! 2. SmartSequenceShrinking - Shrink while preserving invariants and preconditions
//!
//! Run with: cargo run --example sequence_shrinking

use protest_stateful::operations::shrinking::{DeltaDebugSequenceShrinker, SmartSequenceShrinking};
use protest_stateful::prelude::*;

fn main() {
    println!("=== Advanced Sequence Shrinking Demo ===\n");

    example_1_delta_debug_basic();
    println!("\n{}\n", "=".repeat(70));

    example_2_delta_debug_complex();
    println!("\n{}\n", "=".repeat(70));

    example_3_smart_shrinking_with_preconditions();
    println!("\n{}\n", "=".repeat(70));

    example_4_comparing_strategies();
}

// Simple counter for examples
#[derive(Debug, Clone)]
struct Counter {
    value: i32,
}

#[derive(Debug, Clone)]
enum CounterOp {
    Increment,
    Decrement,
    Add(i32),
    Reset,
}

impl Operation for CounterOp {
    type State = Counter;

    fn execute(&self, state: &mut Self::State) {
        match self {
            CounterOp::Increment => state.value += 1,
            CounterOp::Decrement => state.value -= 1,
            CounterOp::Add(n) => state.value += n,
            CounterOp::Reset => state.value = 0,
        }
    }

    fn precondition(&self, state: &Self::State) -> bool {
        match self {
            CounterOp::Decrement => state.value > 0, // Can't go negative
            _ => true,
        }
    }
}

/// Example 1: Basic delta debugging
fn example_1_delta_debug_basic() {
    println!("Example 1: DeltaDebugSequenceShrinker - Basic Usage");
    println!("{}", "-".repeat(70));

    // Create a long sequence that eventually violates an invariant
    let mut seq = OperationSequence::new();
    for _ in 0..15 {
        seq.push(CounterOp::Increment);
    }

    println!(
        "Original sequence: {} operations (15 increments)",
        seq.len()
    );

    // Create a test with invariant: value < 10
    let test: StatefulTest<Counter, CounterOp> = StatefulTest::new(Counter { value: 0 })
        .invariant("less_than_10", |s: &Counter| s.value < 10);

    // Use delta debugging to find minimal failing sequence
    let shrinker = DeltaDebugSequenceShrinker::new(seq);
    let (minimal, test_count) =
        shrinker.minimize_with_stats(|sequence| test.run(sequence).is_err());

    println!("\nMinimal failing sequence: {} operations", minimal.len());
    println!("Test function called: {} times", test_count);

    // Verify the minimal sequence
    let mut state = Counter { value: 0 };
    for op in minimal.operations() {
        op.execute(&mut state);
    }
    println!("Final value: {} (violates invariant < 10)", state.value);

    println!("\nWhy this is efficient:");
    println!("  - Brute force would try 2^15 = 32,768 combinations");
    println!(
        "  - Delta debugging found it in {} tests (O(n log n))",
        test_count
    );
}

/// Example 2: Delta debugging with complex sequence
fn example_2_delta_debug_complex() {
    println!("Example 2: DeltaDebugSequenceShrinker - Complex Sequence");
    println!("{}", "-".repeat(70));

    // Create a complex sequence with different operations
    let mut seq = OperationSequence::new();
    seq.push(CounterOp::Add(3));
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Add(2));
    seq.push(CounterOp::Decrement);
    seq.push(CounterOp::Add(15)); // This alone violates the invariant
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Decrement);

    println!("Original sequence:");
    for (i, op) in seq.operations().iter().enumerate() {
        println!("  {}: {:?}", i, op);
    }

    let test: StatefulTest<Counter, CounterOp> = StatefulTest::new(Counter { value: 0 })
        .invariant("less_than_12", |s: &Counter| s.value < 12);

    let shrinker = DeltaDebugSequenceShrinker::new(seq);
    let (minimal, test_count) =
        shrinker.minimize_with_stats(|sequence| test.run(sequence).is_err());

    println!("\nMinimal failing sequence:");
    for (i, op) in minimal.operations().iter().enumerate() {
        println!("  {}: {:?}", i, op);
    }
    println!(
        "\nReduced from {} to {} operations in {} tests",
        8,
        minimal.len(),
        test_count
    );

    // The minimal should be just Add(15) since that alone violates the invariant
    println!("\nDelta debugging found the single culprit operation!");
}

/// Example 3: Smart shrinking with preconditions
fn example_3_smart_shrinking_with_preconditions() {
    println!("Example 3: SmartSequenceShrinking - Preserving Preconditions");
    println!("{}", "-".repeat(70));

    // Create a sequence with decrements that depend on prior increments
    let mut seq = OperationSequence::new();
    seq.push(CounterOp::Increment); // 1
    seq.push(CounterOp::Increment); // 2
    seq.push(CounterOp::Increment); // 3
    seq.push(CounterOp::Increment); // 4
    seq.push(CounterOp::Increment); // 5
    seq.push(CounterOp::Decrement); // 4
    seq.push(CounterOp::Decrement); // 3
    seq.push(CounterOp::Increment); // 4
    seq.push(CounterOp::Increment); // 5

    println!("Original sequence:");
    let mut value = 0;
    for (i, op) in seq.operations().iter().enumerate() {
        match op {
            CounterOp::Increment => value += 1,
            CounterOp::Decrement => value -= 1,
            _ => {}
        }
        println!("  {}: {:?} -> value = {}", i, op, value);
    }

    let initial = Counter { value: 0 };

    // Shrink to find sequences where value >= 4
    let config = SmartSequenceShrinking::new()
        .preserve_preconditions(true) // Ensures Decrement only when value > 0
        .max_attempts(100);

    println!("\nShrinking with precondition preservation...");
    let (minimal, attempts) = config.shrink_with_stats(&seq, &initial, |sequence| {
        let mut state = initial.clone();
        for op in sequence.operations() {
            op.execute(&mut state);
        }
        state.value >= 4
    });

    println!("\nMinimal sequence:");
    let mut value = 0;
    for (i, op) in minimal.operations().iter().enumerate() {
        match op {
            CounterOp::Increment => value += 1,
            CounterOp::Decrement => value -= 1,
            _ => {}
        }
        println!("  {}: {:?} -> value = {}", i, op, value);
    }

    println!(
        "\nReduced from {} to {} operations",
        seq.len(),
        minimal.len()
    );
    println!("Shrinking attempts: {}", attempts);

    // Verify all preconditions are satisfied
    let mut state = initial.clone();
    println!("\nVerifying preconditions...");
    for op in minimal.operations() {
        assert!(op.precondition(&state), "Precondition violated!");
        op.execute(&mut state);
    }
    println!("✓ All preconditions satisfied!");
}

/// Example 4: Comparing both strategies
fn example_4_comparing_strategies() {
    println!("Example 4: Strategy Comparison");
    println!("{}", "-".repeat(70));

    // Create a moderate-sized sequence
    let mut seq = OperationSequence::new();
    for i in 0..12 {
        if i % 3 == 0 {
            seq.push(CounterOp::Add(2));
        } else {
            seq.push(CounterOp::Increment);
        }
    }

    println!("Testing sequence with {} operations", seq.len());

    let test: StatefulTest<Counter, CounterOp> =
        StatefulTest::new(Counter { value: 0 }).invariant("less_than_8", |s: &Counter| s.value < 8);

    // Strategy 1: Delta Debugging
    println!("\n1. DeltaDebugSequenceShrinker:");
    let delta_shrinker = DeltaDebugSequenceShrinker::new(seq.clone());
    let (delta_minimal, delta_tests) =
        delta_shrinker.minimize_with_stats(|sequence| test.run(sequence).is_err());

    println!("   Result: {} operations", delta_minimal.len());
    println!("   Tests performed: {}", delta_tests);

    // Verify result
    let mut state = Counter { value: 0 };
    for op in delta_minimal.operations() {
        op.execute(&mut state);
    }
    println!("   Final value: {}", state.value);

    // Strategy 2: Smart Shrinking
    println!("\n2. SmartSequenceShrinking:");
    let smart_config = SmartSequenceShrinking::new();
    let initial = Counter { value: 0 };
    let (smart_minimal, smart_attempts) =
        smart_config.shrink_with_stats(&seq, &initial, |sequence| test.run(sequence).is_err());

    println!("   Result: {} operations", smart_minimal.len());
    println!("   Attempts: {}", smart_attempts);

    // Verify result
    let mut state = initial.clone();
    for op in smart_minimal.operations() {
        op.execute(&mut state);
    }
    println!("   Final value: {}", state.value);

    println!("\n{}", "=".repeat(70));
    println!("Summary:");
    println!("  - Delta Debugging: Fast, binary search approach, O(n log n)");
    println!("  - Smart Shrinking: Iterative, respects constraints, configurable");
    println!("\nChoose based on:");
    println!("  - Delta Debug: When you want the fastest minimization");
    println!("  - Smart Shrinking: When you need fine control over constraints");
}
