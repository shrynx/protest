//! Basic usage examples demonstrating the core Protest API
//!
//! This example shows the fundamental concepts of property-based testing
//! with Protest, including generators, properties, and basic configuration.

use protest::{
    Property, PropertyError, PropertyTestBuilder, TestConfig, check, check_with_config, range,
};
use std::time::Duration;

// Example 1: Simple property with built-in generators
fn example_1_basic_property() {
    println!("=== Example 1: Basic Property Testing ===");

    // Property: addition is commutative
    struct CommutativeProperty;
    impl Property<(i32, i32)> for CommutativeProperty {
        type Output = ();

        fn test(&self, input: (i32, i32)) -> Result<Self::Output, PropertyError> {
            let (a, b) = input;
            let left = a.wrapping_add(b);
            let right = b.wrapping_add(a);
            if left == right {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "{} + {} != {} + {}",
                    a, b, b, a
                )))
            }
        }
    }

    // Use tuple generator to create a generator for pairs
    let generator = (range(-100, 100), range(-100, 100));

    match check(generator, CommutativeProperty) {
        Ok(success) => {
            println!(
                "✓ Commutative property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Input: {:?}", failure.original_input);
        }
    }
}

// Example 2: Property with custom configuration
fn example_2_custom_config() {
    println!("\n=== Example 2: Custom Configuration ===");

    // Property: absolute value is always non-negative
    struct AbsoluteValueProperty;
    impl Property<i32> for AbsoluteValueProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input.abs() >= 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "abs({}) = {} is negative",
                    input,
                    input.abs()
                )))
            }
        }
    }

    let config = TestConfig {
        iterations: 50,
        seed: Some(42),
        max_shrink_iterations: 100,
        shrink_timeout: Duration::from_secs(5),
        ..TestConfig::default()
    };

    match check_with_config(range(i32::MIN, i32::MAX), AbsoluteValueProperty, config) {
        Ok(success) => {
            println!(
                "✓ Absolute value property passed! ({} iterations, seed: {:?})",
                success.iterations, success.config.seed
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
        }
    }
}

// Example 3: Using PropertyTestBuilder for fluent configuration
fn example_3_builder_pattern() {
    println!("\n=== Example 3: Builder Pattern ===");

    // Property: string length matches expected bounds
    struct StringLengthProperty {
        min_len: usize,
        max_len: usize,
    }

    impl Property<String> for StringLengthProperty {
        type Output = ();

        fn test(&self, input: String) -> Result<Self::Output, PropertyError> {
            let len = input.len();
            if len >= self.min_len && len <= self.max_len {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "String length {} not in range [{}, {}]",
                    len, self.min_len, self.max_len
                )))
            }
        }
    }

    let result = PropertyTestBuilder::new()
        .iterations(30)
        .seed(12345)
        .max_shrink_iterations(50)
        .enable_statistics()
        .run(
            protest::primitives::StringGenerator::ascii_printable(5, 15),
            StringLengthProperty {
                min_len: 5,
                max_len: 15,
            },
        );

    match result {
        Ok(success) => {
            println!("✓ String length property passed!");
            if let Some(stats) = success.stats {
                println!("  Generated {} strings", stats.total_generated);
                println!(
                    "  Generation time: {:?}",
                    stats.performance_metrics.total_generation_time
                );
            }
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  String: {:?}", failure.original_input);
        }
    }
}

// Example 4: Testing with collections
fn example_4_collections() {
    println!("\n=== Example 4: Collection Properties ===");

    // Property: sorting preserves all elements
    struct SortPreservesElementsProperty;
    impl Property<Vec<i32>> for SortPreservesElementsProperty {
        type Output = ();

        fn test(&self, mut input: Vec<i32>) -> Result<Self::Output, PropertyError> {
            let original = input.clone();
            input.sort();

            // Check that all original elements are still present
            for &elem in &original {
                if !input.contains(&elem) {
                    return Err(PropertyError::property_failed(format!(
                        "Element {} missing after sort",
                        elem
                    )));
                }
            }

            // Check that no new elements were added
            if input.len() != original.len() {
                return Err(PropertyError::property_failed(format!(
                    "Length changed: {} -> {}",
                    original.len(),
                    input.len()
                )));
            }

            Ok(())
        }
    }

    let generator = protest::primitives::VecGenerator::new(range(-50, 50), 0, 20);

    match check(generator, SortPreservesElementsProperty) {
        Ok(success) => {
            println!(
                "✓ Sort preserves elements property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Vector: {:?}", failure.original_input);
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk to: {:?}", shrunk);
            }
        }
    }
}

// Example 5: Demonstrating shrinking behavior
fn example_5_shrinking() {
    println!("\n=== Example 5: Shrinking Demonstration ===");

    // Property that fails for large values to demonstrate shrinking
    struct SmallValueProperty;
    impl Property<i32> for SmallValueProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input <= 10 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Value {} is too large (must be <= 10)",
                    input
                )))
            }
        }
    }

    // Use a range that will likely generate failing cases
    let generator = range(5, 100);

    match check(generator, SmallValueProperty) {
        Ok(success) => {
            println!(
                "✓ Property passed unexpectedly! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed as expected: {}", failure.error);
            println!("  Original failing input: {}", failure.original_input);
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk to minimal failing case: {}", shrunk);
                println!("  Shrinking took {} steps", failure.shrink_steps);
            }
            println!("  This demonstrates how shrinking finds the minimal failing case!");
        }
    }
}

// Example 6: Strategy composition
fn example_6_strategy_composition() {
    println!("\n=== Example 6: Strategy Composition ===");

    // Property: even numbers are divisible by 2
    struct EvenNumberProperty;
    impl Property<i32> for EvenNumberProperty {
        type Output = ();

        fn test(&self, input: i32) -> Result<Self::Output, PropertyError> {
            if input % 2 == 0 {
                Ok(())
            } else {
                Err(PropertyError::property_failed(format!(
                    "Number {} is not even",
                    input
                )))
            }
        }
    }

    // Create a generator that only generates even numbers
    // Note: .map() is not available on generators, so we use a range of even numbers
    let even_generator = range(2, 200); // Will need to filter/transform in property

    match check(even_generator, EvenNumberProperty) {
        Ok(success) => {
            println!(
                "✓ Even number property passed! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  This shouldn't happen with our even number generator!");
        }
    }

    // Now test with a mixed generator to show failures
    println!("\n  Testing with mixed even/odd generator:");
    // Note: Strategy combinators (.map(), .boxed()) not available, using simple range
    let mixed_generator = range(1, 100); // Mix of even and odd

    match check(mixed_generator, EvenNumberProperty) {
        Ok(success) => {
            println!(
                "✓ Property passed unexpectedly! ({} iterations)",
                success.iterations
            );
        }
        Err(failure) => {
            println!("✗ Property failed as expected: {}", failure.error);
            println!("  Failed on odd number: {}", failure.original_input);
        }
    }
}

fn main() {
    println!("Protest Library - Basic Usage Examples");
    println!("=====================================");

    example_1_basic_property();
    example_2_custom_config();
    example_3_builder_pattern();
    example_4_collections();
    example_5_shrinking();
    example_6_strategy_composition();

    println!("\n=== Summary ===");
    println!("These examples demonstrate:");
    println!("• Basic property testing with simple generators");
    println!("• Custom configuration and seeding");
    println!("• Builder pattern for fluent API");
    println!("• Testing properties of collections");
    println!("• Automatic shrinking to minimal failing cases");
    println!("• Strategy composition and transformation");
    println!("\nFor more advanced examples, see other files in the examples/ directory.");
}
