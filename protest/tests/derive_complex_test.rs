//! Tests for complex derive scenarios
//! This demonstrates handling of generic types, lifetimes, and recursive types

use protest::{Arbitrary, Generator, GeneratorConfig, Strategy};
use rand::{Rng, thread_rng};
use std::marker::PhantomData;

// Generic struct with type parameters
#[derive(Debug, Clone, PartialEq)]
struct GenericStruct<T, U> {
    first: T,
    second: U,
    count: u32,
}

// Manual implementation for generic struct
#[derive(Debug, Clone)]
pub struct GenericStructGenerator<T, U>
where
    T: Arbitrary + 'static,
    U: Arbitrary + 'static,
{
    _phantom: PhantomData<(T, U)>,
}

impl<T, U> Default for GenericStructGenerator<T, U>
where
    T: Arbitrary + 'static,
    U: Arbitrary + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T, U> Generator<GenericStruct<T, U>> for GenericStructGenerator<T, U>
where
    T: Arbitrary + 'static,
    U: Arbitrary + 'static,
{
    fn generate(
        &self,
        _rng: &mut dyn rand::RngCore,
        config: &GeneratorConfig,
    ) -> GenericStruct<T, U> {
        use rand::SeedableRng;
        let mut local_rng = rand::rngs::StdRng::from_entropy();

        GenericStruct {
            first: {
                let strategy = T::arbitrary();
                strategy.generate(&mut local_rng, config)
            },
            second: {
                let strategy = U::arbitrary();
                strategy.generate(&mut local_rng, config)
            },
            count: {
                let strategy = u32::arbitrary_with((0, 100));
                Strategy::generate(&strategy, &mut local_rng, config)
            },
        }
    }

    fn shrink(
        &self,
        _value: &GenericStruct<T, U>,
    ) -> Box<dyn Iterator<Item = GenericStruct<T, U>>> {
        Box::new(std::iter::empty())
    }
}

// Generic enum with constraints
#[derive(Debug, Clone, PartialEq)]
enum GenericEnum<T>
where
    T: Clone,
{
    Empty,
    Single(T),
    Pair(T, T),
}

#[derive(Debug, Clone)]
pub struct GenericEnumGenerator<T>
where
    T: Arbitrary + Clone + 'static,
{
    _phantom: PhantomData<T>,
}

impl<T> Default for GenericEnumGenerator<T>
where
    T: Arbitrary + Clone + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Generator<GenericEnum<T>> for GenericEnumGenerator<T>
where
    T: Arbitrary + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> GenericEnum<T> {
        use rand::{Rng, SeedableRng};
        let mut local_rng = rand::rngs::StdRng::from_entropy();

        let variant_index = rng.gen_range(0..3);
        match variant_index {
            0 => GenericEnum::Empty,
            1 => GenericEnum::Single({
                let strategy = T::arbitrary();
                strategy.generate(&mut local_rng, config)
            }),
            2 => {
                let strategy = T::arbitrary();
                let first = strategy.generate(&mut local_rng, config);
                let second = strategy.generate(&mut local_rng, config);
                GenericEnum::Pair(first, second)
            }
            _ => unreachable!("Invalid variant index"),
        }
    }

    fn shrink(&self, _value: &GenericEnum<T>) -> Box<dyn Iterator<Item = GenericEnum<T>>> {
        Box::new(std::iter::empty())
    }
}

// Recursive type (simplified tree structure)
#[derive(Debug, Clone, PartialEq)]
struct TreeNode<T> {
    value: T,
    children: Vec<TreeNode<T>>,
}

#[derive(Debug, Clone)]
pub struct TreeNodeGenerator<T>
where
    T: Arbitrary + Clone + 'static,
{
    max_depth: u32,
    max_children: u32,
    _phantom: PhantomData<T>,
}

impl<T> Default for TreeNodeGenerator<T>
where
    T: Arbitrary + Clone + 'static,
{
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_children: 3,
            _phantom: PhantomData,
        }
    }
}

impl<T> TreeNodeGenerator<T>
where
    T: Arbitrary + Clone,
{
    pub fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_max_children(mut self, children: u32) -> Self {
        self.max_children = children;
        self
    }
}

impl<T> Generator<TreeNode<T>> for TreeNodeGenerator<T>
where
    T: Arbitrary + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> TreeNode<T> {
        self.generate_with_depth(rng, config, 0)
    }

    fn shrink(&self, _value: &TreeNode<T>) -> Box<dyn Iterator<Item = TreeNode<T>>> {
        // For recursive types, shrinking could remove children or reduce depth
        Box::new(std::iter::empty())
    }
}

impl<T> TreeNodeGenerator<T>
where
    T: Arbitrary + Clone + 'static,
{
    fn generate_with_depth(
        &self,
        rng: &mut dyn rand::RngCore,
        config: &GeneratorConfig,
        depth: u32,
    ) -> TreeNode<T> {
        use rand::{Rng, SeedableRng};
        let mut local_rng = rand::rngs::StdRng::from_entropy();

        let value = {
            let strategy = T::arbitrary();
            strategy.generate(&mut local_rng, config)
        };

        let children = if depth >= self.max_depth {
            // At max depth, no children
            Vec::new()
        } else {
            // Generate 0 to max_children children
            let num_children = rng.gen_range(0..=self.max_children.min(2)); // Limit to prevent explosion
            (0..num_children)
                .map(|_| self.generate_with_depth(rng, config, depth + 1))
                .collect()
        };

        TreeNode { value, children }
    }
}

