//! Advanced Shrinking Strategies Example
//!
//! This example demonstrates the three advanced shrinking strategies in protest-extras:
//! 1. CascadingShrinker - Applies multiple strategies in sequence
//! 2. GuidedShrinker - Uses test feedback to guide minimization
//! 3. ConfigurableShrinker - Supports breadth-first and depth-first search
//!
//! Run with: cargo run --example advanced_shrinking

use protest_extras::shrinking::{
    CascadingShrinker, ConfigurableShrinker, GuidedShrinker, ShrinkStrategy,
};

fn main() {
    println!("=== Advanced Shrinking Strategies Demo ===\n");

    example_1_cascading_shrinker();
    println!("\n{}\n", "=".repeat(60));

    example_2_guided_shrinker();
    println!("\n{}\n", "=".repeat(60));

    example_3_configurable_shrinker_dfs();
    println!("\n{}\n", "=".repeat(60));

    example_4_configurable_shrinker_bfs();
    println!("\n{}\n", "=".repeat(60));

    example_5_comparing_strategies();
}

/// Example 1: CascadingShrinker for thorough exploration
fn example_1_cascading_shrinker() {
    println!("Example 1: CascadingShrinker - Thorough Exploration");
    println!("{}", "-".repeat(60));

    let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Original vector: {:?}", original);
    println!("Length: {}", original.len());

    let shrinker = CascadingShrinker::new(original.clone());
    let candidates: Vec<_> = shrinker.shrink().collect();

    println!(
        "\nCascading shrinking generates {} candidates:",
        candidates.len()
    );
    println!("  - Single element removals: {}", original.len());
    println!("  - Chunk removals (halves, thirds): ~6");
    println!("  - Empty vector: 1");
    println!(
        "  - Adjacent pair removals: {}",
        original.len().saturating_sub(1)
    );

    // Show some interesting candidates
    println!("\nSample candidates:");
    for (i, candidate) in candidates.iter().take(5).enumerate() {
        println!("  {}: {:?} (len={})", i + 1, candidate, candidate.len());
    }

    // The cascading shrinker is useful when you want to explore many
    // different ways to shrink a value without knowing which will be most effective
    println!("\nUse Case:");
    println!("  Best for: Exploring all shrinking possibilities systematically");
    println!("  When to use: When you don't know what minimal form looks like");
}

/// Example 2: GuidedShrinker with test feedback
fn example_2_guided_shrinker() {
    println!("Example 2: GuidedShrinker - Test Feedback-Driven");
    println!("{}", "-".repeat(60));

    let original = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    println!("Original vector: {:?}", original);
    println!("Sum: {}", original.iter().sum::<i32>());

    // Find minimal subset that still sums to > 200
    let shrinker = GuidedShrinker::new(original.clone());

    println!("\nFinding minimal subset where sum > 200...");
    let (minimal, iterations) = shrinker.find_minimal_with_stats(|v| v.iter().sum::<i32>() > 200);

    println!("Minimal vector: {:?}", minimal);
    println!("Sum: {}", minimal.iter().sum::<i32>());
    println!(
        "Reduced from {} to {} elements",
        original.len(),
        minimal.len()
    );
    println!("Iterations used: {}", iterations);

    // Another example: find minimal subset containing a specific element
    let original2 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let shrinker2 = GuidedShrinker::new(original2.clone());

    println!("\nFinding minimal subset containing element 7...");
    let (minimal2, iterations2) =
        shrinker2.find_minimal_with_stats(|v| v.contains(&7) && !v.is_empty());

    println!("Minimal vector: {:?}", minimal2);
    println!("Iterations used: {}", iterations2);

    println!("\nUse Case:");
    println!("  Best for: Efficiently finding minimal failing examples");
    println!(
        "  When to use: When you can run the test many times and want the smallest counterexample"
    );
}

/// Example 3: ConfigurableShrinker with Depth-First Search
fn example_3_configurable_shrinker_dfs() {
    println!("Example 3: ConfigurableShrinker - Depth-First Search");
    println!("{}", "-".repeat(60));

    let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Original vector: {:?}", original);

    let shrinker =
        ConfigurableShrinker::new(original.clone(), ShrinkStrategy::DepthFirst).with_max_depth(20);

    println!("\nFinding minimal subset with length >= 3 (DFS)...");
    let minimal = shrinker.find_minimal(|v| v.len() >= 3);

    println!("Minimal vector: {:?}", minimal);
    println!("Length: {}", minimal.len());

    println!("\nDepth-First Strategy:");
    println!("  - Follows the first successful shrink immediately");
    println!("  - Goes deep before exploring alternatives");
    println!("  - Fast but may not find the absolute minimal");
    println!("  - Good when any reasonably small counterexample is sufficient");
}

