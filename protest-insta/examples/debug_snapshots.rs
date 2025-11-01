//! Debug Snapshot Testing Example
//!
//! This example shows how to use debug snapshots to test computation results
//! and verify output format across different inputs.
//!
//! Run with: cargo run --example debug_snapshots --features protest/derive

use protest::{Generator, config::GeneratorConfig, primitives::IntGenerator};
use protest_insta::PropertySnapshots;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ComputationResult {
    input: i32,
    factorial: u64,
    is_even: bool,
    digits: Vec<u8>,
}

fn factorial(n: i32) -> u64 {
    if n <= 1 { 1 } else { (1..=n as u64).product() }
}

fn get_digits(n: i32) -> Vec<u8> {
    n.abs()
        .to_string()
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect()
}

fn compute(n: i32) -> ComputationResult {
    // Limit factorial to prevent overflow
    let n_clamped = n.clamp(0, 20);

    ComputationResult {
        input: n,
        factorial: factorial(n_clamped),
        is_even: n % 2 == 0,
        digits: get_digits(n),
    }
}

fn main() {
    println!("=== Debug Snapshot Testing Example ===\n");

    // Create a seeded RNG for reproducibility
    let mut rng = StdRng::seed_from_u64(123);
    let config = GeneratorConfig::default();

    // Create generator for small integers
    let generator = IntGenerator::new(1, 15);

    // Create snapshot helper
    let mut snapshots = PropertySnapshots::new("computations");

    println!("Computing results for 8 generated inputs...\n");

    // Generate and snapshot 8 computation results
    for i in 0..8 {
        let input = generator.generate(&mut rng, &config);
        let result = compute(input);

        println!("Computation {}: {:?}", i, result);

        // Create a debug snapshot
        snapshots.assert_debug_snapshot(&result);
    }

    println!("\n✅ Created 8 debug snapshots in snapshots/ directory");
    println!("   Review them with: cargo insta review");
}
