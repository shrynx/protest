//! Advanced property testing patterns and techniques
//!
//! This example demonstrates sophisticated testing patterns:
//! - Stateful property testing
//! - Model-based testing
//! - Combining multiple properties
//! - Conditional properties

use protest::ergonomic::*;
use protest::execution::check;
use protest::primitives::*;
use std::collections::HashMap;

fn main() {
    println!("Protest - Advanced Patterns Example");
    println!("====================================\n");

    example_1_stateful_testing();
    example_2_model_based_testing();
    example_3_combining_properties();
    example_4_conditional_properties();

    println!("\n=== Summary ===");
    println!("✓ All advanced examples completed!");
}

// Example 1: Stateful Property Testing
fn example_1_stateful_testing() {
    println!("=== Example 1: Stateful Property Testing ===\n");

    println!("Testing: Stack push/pop maintains LIFO order");

    let result = check_with_closure(
        VecGenerator::new(IntGenerator::new(1, 100), 1, 20),
        |items: Vec<i32>| {
            let mut stack = Vec::new();

            // Push all items
            for &item in &items {
                stack.push(item);
            }

            // Pop and verify LIFO order
            let mut popped = Vec::new();
            while let Some(item) = stack.pop() {
                popped.push(item);
            }

            // Reversed items should equal original
            popped.reverse();
            popped == items
        },
    );

    match result {
        Ok(success) => println!(
            "✓ LIFO property passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: HashMap preserves inserted values");

    let result = check_with_closure(
        VecGenerator::new(TupleStrategy2::<i32, String>::new(), 0, 20),
        |pairs: Vec<(i32, String)>| {
            let mut map: HashMap<i32, String> = HashMap::new();

            // Insert all pairs
            for (k, v) in pairs.iter() {
                map.insert(*k, v.clone());
            }

            // Verify all unique keys can be retrieved
            let unique_pairs: Vec<_> = {
                let mut seen = HashMap::new();
                for (k, v) in pairs.iter().rev() {
                    seen.entry(*k).or_insert(v.clone());
                }
                seen.into_iter().collect()
            };

            unique_pairs.iter().all(|(k, v)| map.get(k) == Some(v))
        },
    );

    match result {
        Ok(success) => println!(
            "✓ HashMap property passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }
}

// Example 2: Model-Based Testing
fn example_2_model_based_testing() {
    println!("=== Example 2: Model-Based Testing ===\n");

    println!("Testing: Vec behaves like a simple list model");

    let result = check_with_closure(
        VecGenerator::new(IntGenerator::new(-50, 50), 0, 30),
        |items: Vec<i32>| {
            let vec = items.clone();

            // Property 1: Length matches
            if vec.len() != items.len() {
                return false;
            }

            // Property 2: Indexing matches
            for (i, &item) in items.iter().enumerate() {
                if vec[i] != item {
                    return false;
                }
            }

            // Property 3: Iteration matches
            let collected: Vec<_> = vec.to_vec();
            if collected != items {
                return false;
            }

            true
        },
    );

    match result {
        Ok(success) => println!(
            "✓ Model-based test passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: String concatenation model");

    let result = check_with_closure(
        TupleStrategy2::<String, String>::new(),
        |(a, b): (String, String)| {
            let concatenated = format!("{}{}", a, b);

            // Model properties
            concatenated.len() == a.len() + b.len()
                && concatenated.starts_with(&a)
                && concatenated.ends_with(&b)
        },
    );

    match result {
        Ok(success) => println!(
            "✓ String model passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }
}

// Example 3: Combining Multiple Properties
fn example_3_combining_properties() {
    println!("=== Example 3: Combining Multiple Properties ===\n");

    println!("Testing: Vector operations maintain multiple invariants");

    let result = check_with_closure(
        VecGenerator::new(IntGenerator::new(-100, 100), 0, 50),
        |v: Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();

            // Combine multiple properties
            let length_preserved = sorted.len() == v.len();
            let is_sorted = sorted.windows(2).all(|w| w[0] <= w[1]);
            let contains_same_elements = {
                let mut v_sorted = v.clone();
                v_sorted.sort();
                v_sorted == sorted
            };

            length_preserved && is_sorted && contains_same_elements
        },
    );

    match result {
        Ok(success) => println!(
            "✓ Combined properties passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: Using property pattern helpers");

    // Test idempotence of sorting
    let idempotent_sort = idempotent(|mut v: Vec<i32>| {
        v.sort();
        v
    });

    let result = check(
        VecGenerator::new(IntGenerator::new(-50, 50), 0, 30),
        idempotent_sort,
    );

    match result {
        Ok(success) => println!(
            "✓ Idempotent sort passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }
}

// Example 4: Conditional Properties
fn example_4_conditional_properties() {
    println!("=== Example 4: Conditional Properties ===\n");

    println!("Testing: Division properties (when divisor != 0)");

    let result = check_with_closure(TupleStrategy2::<i32, i32>::new(), |(a, b): (i32, i32)| {
        if b == 0 {
            // Skip this test case
            return true;
        }

        // Property only applies when b != 0
        let quotient = a / b;
        let remainder = a % b;

        // Verify: a == b * quotient + remainder
        a == b.wrapping_mul(quotient).wrapping_add(remainder)
    });

    match result {
        Ok(success) => println!(
            "✓ Division property passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: Square root properties (for non-negative numbers)");

    let result = check_with_closure(IntGenerator::new(0, 10000), |x: i32| {
        if x < 0 {
            return true; // Skip negative numbers
        }

        let sqrt = (x as f64).sqrt();
        let squared = (sqrt * sqrt) as i32;

        // Allow small rounding error
        (squared - x).abs() <= 1
    });

    match result {
        Ok(success) => println!(
            "✓ Square root property passed ({} iterations)\n",
            success.iterations
        ),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protest::execution::check;

    #[test]
    fn test_stack_lifo() {
        let result = check_with_closure(
            VecGenerator::new(IntGenerator::new(1, 50), 1, 10),
            |items: Vec<i32>| {
                let mut stack = Vec::new();
                for &item in &items {
                    stack.push(item);
                }
                let mut popped = Vec::new();
                while let Some(item) = stack.pop() {
                    popped.push(item);
                }
                popped.reverse();
                popped == items
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_idempotent() {
        let idempotent_sort = idempotent(|mut v: Vec<i32>| {
            v.sort();
            v
        });

        let result = check(
            VecGenerator::new(IntGenerator::new(-50, 50), 0, 20),
            idempotent_sort,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_vector_length_preserved() {
        let result = check_with_closure(
            VecGenerator::new(IntGenerator::new(-100, 100), 0, 30),
            |v: Vec<i32>| {
                let mut sorted = v.clone();
                sorted.sort();
                sorted.len() == v.len()
            },
        );
        assert!(result.is_ok());
    }
}
