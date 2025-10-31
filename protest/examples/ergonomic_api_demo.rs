//! Demonstration of the Ergonomic API for Protest
//!
//! This example shows how to use the ergonomic API features including:
//! - Closure-based properties
//! - Automatic generator inference
//! - Fluent builder API
//! - Common property patterns

use protest::ergonomic::*;
use protest::execution::check;
use protest::primitives::*;

fn main() {
    println!("Protest Ergonomic API Demonstration");
    println!("====================================\n");

    // Example 1: Closure-based properties
    example_1_closure_properties();

    // Example 2: Fluent builder API
    example_2_fluent_builder();

    // Example 3: Common property patterns
    example_3_common_patterns();

    // Example 4: Composing patterns
    example_4_composed_patterns();

    println!("\n=== Summary ===");
    println!("The ergonomic API dramatically reduces boilerplate!");
    println!("Compare this to the verbose API in basic_usage.rs");
}

fn example_1_closure_properties() {
    println!("=== Example 1: Closure-Based Properties ===\n");

    // Old way: Define a struct implementing Property trait
    // New way: Use closures directly!

    println!("Testing: All positive numbers are non-negative");
    let result = check_with_closure(IntGenerator::new(1, 100), |x: i32| x >= 0);
    match result {
        Ok(success) => println!("✓ Property passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }

    println!("Testing: Absolute value is always non-negative");
    let result = check_with_closure(IntGenerator::new(-100, 100), |x: i32| x.abs() >= 0);
    match result {
        Ok(success) => println!("✓ Property passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }

    println!("Testing: Double reverse equals original (Vec)");
    let result = check_with_closure(
        VecGenerator::new(IntGenerator::new(-10, 10), 0, 20),
        |mut v: Vec<i32>| {
            let original = v.clone();
            v.reverse();
            v.reverse();
            v == original
        },
    );
    match result {
        Ok(success) => println!("✓ Property passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }
}

fn example_2_fluent_builder() {
    println!("=== Example 2: Fluent Builder API ===\n");

    // The builder API allows chaining configuration

    println!("Testing with custom iterations and seed");
    let result = property(|x: i32| x * 2 > x)
        .iterations(50)
        .seed(42)
        .run_with(IntGenerator::new(1, 100));

    match result {
        Ok(success) => {
            println!(
                "✓ Property passed ({} iterations, seed: {:?})\n",
                success.iterations, success.config.seed
            );
        }
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }

    println!("Testing with custom generator and configuration");
    let result = ErgonomicPropertyTest::<String>::new()
        .iterations(30)
        .size_hint(10)
        .run_with(StringGenerator::ascii_alphanumeric(5, 15), |s: String| {
            s.len() >= 5 && s.len() <= 15
        });

    match result {
        Ok(success) => println!("✓ Property passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }
}

fn example_3_common_patterns() {
    println!("=== Example 3: Common Property Patterns ===\n");

    // The library provides helpers for common mathematical properties

    println!("Testing commutativity: a + b == b + a");
    let property = commutative(|a: i32, b: i32| a.wrapping_add(b));
    let result = check(TupleStrategy2::<i32, i32>::new(), property);
    match result {
        Ok(success) => println!(
            "✓ Addition is commutative ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }

    println!("Testing idempotence: abs(abs(x)) == abs(x)");
    let property = idempotent(|x: i32| x.abs());
    let result = check(IntGenerator::new(-100, 100), property);
    match result {
        Ok(success) => println!(
            "✓ Absolute value is idempotent ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }

    println!("Testing round-trip: parse(to_string(x)) == x");
    let property = round_trip(
        |x: i32| x.to_string(),
        |s: String| s.parse::<i32>().unwrap(),
    );
    let result = check(IntGenerator::new(-1000, 1000), property);
    match result {
        Ok(success) => println!(
            "✓ String conversion is round-trippable ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }

    println!("Testing identity: x + 0 == x");
    let property = has_identity(|a: i32, b: i32| a.wrapping_add(b), 0);
    let result = check(IntGenerator::new(-100, 100), property);
    match result {
        Ok(success) => println!(
            "✓ 0 is the additive identity ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Property failed: {}\n", failure.error),
    }
}

fn example_4_composed_patterns() {
    println!("=== Example 4: Composing Multiple Properties ===\n");

    // You can easily test multiple related properties

    println!("Testing vector operations:");

    // Property 1: Sorting doesn't change length
    println!("  1. Sorting preserves length");
    let result = check_with_closure(
        VecGenerator::new(IntGenerator::new(-50, 50), 0, 30),
        |v: Vec<i32>| {
            let original_len = v.len();
            let mut sorted = v.clone();
            sorted.sort();
            sorted.len() == original_len
        },
    );
    match result {
        Ok(_) => println!("     ✓ Length preserved"),
        Err(_) => println!("     ✗ Failed"),
    }

    // Property 2: Sorting is idempotent
    println!("  2. Sorting is idempotent");
    let result = check_with_closure(
        VecGenerator::new(IntGenerator::new(-50, 50), 0, 30),
        |v: Vec<i32>| {
            let mut once = v.clone();
            once.sort();
            let mut twice = once.clone();
            twice.sort();
            once == twice
        },
    );
    match result {
        Ok(_) => println!("     ✓ Idempotent"),
        Err(_) => println!("     ✗ Failed"),
    }

    // Property 3: Reversing twice gives original (this is an inverse, not idempotent!)
    println!("  3. Double reverse equals original");
    let property = inverse(
        |mut v: Vec<i32>| {
            v.reverse();
            v
        },
        |mut v: Vec<i32>| {
            v.reverse();
            v
        },
    );
    let result = check(
        VecGenerator::new(IntGenerator::new(-50, 50), 0, 30),
        property,
    );
    match result {
        Ok(_) => println!("     ✓ Double reverse works"),
        Err(_) => println!("     ✗ Failed"),
    }

    println!();
}
