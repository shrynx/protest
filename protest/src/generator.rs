//! Core generator infrastructure and registry system.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::config::GeneratorConfig;

/// Core generator trait for creating random test data
pub trait Generator<T> {
    /// Generate a random value of type T using the provided RNG and configuration
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> T;

    /// Create an iterator of shrunk values from the given value
    fn shrink(&self, value: &T) -> Box<dyn Iterator<Item = T>>;
}

/// Type-safe registry for storing and retrieving generators
pub struct GeneratorRegistry {
    generators: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl GeneratorRegistry {
    /// Create a new empty generator registry
    pub fn new() -> Self {
        Self {
            generators: HashMap::new(),
        }
    }

    /// Register a generator for a specific type
    pub fn register<T: 'static, G: Generator<T> + Send + Sync + 'static>(&mut self, generator: G) {
        let type_id = TypeId::of::<T>();
        self.generators
            .insert(type_id, Box::new(BoxedGenerator::new(generator)));
    }

    /// Get a generator for a specific type
    pub fn get<T: 'static>(&self) -> Option<&BoxedGenerator<T>> {
        let type_id = TypeId::of::<T>();
        self.generators
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<BoxedGenerator<T>>())
    }

    /// Check if a generator is registered for a specific type
    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.generators.contains_key(&type_id)
    }

    /// Remove a generator for a specific type
    pub fn remove<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        self.generators.remove(&type_id).is_some()
    }

    /// Get the number of registered generators
    pub fn len(&self) -> usize {
        self.generators.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.generators.is_empty()
    }
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper that stores generators in a type-erased way
pub struct BoxedGenerator<T> {
    generator: Box<dyn GeneratorTrait<T> + Send + Sync>,
}

impl<T> BoxedGenerator<T> {
    /// Create a new boxed generator
    pub fn new<G: Generator<T> + Send + Sync + 'static>(generator: G) -> Self {
        Self {
            generator: Box::new(GeneratorWrapper { inner: generator }),
        }
    }
}

impl<T> Generator<T> for BoxedGenerator<T> {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> T {
        self.generator.generate_boxed(rng, config)
    }

    fn shrink(&self, value: &T) -> Box<dyn Iterator<Item = T>> {
        self.generator.shrink_boxed(value)
    }
}

/// Internal trait for type-erased generators
trait GeneratorTrait<T> {
    fn generate_boxed(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> T;
    fn shrink_boxed(&self, value: &T) -> Box<dyn Iterator<Item = T>>;
}

/// Wrapper to make any Generator work with trait objects
struct GeneratorWrapper<G> {
    inner: G,
}

impl<T, G: Generator<T>> GeneratorTrait<T> for GeneratorWrapper<G> {
    fn generate_boxed(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> T {
        self.inner.generate(rng, config)
    }

    fn shrink_boxed(&self, value: &T) -> Box<dyn Iterator<Item = T>> {
        self.inner.shrink(value)
    }
}

/// A simple generator that always produces the same value
#[derive(Debug, Clone)]
pub struct ConstantGenerator<T> {
    value: T,
}

impl<T: Clone> ConstantGenerator<T> {
    /// Create a new constant generator
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Clone + 'static> Generator<T> for ConstantGenerator<T> {
    fn generate(&self, _rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> T {
        self.value.clone()
    }

    fn shrink(&self, _value: &T) -> Box<dyn Iterator<Item = T>> {
        // A constant value cannot be shrunk
        Box::new(std::iter::empty())
    }
}

/// A generator that chooses randomly from a collection of values
#[derive(Debug, Clone)]
pub struct OneOfGenerator<T> {
    values: Vec<T>,
}

impl<T: Clone> OneOfGenerator<T> {
    /// Create a new one-of generator
    pub fn new(values: Vec<T>) -> Self {
        if values.is_empty() {
            panic!("OneOfGenerator cannot be created with empty values");
        }
        Self { values }
    }
}

impl<T: Clone + 'static> Generator<T> for OneOfGenerator<T> {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> T {
        use rand::Rng;
        let index = rng.gen_range(0..self.values.len());
        self.values[index].clone()
    }

    fn shrink(&self, _value: &T) -> Box<dyn Iterator<Item = T>> {
        // For OneOf, we could try other values in the collection as shrinks
        // For now, return empty iterator
        Box::new(std::iter::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_generator_registry_basic_operations() {
        let mut registry = GeneratorRegistry::new();

        // Initially empty
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(!registry.contains::<i32>());

        // Register a generator
        let generator = ConstantGenerator::new(42);
        registry.register::<i32, _>(generator);

        // Check registration
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains::<i32>());
        assert!(!registry.contains::<String>());

        // Remove generator
        assert!(registry.remove::<i32>());
        assert!(!registry.remove::<i32>()); // Second removal should return false
        assert!(registry.is_empty());
    }

    #[test]
    fn test_generator_registry_type_safety() {
        let mut registry = GeneratorRegistry::new();

        // Register generators for different types
        registry.register::<i32, _>(ConstantGenerator::new(42));
        registry.register::<String, _>(ConstantGenerator::new("hello".to_string()));

        assert_eq!(registry.len(), 2);
        assert!(registry.contains::<i32>());
        assert!(registry.contains::<String>());
        assert!(!registry.contains::<f64>());
    }

    #[test]
    fn test_constant_generator() {
        let generator = ConstantGenerator::new(42);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Should always generate the same value
        for _ in 0..10 {
            let value = generator.generate(&mut rng, &config);
            assert_eq!(value, 42);
        }

        // Should not produce any shrinks
        let shrinks: Vec<_> = generator.shrink(&42).collect();
        assert!(shrinks.is_empty());
    }

    #[test]
    fn test_one_of_generator() {
        let values = vec![1, 2, 3, 4, 5];
        let generator = OneOfGenerator::new(values.clone());
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Should generate values from the collection
        for _ in 0..20 {
            let value = generator.generate(&mut rng, &config);
            assert!(values.contains(&value));
        }
    }

    #[test]
    #[should_panic(expected = "OneOfGenerator cannot be created with empty values")]
    fn test_one_of_generator_empty_values() {
        OneOfGenerator::<i32>::new(vec![]);
    }

    #[test]
    fn test_boxed_generator() {
        let generator = BoxedGenerator::new(ConstantGenerator::new("test"));
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = generator.generate(&mut rng, &config);
        assert_eq!(value, "test");

        let shrinks: Vec<_> = generator.shrink(&value).collect();
        assert!(shrinks.is_empty());
    }

    #[test]
    fn test_generator_registry_with_boxed_generators() {
        let mut registry = GeneratorRegistry::new();

        // Register generators using the registry
        registry.register::<i32, _>(ConstantGenerator::new(42));
        registry.register::<String, _>(ConstantGenerator::new("hello".to_string()));

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Test retrieving and using generators
        if let Some(int_gen) = registry.get::<i32>() {
            let value = int_gen.generate(&mut rng, &config);
            assert_eq!(value, 42);
        } else {
            panic!("Expected i32 generator to be registered");
        }

        if let Some(string_gen) = registry.get::<String>() {
            let value = string_gen.generate(&mut rng, &config);
            assert_eq!(value, "hello");
        } else {
            panic!("Expected String generator to be registered");
        }
    }
}
