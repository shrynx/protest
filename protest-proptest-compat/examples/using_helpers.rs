//! Example: Using Migration Helper Functions
//!
//! This example demonstrates how to use the migration helper functions
//! provided by protest-proptest-compat.
//!
//! Run with: cargo run --example using_helpers

use protest::primitives::IntGenerator;
use protest::{Generator, config::GeneratorConfig};
use protest_proptest_compat::{
    GeneratorAdapter, option_generator, range_to_generator, vec_generator,
};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    println!("=== Using Protest-Proptest-Compat Helpers ===\n");

    let mut rng = StdRng::seed_from_u64(42);
    let config = GeneratorConfig::default();

    // Example 1: range_to_generator
    println!("1. Range to Generator");
    println!("   Converting proptest range (0..100) to Protest generator:");
    let int_gen = range_to_generator(0, 100);
    let value = int_gen.generate(&mut rng, &config);
    println!("   Generated value: {}", value);

    // Example 2: vec_generator
    println!("\n2. Vector Generator Helper");
    println!("   Creating a Vec<i32> generator with 5-10 elements:");
    let vec_gen = vec_generator(IntGenerator::new(0, 50), 5, 10);
    let vec = vec_gen.generate(&mut rng, &config);
    println!("   Generated vector: {:?}", vec);
    println!("   Length: {}", vec.len());

    // Example 3: option_generator
    println!("\n3. Option Generator Helper");
    println!("   Creating Option<i32> generator with 70% Some probability:");
    let opt_gen = option_generator(IntGenerator::new(0, 100), 0.7);

    let mut some_count = 0;
    let mut none_count = 0;

    for _ in 0..100 {
        match opt_gen.generate(&mut rng, &config) {
            Some(_) => some_count += 1,
            None => none_count += 1,
        }
    }

    println!("   Generated 100 values:");
    println!("   - Some: {} ({}%)", some_count, some_count);
    println!("   - None: {} ({}%)", none_count, none_count);

    // Example 4: GeneratorAdapter
    println!("\n4. Generator Adapter");
    println!("   Wrapping a Protest generator for easier use:");
    let generator = IntGenerator::new(1, 10);
    let adapter = GeneratorAdapter::new(generator);

    println!("   Generated values:");
    for i in 0..5 {
        let value = adapter.generate(&mut rng);
        println!("   {}. {}", i + 1, value);
    }

    println!("\n✅ These helpers make migration from proptest easier!");
    println!("   See protest-proptest-compat README for more patterns.");
}
