//! JSON Snapshot Testing Example
//!
//! This example demonstrates how to use property-based testing with JSON snapshots
//! to test serialization of complex data structures.
//!
//! Run with: cargo run --example json_snapshots --features protest/derive

use protest::{
    Generator, config::GeneratorConfig, primitives::IntGenerator, primitives::VecGenerator,
};
use protest_insta::PropertySnapshots;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct UserProfile {
    user_id: i32,
    username: String,
    scores: Vec<i32>,
    total_score: i32,
    is_premium: bool,
}

fn generate_username(id: i32) -> String {
    format!("user_{}", id)
}

fn main() {
    println!("=== JSON Snapshot Testing Example ===\n");

    // Create a seeded RNG for reproducibility
    let mut rng = StdRng::seed_from_u64(42);
    let config = GeneratorConfig::default();

    // Create generators
    let id_generator = IntGenerator::new(1, 1000);
    let score_generator = VecGenerator::new(IntGenerator::new(0, 100), 1, 10);

    // Create snapshot helper
    let mut snapshots = PropertySnapshots::new("user_profiles");

    println!("Generating 5 user profiles and creating JSON snapshots...\n");

    // Generate and snapshot 5 different user profiles
    for i in 0..5 {
        let user_id = id_generator.generate(&mut rng, &config);
        let username = generate_username(user_id);
        let scores = score_generator.generate(&mut rng, &config);
        let total_score: i32 = scores.iter().sum();
        let is_premium = total_score > 250;

        let profile = UserProfile {
            user_id,
            username,
            scores: scores.clone(),
            total_score,
            is_premium,
        };

        println!("Profile {}: {:?}", i, profile);

        // Create a JSON snapshot - this will create files in snapshots/
        snapshots.assert_json_snapshot(&profile);
    }

    println!("\n✅ Created 5 JSON snapshots in snapshots/ directory");
    println!("   Review them with: cargo insta review");
}
