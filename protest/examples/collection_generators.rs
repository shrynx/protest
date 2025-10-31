//! Example demonstrating the collection generator types added to Protest
//!
//! This example shows how to use:
//! - HashSetGenerator for HashSet<T>
//! - BTreeMapGenerator for BTreeMap<K, V>
//! - BTreeSetGenerator for BTreeSet<T>
//! - ResultGenerator for Result<T, E>
//! - UnitGenerator for ()
//! - ArrayGenerator for [T; N]

use protest::{
    ArrayGenerator, BTreeMapGenerator, BTreeSetGenerator, Generator, GeneratorConfig,
    HashSetGenerator, IntGenerator, Property, PropertyError, ResultGenerator, StringGenerator,
    UnitGenerator, check,
};
use rand::thread_rng;
use std::collections::{BTreeMap, BTreeSet, HashSet};

// ============================================================================
// Property: HashSet contains unique elements
// ============================================================================

struct HashSetUniquenessProperty;

impl Property<HashSet<i32>> for HashSetUniquenessProperty {
    type Output = ();

    fn test(&self, set: HashSet<i32>) -> Result<Self::Output, PropertyError> {
        // HashSet guarantees uniqueness - count should match length
        let values: Vec<_> = set.iter().collect();
        let unique_count = values.len();

        if unique_count == set.len() {
            Ok(())
        } else {
            Err(PropertyError::property_failed(format!(
                "HashSet invariant violated: {} unique values but length is {}",
                unique_count,
                set.len()
            )))
        }
    }
}

// ============================================================================
// Property: BTreeMap maintains sorted order
// ============================================================================

struct BTreeMapOrderProperty;

impl Property<BTreeMap<i32, String>> for BTreeMapOrderProperty {
    type Output = ();

    fn test(&self, map: BTreeMap<i32, String>) -> Result<Self::Output, PropertyError> {
        // BTreeMap should maintain sorted order of keys
        let keys: Vec<_> = map.keys().copied().collect();

        for window in keys.windows(2) {
            if window[0] >= window[1] {
                return Err(PropertyError::property_failed(format!(
                    "BTreeMap keys not in order: {} >= {}",
                    window[0], window[1]
                )));
            }
        }

        Ok(())
    }
}

// ============================================================================
// Property: BTreeSet maintains sorted order
// ============================================================================

struct BTreeSetOrderProperty;

impl Property<BTreeSet<i32>> for BTreeSetOrderProperty {
    type Output = ();

    fn test(&self, set: BTreeSet<i32>) -> Result<Self::Output, PropertyError> {
        // BTreeSet should maintain sorted order
        let values: Vec<_> = set.iter().copied().collect();

        for window in values.windows(2) {
            if window[0] >= window[1] {
                return Err(PropertyError::property_failed(format!(
                    "BTreeSet values not in order: {} >= {}",
                    window[0], window[1]
                )));
            }
        }

        Ok(())
    }
}

// ============================================================================
// Property: Result can be either Ok or Err
// ============================================================================

struct ResultVariantProperty;

impl Property<Result<i32, String>> for ResultVariantProperty {
    type Output = ();

    fn test(&self, result: Result<i32, String>) -> Result<Self::Output, PropertyError> {
        // Result should be either Ok or Err (always true by type system)
        match result {
            Ok(n) => {
                // For this test, we expect values in range
                if n >= 0 && n <= 100 {
                    Ok(())
                } else {
                    Err(PropertyError::property_failed(format!(
                        "Ok value {} out of expected range",
                        n
                    )))
                }
            }
            Err(s) => {
                // String should not be empty for this test
                if !s.is_empty() {
                    Ok(())
                } else {
                    Err(PropertyError::property_failed(
                        "Err string should not be empty",
                    ))
                }
            }
        }
    }
}

// ============================================================================
// Property: Arrays have fixed size
// ============================================================================

struct ArraySizeProperty;

impl Property<[i32; 5]> for ArraySizeProperty {
    type Output = ();

    fn test(&self, arr: [i32; 5]) -> Result<Self::Output, PropertyError> {
        // Array should always have exactly 5 elements (guaranteed by type)
        if arr.len() == 5 {
            // Check all elements are in expected range
            for elem in &arr {
                if !(*elem >= 1 && *elem <= 100) {
                    return Err(PropertyError::property_failed(format!(
                        "Array element {} out of range",
                        elem
                    )));
                }
            }
            Ok(())
        } else {
            Err(PropertyError::property_failed(format!(
                "Array size should be 5 but was {}",
                arr.len()
            )))
        }
    }
}

// ============================================================================
// Main examples
// ============================================================================

