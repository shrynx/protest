//! Integration tests for the derive macro functionality
//! This tests the actual derive macro implementation

#![allow(unused_imports)]

use protest::{Arbitrary, Generator, GeneratorConfig, Strategy};
use rand::thread_rng;

// Test basic struct derivation
#[derive(Debug, Clone, PartialEq, protest::Generator)]
struct SimpleStruct {
    id: u32,
    name: String,
    active: bool,
}

// Test enum derivation
#[derive(Debug, Clone, PartialEq, protest::Generator)]
enum SimpleEnum {
    Variant1,
    Variant2(i32),
    Variant3 { field: String },
}

// Test generic struct derivation
#[derive(Debug, Clone, PartialEq, protest::Generator)]
struct GenericStruct<T, U> {
    first: T,
    second: U,
}

// Test tuple struct derivation
#[derive(Debug, Clone, PartialEq, protest::Generator)]
struct TupleStruct(u32, String);

// Test unit struct derivation
#[derive(Debug, Clone, PartialEq, protest::Generator)]
struct UnitStruct;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_struct_derivation() {
        let generator = SimpleStructGenerator::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate multiple instances to verify it works
        for _ in 0..10 {
            let instance = generator.generate(&mut rng, &config);
            println!("Generated SimpleStruct: {:?}", instance);

            // Basic validation - the struct should be created successfully
            // We can't make specific assertions about the values since they're random
        }
    }

    #[test]
    fn test_simple_enum_derivation() {
        let generator = SimpleEnumGenerator::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let mut variant1_count = 0;
        let mut variant2_count = 0;
        let mut variant3_count = 0;

        // Generate many instances to verify all variants can be generated
        for _ in 0..100 {
            let instance = generator.generate(&mut rng, &config);
            match instance {
                SimpleEnum::Variant1 => variant1_count += 1,
                SimpleEnum::Variant2(_) => variant2_count += 1,
                SimpleEnum::Variant3 { .. } => variant3_count += 1,
            }
        }

        // All variants should have been generated at least once
        assert!(variant1_count > 0, "Variant1 was never generated");
        assert!(variant2_count > 0, "Variant2 was never generated");
        assert!(variant3_count > 0, "Variant3 was never generated");
    }

    #[test]
    fn test_generic_struct_derivation() {
        let generator = GenericStructGenerator::<i32, String>::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..5 {
            let instance = generator.generate(&mut rng, &config);
            println!("Generated GenericStruct<i32, String>: {:?}", instance);
        }
    }

    #[test]
    fn test_tuple_struct_derivation() {
        let generator = TupleStructGenerator::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..5 {
            let instance = generator.generate(&mut rng, &config);
            println!("Generated TupleStruct: {:?}", instance);
        }
    }

    #[test]
    fn test_unit_struct_derivation() {
        let generator = UnitStructGenerator::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let instance = generator.generate(&mut rng, &config);
        assert_eq!(instance, UnitStruct);
        println!("Generated UnitStruct: {:?}", instance);
    }

    #[test]
    fn test_arbitrary_trait_implementation() {
        // Test that the Arbitrary trait is automatically implemented
        let strategy = SimpleStruct::arbitrary();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let instance = strategy.generate(&mut rng, &config);
        println!("Generated via Arbitrary trait: {:?}", instance);
    }

    #[test]
    fn test_shrinking_basic() {
        let generator = SimpleStructGenerator::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let instance = generator.generate(&mut rng, &config);
        let shrinks: Vec<_> = generator.shrink(&instance).collect();

        // For now, shrinking returns empty iterator (basic implementation)
        assert!(shrinks.is_empty());
    }
}
