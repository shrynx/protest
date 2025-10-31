//! Simple example: Vector reverse property
//!
//! This example demonstrates a classic property-based test:
//! reversing a vector twice should return the original vector.

use protest::{
    Property, PropertyError, PropertyTestBuilder, check,
    primitives::{IntGenerator, VecGenerator},
};

// Property: reversing a vector twice returns the original
struct DoubleReverseProperty;

impl Property<Vec<i32>> for DoubleReverseProperty {
    type Output = ();

    fn test(&self, input: Vec<i32>) -> Result<Self::Output, PropertyError> {
        let original = input.clone();
        let mut reversed_once = input;

        // Reverse once
        reversed_once.reverse();

        // Reverse again
        reversed_once.reverse();

        // Should be back to original
        if reversed_once == original {
            Ok(())
        } else {
            Err(PropertyError::property_failed(format!(
                "Double reverse failed: {:?} != {:?}",
                reversed_once, original
            )))
        }
    }
}

fn main() {
    println!("Testing: reverse(reverse(vec)) == vec");
    println!("=====================================");

    // Create a generator for vectors of integers
    let generator = VecGenerator::new(
        IntGenerator::new(-100, 100), // Generate integers from -100 to 100
        0,                            // Minimum vector length
        20,                           // Maximum vector length
    );

    // Run the property test
    match check(generator, DoubleReverseProperty) {
        Ok(success) => {
            println!("✓ Property passed! Tested {} vectors", success.iterations);
            println!("  All vectors satisfied: reverse(reverse(v)) == v");
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Original vector: {:?}", failure.original_input);
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Minimal failing case: {:?}", shrunk);
            }
        }
    }

    // Let's also test with more iterations and statistics
    println!("\nRunning extended test with statistics...");

    let result = PropertyTestBuilder::new()
        .iterations(1000)
        .enable_statistics()
        .run(
            VecGenerator::new(IntGenerator::new(-1000, 1000), 0, 50),
            DoubleReverseProperty,
        );

    match result {
        Ok(success) => {
            println!("✓ Extended test passed! {} iterations", success.iterations);

            if let Some(stats) = success.stats {
                println!("  Generated {} vectors total", stats.total_generated);
                println!(
                    "  Average generation time: {:?}",
                    stats.performance_metrics.average_generation_time
                );

                // Show some collection statistics
                if let Some(collection_stats) =
                    stats.coverage_info.collection_coverage.values().next()
                {
                    println!("  Vector size statistics:");
                    println!("    Average size: {:.1}", collection_stats.average_size);
                    println!(
                        "    Size range: [{}, {}]",
                        collection_stats.min_size, collection_stats.max_size
                    );
                }
            }
        }
        Err(failure) => {
            println!("✗ Extended test failed: {}", failure.error);
        }
    }

    // Test edge cases explicitly
    println!("\nTesting specific edge cases...");

    // Empty vector
    test_specific_case(vec![], "empty vector");

    // Single element
    test_specific_case(vec![42], "single element");

    // Two elements
    test_specific_case(vec![1, 2], "two elements");

    // Palindrome (should be same when reversed)
    test_specific_case(vec![1, 2, 3, 2, 1], "palindrome");

    println!("\n=== Summary ===");
    println!("The property 'reverse(reverse(v)) == v' should always hold for any vector.");
    println!("This is because reversing is its own inverse operation.");
    println!("Property-based testing helps us verify this across many random cases!");
}

fn test_specific_case(vec: Vec<i32>, description: &str) {
    let original = vec.clone();
    let mut test_vec = vec;

    test_vec.reverse();
    test_vec.reverse();

    if test_vec == original {
        println!("  ✓ {} case passed", description);
    } else {
        println!(
            "  ✗ {} case failed: {:?} != {:?}",
            description, test_vec, original
        );
    }
}