fn main() {
    println!("=== Collection Generators Examples ===\n");

    // Example 1: HashSet Generator
    println!("1. HashSet Generator");
    println!("   Generating HashSet<i32> with 0-10 elements in range 1-100");

    let hashset_gen = HashSetGenerator::new(IntGenerator::new(1, 100), 0, 10);
    let mut rng = thread_rng();
    let config = GeneratorConfig::default();

    let sample_set = hashset_gen.generate(&mut rng, &config);
    println!("   Sample: {:?}", sample_set);
    println!("   Size: {}", sample_set.len());

    let result = check(hashset_gen.clone(), HashSetUniquenessProperty);
    println!("   Property check: {:?}\n", result);

    // Example 2: BTreeMap Generator
    println!("2. BTreeMap Generator");
    println!("   Generating BTreeMap<i32, String> with 0-5 entries");

    let btreemap_gen = BTreeMapGenerator::new(
        IntGenerator::new(1, 50),
        StringGenerator::ascii_printable(3, 8),
        0,
        5,
    );

    let sample_map = btreemap_gen.generate(&mut rng, &config);
    println!("   Sample: {:?}", sample_map);
    println!("   Size: {}", sample_map.len());

    let result = check(btreemap_gen, BTreeMapOrderProperty);
    println!("   Property check (sorted order): {:?}\n", result);

    // Example 3: BTreeSet Generator
    println!("3. BTreeSet Generator");
    println!("   Generating BTreeSet<i32> with 0-8 elements in range 1-50");

    let btreeset_gen = BTreeSetGenerator::new(IntGenerator::new(1, 50), 0, 8);

    let sample_btreeset = btreeset_gen.generate(&mut rng, &config);
    println!("   Sample: {:?}", sample_btreeset);
    println!("   Size: {}", sample_btreeset.len());

    let result = check(btreeset_gen, BTreeSetOrderProperty);
    println!("   Property check (sorted order): {:?}\n", result);

    // Example 4: Result Generator
    println!("4. Result Generator");
    println!("   Generating Result<i32, String> with 50% probability each");

    let result_gen = ResultGenerator::new(
        IntGenerator::new(0, 100),
        StringGenerator::ascii_printable(5, 20),
    );

    println!("   Samples:");
    for i in 0..5 {
        let sample_result = result_gen.generate(&mut rng, &config);
        println!("     [{}] {:?}", i, sample_result);
    }

    let result = check(result_gen, ResultVariantProperty);
    println!("   Property check: {:?}\n", result);

    // Example 5: Result Generator with custom probability
    println!("5. Result Generator with Custom Probability");
    println!("   Generating Result<i32, String> with 80% Ok probability");

    let biased_result_gen = ResultGenerator::with_ok_probability(
        IntGenerator::new(0, 100),
        StringGenerator::ascii_printable(5, 20),
        0.8, // 80% Ok, 20% Err
    );

    let mut ok_count = 0;
    let mut err_count = 0;
    for _ in 0..20 {
        match biased_result_gen.generate(&mut rng, &config) {
            Ok(_) => ok_count += 1,
            Err(_) => err_count += 1,
        }
    }
    println!(
        "   Out of 20 samples: {} Ok, {} Err (expected ~16 Ok, ~4 Err)\n",
        ok_count, err_count
    );

    // Example 6: Unit Generator
    println!("6. Unit Generator");
    println!("   Generating unit type ()");

    let unit_gen = UnitGenerator;
    let unit = unit_gen.generate(&mut rng, &config);
    println!("   Sample: {:?}", unit);
    println!("   Unit type has only one value - always ()\n");

    // Example 7: Array Generator
    println!("7. Array Generator");
    println!("   Generating [i32; 5] arrays with elements in range 1-100");

    let array_gen = ArrayGenerator::<i32, _, 5>::new(IntGenerator::new(1, 100));

    let sample_array = array_gen.generate(&mut rng, &config);
    println!("   Sample: {:?}", sample_array);
    println!("   Length: {} (always 5 by type)", sample_array.len());

    let result = check(array_gen, ArraySizeProperty);
    println!("   Property check: {:?}\n", result);

    // Example 8: Nested collections
    println!("8. Nested Collections");
    println!("   Generating HashSet<BTreeSet<i32>>");

    let nested_gen =
        HashSetGenerator::new(BTreeSetGenerator::new(IntGenerator::new(1, 20), 1, 3), 0, 5);

    let nested = nested_gen.generate(&mut rng, &config);
    println!("   Sample: {:?}", nested);
    println!("   Outer HashSet size: {}", nested.len());
    for (i, inner) in nested.iter().enumerate() {
        println!("     Inner BTreeSet {}: {:?}", i, inner);
    }

    println!("\n=== All examples completed successfully! ===");
}