/// Example 4: ConfigurableShrinker with Breadth-First Search
fn example_4_configurable_shrinker_bfs() {
    println!("Example 4: ConfigurableShrinker - Breadth-First Search");
    println!("{}", "-".repeat(60));

    let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Original vector: {:?}", original);

    let shrinker = ConfigurableShrinker::new(original.clone(), ShrinkStrategy::BreadthFirst)
        .with_max_depth(20);

    println!("\nFinding minimal subset with length >= 3 (BFS)...");
    let minimal = shrinker.find_minimal(|v| v.len() >= 3);

    println!("Minimal vector: {:?}", minimal);
    println!("Length: {}", minimal.len());

    println!("\nBreadth-First Strategy:");
    println!("  - Explores all shrinks at current depth before going deeper");
    println!("  - More thorough exploration");
    println!("  - Finds the truly minimal counterexample");
    println!("  - Slower but more comprehensive");
}

/// Example 5: Comparing all strategies on the same problem
fn example_5_comparing_strategies() {
    println!("Example 5: Strategy Comparison");
    println!("{}", "-".repeat(60));

    // Problem: Find minimal subset where sum of squares > 500
    let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let sum_of_squares = |v: &Vec<i32>| v.iter().map(|x| x * x).sum::<i32>();

    println!("Original vector: {:?}", original);
    println!("Sum of squares: {}", sum_of_squares(&original));
    println!("\nFinding minimal subset where sum of squares > 500\n");

    // Strategy 1: Cascading (manual exploration)
    let cascading = CascadingShrinker::new(original.clone());
    let candidates: Vec<_> = cascading.shrink().collect();
    let cascading_result = candidates
        .iter()
        .filter(|v| sum_of_squares(v) > 500)
        .min_by_key(|v| v.len())
        .cloned();

    if let Some(result) = cascading_result {
        println!("1. CascadingShrinker:");
        println!("   Result: {:?}", result);
        println!("   Length: {}", result.len());
        println!("   Sum of squares: {}", sum_of_squares(&result));
        println!("   Total candidates explored: {}", candidates.len());
    }

    // Strategy 2: Guided
    let guided = GuidedShrinker::new(original.clone());
    let (guided_result, guided_iterations) =
        guided.find_minimal_with_stats(|v| sum_of_squares(v) > 500);

    println!("\n2. GuidedShrinker:");
    println!("   Result: {:?}", guided_result);
    println!("   Length: {}", guided_result.len());
    println!("   Sum of squares: {}", sum_of_squares(&guided_result));
    println!("   Iterations: {}", guided_iterations);

    // Strategy 3: DFS
    let dfs =
        ConfigurableShrinker::new(original.clone(), ShrinkStrategy::DepthFirst).with_max_depth(20);
    let dfs_result = dfs.find_minimal(|v| sum_of_squares(v) > 500);

    println!("\n3. ConfigurableShrinker (DFS):");
    println!("   Result: {:?}", dfs_result);
    println!("   Length: {}", dfs_result.len());
    println!("   Sum of squares: {}", sum_of_squares(&dfs_result));

    // Strategy 4: BFS
    let bfs = ConfigurableShrinker::new(original.clone(), ShrinkStrategy::BreadthFirst)
        .with_max_depth(20);
    let bfs_result = bfs.find_minimal(|v| sum_of_squares(v) > 500);

    println!("\n4. ConfigurableShrinker (BFS):");
    println!("   Result: {:?}", bfs_result);
    println!("   Length: {}", bfs_result.len());
    println!("   Sum of squares: {}", sum_of_squares(&bfs_result));

    println!("\n{}", "=".repeat(60));
    println!("Summary:");
    println!("  - Cascading: Generates all candidates upfront, manual filtering");
    println!("  - Guided: Fast, iterative, good balance");
    println!("  - DFS: Quick to find *a* minimal, may not be *the* minimal");
    println!("  - BFS: Slowest but finds truly minimal counterexample");
    println!("\nChoose based on:");
    println!("  - How expensive is your test?");
    println!("  - How minimal does the result need to be?");
    println!("  - How much time do you have?");
}
