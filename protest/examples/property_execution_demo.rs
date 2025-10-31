//! Demonstration of the property execution engine

use protest::primitives::IntGenerator;
use protest::{Generator, Property, PropertyError};
use protest::{PropertyTestBuilder, TestConfig, check, check_with_config};
use std::time::Duration;

// Example property: addition is commutative
struct CommutativeAddition;
impl Property<(i32, i32)> for CommutativeAddition {
    type Output = ();

    fn test(&self, input: (i32, i32)) -> Result<Self::Output, PropertyError> {
        let (a, b) = input;
        if a.wrapping_add(b) == b.wrapping_add(a) {
            Ok(())
        } else {
            Err(PropertyError::property_failed_with_context(
                format!("{} + {} != {} + {}", a, b, b, a),
                Some("commutative property"),
                None,
            ))
        }
    }
}

// Example generator for pairs of integers
struct PairGenerator {
    int_gen: IntGenerator<i32>,
}

impl PairGenerator {
    fn new() -> Self {
        Self {
            int_gen: IntGenerator::new(-100, 100),
        }
    }
}

impl Generator<(i32, i32)> for PairGenerator {
    fn generate(
        &self,
        rng: &mut dyn rand::RngCore,
        config: &protest::GeneratorConfig,
    ) -> (i32, i32) {
        let a = self.int_gen.generate(rng, config);
        let b = self.int_gen.generate(rng, config);
        (a, b)
    }

    fn shrink(&self, value: &(i32, i32)) -> Box<dyn Iterator<Item = (i32, i32)>> {
        let (a, b) = *value;
        let shrinks = vec![(0, b), (a, 0), (0, 0), (a / 2, b), (a, b / 2)];
        Box::new(shrinks.into_iter().filter(move |&(x, y)| (x, y) != (a, b)))
    }
}

fn main() {
    println!("Property Execution Engine Demo");
    println!("==============================");

    // Test 1: Basic property test with default configuration
    println!("\n1. Testing commutative addition with default config:");
    let generator = PairGenerator::new();
    let property = CommutativeAddition;

    match check(generator, property) {
        Ok(success) => {
            println!(
                "✓ Property passed! Completed {} iterations",
                success.iterations
            );
            if let Some(stats) = success.stats {
                println!("  Generated {} test cases", stats.total_generated);
            }
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Original input: {:?}", failure.original_input);
            if let Some(shrunk) = failure.shrunk_input {
                println!("  Shrunk input: {:?}", shrunk);
                println!("  Shrinking steps: {}", failure.shrink_steps);
            }
        }
    }

    // Test 2: Property test with custom configuration
    println!("\n2. Testing with custom configuration (fewer iterations, specific seed):");
    let generator = PairGenerator::new();
    let property = CommutativeAddition;
    let config = TestConfig {
        iterations: 50,
        seed: Some(12345),
        max_shrink_iterations: 100,
        shrink_timeout: Duration::from_secs(5),
        ..TestConfig::default()
    };

    match check_with_config(generator, property, config) {
        Ok(success) => {
            println!(
                "✓ Property passed! Completed {} iterations with seed {:?}",
                success.iterations, success.config.seed
            );
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  {}", failure.summary());
        }
    }

    // Test 3: Using the builder pattern
    println!("\n3. Testing with PropertyTestBuilder:");
    let result = PropertyTestBuilder::new()
        .iterations(25)
        .seed(54321)
        .max_shrink_iterations(50)
        .shrink_timeout(Duration::from_secs(2))
        .run(PairGenerator::new(), CommutativeAddition);

    match result {
        Ok(success) => {
            println!("✓ Property passed! Builder pattern test completed successfully");
            println!("  Iterations: {}", success.iterations);
            println!("  Seed: {:?}", success.config.seed);
        }
        Err(failure) => {
            println!("✗ Property failed: {}", failure.error);
            println!("  Test duration: {:?}", failure.test_duration);
            println!("  Shrink duration: {:?}", failure.shrink_duration);
        }
    }

    println!("\nDemo completed!");
}
