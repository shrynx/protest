//! Basic tests for manual Generator implementations
//! This demonstrates how the derive macro should work once implemented

use protest::{Arbitrary, Generator, GeneratorConfig};
use rand::thread_rng;

// Manual implementation of what the derive macro should generate
#[derive(Debug, Clone, PartialEq)]
struct SimpleStruct {
    id: u32,
    name: String,
    active: bool,
}

// Manual generator implementation (this is what the derive macro should generate)
#[derive(Debug, Clone, Default)]
pub struct SimpleStructGenerator {
    _phantom: std::marker::PhantomData<SimpleStruct>,
}

impl Generator<SimpleStruct> for SimpleStructGenerator {
    fn generate(&self, _rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> SimpleStruct {
        use rand::SeedableRng;
        let mut local_rng = rand::rngs::StdRng::from_entropy();

        SimpleStruct {
            id: {
                let strategy = u32::arbitrary();
                Generator::generate(&strategy, &mut local_rng, config)
            },
            name: {
                let strategy = String::arbitrary();
                Generator::generate(&strategy, &mut local_rng, config)
            },
            active: {
                let strategy = bool::arbitrary();
                Generator::generate(&strategy, &mut local_rng, config)
            },
        }
    }

    fn shrink(&self, _value: &SimpleStruct) -> Box<dyn Iterator<Item = SimpleStruct>> {
        // Basic implementation - no shrinking for now
        Box::new(std::iter::empty())
    }
}

// Manual enum implementation
#[derive(Debug, Clone, PartialEq)]
enum SimpleEnum {
    Variant1,
    Variant2(i32),
    Variant3 { field: String },
}

#[derive(Debug, Clone, Default)]
pub struct SimpleEnumGenerator {
    _phantom: std::marker::PhantomData<SimpleEnum>,
}

impl Generator<SimpleEnum> for SimpleEnumGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> SimpleEnum {
        use rand::{Rng, SeedableRng};

        let mut local_rng = rand::rngs::StdRng::from_entropy();

        let variant_index = rng.gen_range(0..3);
        match variant_index {
            0 => SimpleEnum::Variant1,
            1 => SimpleEnum::Variant2({
                let strategy = i32::arbitrary();
                Generator::generate(&strategy, &mut local_rng, config)
            }),
            2 => SimpleEnum::Variant3 {
                field: {
                    let strategy = String::arbitrary();
                    Generator::generate(&strategy, &mut local_rng, config)
                },
            },
            _ => unreachable!("Invalid variant index"),
        }
    }

    fn shrink(&self, _value: &SimpleEnum) -> Box<dyn Iterator<Item = SimpleEnum>> {
        Box::new(std::iter::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_struct_generation() {
        let generator = SimpleStructGenerator::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate multiple instances to verify it works
        for _ in 0..10 {
            let instance = generator.generate(&mut rng, &config);

            // Basic validation - the struct should be created successfully
            println!("Generated SimpleStruct: {:?}", instance);
        }
    }

    #[test]
    fn test_manual_enum_generation() {
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