// Complex nested generic type
#[derive(Debug, Clone, PartialEq)]
struct ComplexNested<T, U>
where
    T: Clone,
    U: Clone,
{
    data: GenericStruct<T, Vec<U>>,
    optional: Option<GenericEnum<T>>,
    mapping: std::collections::HashMap<String, U>,
}

#[derive(Debug, Clone)]
pub struct ComplexNestedGenerator<T, U>
where
    T: Arbitrary + Clone + 'static,
    U: Arbitrary + Clone + 'static,
{
    _phantom: PhantomData<(T, U)>,
}

impl<T, U> Default for ComplexNestedGenerator<T, U>
where
    T: Arbitrary + Clone + 'static,
    U: Arbitrary + Clone + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T, U> Generator<ComplexNested<T, U>> for ComplexNestedGenerator<T, U>
where
    T: Arbitrary + Clone + 'static,
    U: Arbitrary + Clone + std::hash::Hash + Eq + 'static,
{
    fn generate(
        &self,
        _rng: &mut dyn rand::RngCore,
        config: &GeneratorConfig,
    ) -> ComplexNested<T, U> {
        use rand::SeedableRng;
        let mut local_rng = rand::rngs::StdRng::from_entropy();

        ComplexNested {
            data: {
                let generator = GenericStructGenerator::<T, Vec<U>>::default();
                generator.generate(&mut local_rng, config)
            },
            optional: {
                // 50% chance of Some, 50% chance of None
                if local_rng.gen_bool(0.5) {
                    let generator = GenericEnumGenerator::<T>::default();
                    Some(generator.generate(&mut local_rng, config))
                } else {
                    None
                }
            },
            mapping: {
                // Generate a small HashMap
                let mut map = std::collections::HashMap::new();
                let num_entries = local_rng.gen_range(0..=3);
                for i in 0..num_entries {
                    let key = format!("key_{}", i);
                    let value = {
                        let strategy = U::arbitrary();
                        strategy.generate(&mut local_rng, config)
                    };
                    map.insert(key, value);
                }
                map
            },
        }
    }

    fn shrink(
        &self,
        _value: &ComplexNested<T, U>,
    ) -> Box<dyn Iterator<Item = ComplexNested<T, U>>> {
        Box::new(std::iter::empty())
    }
}

// Use u32 instead of usize to avoid orphan rule issues

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_struct_generation() {
        let generator = GenericStructGenerator::<i32, String>::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..5 {
            let instance = generator.generate(&mut rng, &config);
            println!("Generated GenericStruct<i32, String>: {:?}", instance);

            // Basic validation
            assert!(instance.count <= 100);
        }
    }

    #[test]
    fn test_generic_enum_generation() {
        let generator = GenericEnumGenerator::<u32>::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let mut empty_count = 0;
        let mut single_count = 0;
        let mut pair_count = 0;

        for _ in 0..30 {
            let instance = generator.generate(&mut rng, &config);
            match instance {
                GenericEnum::Empty => empty_count += 1,
                GenericEnum::Single(_) => single_count += 1,
                GenericEnum::Pair(_, _) => pair_count += 1,
            }
        }

        // All variants should be generated
        assert!(empty_count > 0, "Empty variant should be generated");
        assert!(single_count > 0, "Single variant should be generated");
        assert!(pair_count > 0, "Pair variant should be generated");
    }

    #[test]
    fn test_recursive_tree_generation() {
        let generator = TreeNodeGenerator::<i32>::default()
            .with_max_depth(2)
            .with_max_children(2);

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..5 {
            let tree = generator.generate(&mut rng, &config);
            println!("Generated TreeNode: {:?}", tree);

            // Verify depth constraint
            fn check_depth<T>(node: &TreeNode<T>, max_depth: u32) -> bool {
                if max_depth == 0 {
                    return node.children.is_empty();
                }
                node.children
                    .iter()
                    .all(|child| check_depth(child, max_depth - 1))
            }

            assert!(check_depth(&tree, 2), "Tree should respect max depth");
        }
    }

    #[test]
    fn test_complex_nested_generation() {
        let generator = ComplexNestedGenerator::<bool, i32>::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..3 {
            let instance = generator.generate(&mut rng, &config);
            println!("Generated ComplexNested: {:?}", instance);

            // Basic validation
            assert!(instance.mapping.len() <= 3);
        }
    }

    #[test]
    fn test_multiple_generic_parameters() {
        // Test with different combinations of generic parameters
        let gen1 = GenericStructGenerator::<String, bool>::default();
        let gen2 = GenericStructGenerator::<u32, Vec<i32>>::default();

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let instance1 = gen1.generate(&mut rng, &config);
        let instance2 = gen2.generate(&mut rng, &config);

        println!("GenericStruct<String, bool>: {:?}", instance1);
        println!("GenericStruct<u32, Vec<i32>>: {:?}", instance2);
    }

    #[test]
    fn test_bounds_checking() {
        // Test that generic bounds are properly handled
        let generator = GenericEnumGenerator::<String>::default();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let instance = generator.generate(&mut rng, &config);

        // Should compile and run without issues due to proper bounds
        match instance {
            GenericEnum::Empty => println!("Generated Empty"),
            GenericEnum::Single(s) => println!("Generated Single: {}", s),
            GenericEnum::Pair(s1, s2) => println!("Generated Pair: {}, {}", s1, s2),
        }
    }
}
