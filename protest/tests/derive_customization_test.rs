//! Tests for derive macro customization features
//! This demonstrates how attribute-based customization should work

use protest::{Arbitrary, Generator, GeneratorConfig};
use rand::thread_rng;

// Example of what customized derive should look like:
// #[derive(Generator)]
// struct CustomizedStruct {
//     #[generator(range = "1..100")]
//     id: u32,
//     #[generator(length = "5..20")]
//     name: String,
//     #[generator(custom = "always_true")]
//     active: bool,
// }

// Manual implementation showing customization
#[derive(Debug, Clone, PartialEq)]
struct CustomizedStruct {
    id: u32,
    name: String,
    active: bool,
}

// Custom generator with field-level customization
#[derive(Debug, Clone, Default)]
pub struct CustomizedStructGenerator {
    id_min: u32,
    id_max: u32,
    name_min_length: usize,
    name_max_length: usize,
    always_active: bool,
}

impl CustomizedStructGenerator {
    pub fn new() -> Self {
        Self {
            id_min: 1,
            id_max: 100,
            name_min_length: 5,
            name_max_length: 20,
            always_active: false,
        }
    }

    pub fn with_id_range(mut self, min: u32, max: u32) -> Self {
        self.id_min = min;
        self.id_max = max;
        self
    }

    pub fn with_name_length(mut self, min: usize, max: usize) -> Self {
        self.name_min_length = min;
        self.name_max_length = max;
        self
    }

    pub fn with_always_active(mut self, active: bool) -> Self {
        self.always_active = active;
        self
    }
}

impl Generator<CustomizedStruct> for CustomizedStructGenerator {
    fn generate(&self, _rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> CustomizedStruct {
        use rand::{Rng, SeedableRng};
        let mut local_rng = rand::rngs::StdRng::from_entropy();

        CustomizedStruct {
            id: {
                // Use custom range instead of default
                local_rng.gen_range(self.id_min..=self.id_max)
            },
            name: {
                // Use custom length constraints
                let strategy = String::arbitrary_with((self.name_min_length, self.name_max_length));
                protest::Strategy::generate(&strategy, &mut local_rng, config)
            },
            active: {
                if self.always_active {
                    true
                } else {
                    let strategy = bool::arbitrary();
                    protest::Strategy::generate(&strategy, &mut local_rng, config)
                }
            },
        }
    }

    fn shrink(&self, _value: &CustomizedStruct) -> Box<dyn Iterator<Item = CustomizedStruct>> {
        Box::new(std::iter::empty())
    }
}

// Example of field-level custom generator specification
#[derive(Debug, Clone, PartialEq)]
struct FieldCustomStruct {
    normal_field: u32,
    custom_field: String,
}

// Custom generator for a specific field
#[derive(Debug, Clone, Default)]
pub struct AlwaysHelloGenerator;

impl Generator<String> for AlwaysHelloGenerator {
    fn generate(&self, _rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        "Hello".to_string()
    }

    fn shrink(&self, _value: &String) -> Box<dyn Iterator<Item = String>> {
        Box::new(std::iter::empty())
    }
}

#[derive(Debug, Clone, Default)]
pub struct FieldCustomStructGenerator {
    custom_generator: AlwaysHelloGenerator,
}

impl Generator<FieldCustomStruct> for FieldCustomStructGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> FieldCustomStruct {
        use rand::SeedableRng;
        let mut local_rng = rand::rngs::StdRng::from_entropy();

        FieldCustomStruct {
            normal_field: {
                let strategy = u32::arbitrary();
                protest::Strategy::generate(&strategy, &mut local_rng, config)
            },
            custom_field: {
                // Use custom generator for this field
                self.custom_generator.generate(rng, config)
            },
        }
    }

    fn shrink(&self, _value: &FieldCustomStruct) -> Box<dyn Iterator<Item = FieldCustomStruct>> {
        Box::new(std::iter::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customized_struct_generation() {
        let generator = CustomizedStructGenerator::new()
            .with_id_range(50, 150)
            .with_name_length(10, 15)
            .with_always_active(true);

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let instance = generator.generate(&mut rng, &config);

            // Verify customizations are applied
            assert!(
                instance.id >= 50 && instance.id <= 150,
                "ID should be in custom range"
            );
            assert!(
                instance.name.len() >= 10 && instance.name.len() <= 15,
                "Name should be in custom length range"
            );
            assert!(
                instance.active,
                "Should always be active due to customization"
            );

            println!("Generated CustomizedStruct: {:?}", instance);
        }
    }

    #[test]
    fn test_field_custom_generator() {
        let generator = FieldCustomStructGenerator::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let instance = generator.generate(&mut rng, &config);

            // Verify custom field generator is used
            assert_eq!(
                instance.custom_field, "Hello",
                "Custom field should always be 'Hello'"
            );

            println!("Generated FieldCustomStruct: {:?}", instance);
        }
    }

    #[test]
    fn test_default_vs_customized() {
        let default_generator = CustomizedStructGenerator::default();
        let custom_generator = CustomizedStructGenerator::new().with_id_range(1000, 2000);

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate with default settings
        let default_instance = default_generator.generate(&mut rng, &config);

        // Generate with custom settings
        let custom_instance = custom_generator.generate(&mut rng, &config);

        // The custom instance should have ID in the specified range
        assert!(custom_instance.id >= 1000 && custom_instance.id <= 2000);

        println!("Default: {:?}", default_instance);
        println!("Custom: {:?}", custom_instance);
    }

    #[test]
    fn test_builder_pattern_customization() {
        // Test that the builder pattern works for customization
        let generator = CustomizedStructGenerator::new()
            .with_id_range(1, 10)
            .with_name_length(3, 5)
            .with_always_active(false);

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let instance = generator.generate(&mut rng, &config);

        assert!(instance.id >= 1 && instance.id <= 10);
        assert!(instance.name.len() >= 3 && instance.name.len() <= 5);

        println!("Builder pattern result: {:?}", instance);
    }
}
