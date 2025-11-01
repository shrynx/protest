//! Property Snapshot Test Function Example
//!
//! This example demonstrates using the `property_snapshot_test` helper function
//! for concise property-based snapshot testing.
//!
//! Run with: cargo run --example property_snapshot_test --features protest/derive

use protest::primitives::IntGenerator;
use protest_insta::property_snapshot_test;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct MathOperation {
    input: i32,
    squared: i32,
    cubed: i64,
    doubled: i32,
    is_positive: bool,
}

fn perform_operations(n: i32) -> MathOperation {
    MathOperation {
        input: n,
        squared: n * n,
        cubed: (n as i64) * (n as i64) * (n as i64),
        doubled: n * 2,
        is_positive: n > 0,
    }
}

fn main() {
    println!("=== Property Snapshot Test Function Example ===\n");
    println!("Testing mathematical operations with property-based inputs...\n");

    // Use the helper function for concise testing
    property_snapshot_test(
        "math_operations",
        IntGenerator::new(-10, 10),
        10,  // Generate 10 test cases
        456, // Seed for reproducibility
        |value, snapshots| {
            let result = perform_operations(value);
            println!("Operation on {}: {:?}", value, result);
            snapshots.assert_json_snapshot(&result);
        },
    );

    println!("\n✅ Created 10 snapshots in snapshots/ directory");
    println!("   Review them with: cargo insta review");
    println!("\nNote: This example uses the property_snapshot_test helper");
    println!("      which combines generator setup with snapshot management.");
}
