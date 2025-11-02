//! Generators for primitive types and basic collections.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::arbitrary::Arbitrary;
use crate::config::GeneratorConfig;
use crate::generator::Generator;
use crate::strategy::Strategy;

/// Generator for boolean values
#[derive(Debug, Clone)]
pub struct BoolGenerator;

impl Generator<bool> for BoolGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> bool {
        use rand::Rng;
        rng.r#gen()
    }

    fn shrink(&self, value: &bool) -> Box<dyn Iterator<Item = bool>> {
        // For booleans, shrink true to false
        if *value {
            Box::new(std::iter::once(false))
        } else {
            Box::new(std::iter::empty())
        }
    }
}

/// Generator for integer types with optional range constraints
#[derive(Debug, Clone)]
pub struct IntGenerator<T> {
    min: T,
    max: T,
}

impl<T> IntGenerator<T>
where
    T: Copy + PartialOrd,
{
    /// Create a new integer generator with the full range for the type
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

macro_rules! impl_int_generator {
    ($($t:ty),*) => {
        $(
            impl Generator<$t> for IntGenerator<$t> {
                fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> $t {
                    use rand::Rng;
                    rng.r#gen_range(self.min..=self.max)
                }

                fn shrink(&self, value: &$t) -> Box<dyn Iterator<Item = $t>> {
                    let value = *value;
                    let min = self.min;

                    if value == min {
                        return Box::new(std::iter::empty());
                    }

                    // Shrink towards zero or the minimum value
                    #[allow(unused_comparisons)]
                    let target = if min <= 0 && 0 <= value { 0 } else { min };

                    let mut shrinks = Vec::new();
                    let mut current = value;

                    while current != target {
                        let diff = if current > target { current - target } else { target - current };
                        let step = if diff == 1 { 1 } else { diff / 2 };
                        current = if current > target { current - step } else { current + step };
                        if current >= min && current != value {
                            shrinks.push(current);
                        }
                        if current == target {
                            break;
                        }
                    }

                    Box::new(shrinks.into_iter())
                }
            }

            impl IntGenerator<$t> {
                /// Create a generator for the full range of the type
                pub fn full_range() -> Self {
                    Self::new(<$t>::MIN, <$t>::MAX)
                }
            }
        )*
    };
}

impl_int_generator!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

/// Generator for floating-point types
#[derive(Debug, Clone)]
pub struct FloatGenerator<T> {
    min: T,
    max: T,
}

impl<T> FloatGenerator<T>
where
    T: Copy + PartialOrd,
{
    /// Create a new float generator with the specified range
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

macro_rules! impl_float_generator {
    ($($t:ty),*) => {
        $(
            impl Generator<$t> for FloatGenerator<$t> {
                fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> $t {
                    use rand::Rng;
                    rng.r#gen_range(self.min..=self.max)
                }

                fn shrink(&self, value: &$t) -> Box<dyn Iterator<Item = $t>> {
                    let value = *value;

                    if value == 0.0 || value.is_nan() || value.is_infinite() {
                        return Box::new(std::iter::empty());
                    }

                    let mut shrinks = Vec::new();

                    // Try zero first
                    if self.min <= 0.0 && 0.0 <= self.max {
                        shrinks.push(0.0);
                    }

                    // Try halving the value
                    let half = value / 2.0;
                    if half != value && self.min <= half && half <= self.max {
                        shrinks.push(half);
                    }

                    // Try the integer part
                    let truncated = value.trunc();
                    if truncated != value && self.min <= truncated && truncated <= self.max {
                        shrinks.push(truncated);
                    }

                    Box::new(shrinks.into_iter())
                }
            }

            impl FloatGenerator<$t> {
                /// Create a generator for a reasonable range of the type
                pub fn reasonable_range() -> Self {
                    Self::new(-1000.0, 1000.0)
                }
            }
        )*
    };
}

impl_float_generator!(f32, f64);

/// Generator for character values
#[derive(Debug, Clone)]
pub struct CharGenerator {
    /// Character ranges to generate from
    ranges: Vec<(char, char)>,
}

impl CharGenerator {
    /// Create a new character generator with ASCII printable characters
    pub fn ascii_printable() -> Self {
        Self {
            ranges: vec![(' ', '~')],
        }
    }

    /// Create a new character generator with ASCII alphanumeric characters
    pub fn ascii_alphanumeric() -> Self {
        Self {
            ranges: vec![('0', '9'), ('A', 'Z'), ('a', 'z')],
        }
    }

    /// Create a new character generator with custom ranges
    pub fn with_ranges(ranges: Vec<(char, char)>) -> Self {
        Self { ranges }
    }
}

impl Generator<char> for CharGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> char {
        use rand::Rng;
        if self.ranges.is_empty() {
            return 'a'; // Fallback
        }

        // Pick a random range
        let range_idx = rng.r#gen_range(0..self.ranges.len());
        let (start, end) = self.ranges[range_idx];

        let start_code = start as u32;
        let end_code = end as u32;
        let code = rng.r#gen_range(start_code..=end_code);

        char::from_u32(code).unwrap_or('a')
    }

    fn shrink(&self, value: &char) -> Box<dyn Iterator<Item = char>> {
        let value = *value;
        let mut shrinks = Vec::new();

        // Shrink towards 'a' if it's in our ranges
        if value != 'a' && self.char_in_ranges('a') {
            shrinks.push('a');
        }

        // Shrink towards '0' if it's in our ranges
        if value != '0' && self.char_in_ranges('0') {
            shrinks.push('0');
        }

        // Shrink towards space if it's in our ranges
        if value != ' ' && self.char_in_ranges(' ') {
            shrinks.push(' ');
        }

        Box::new(shrinks.into_iter())
    }
}

impl CharGenerator {
    fn char_in_ranges(&self, c: char) -> bool {
        let code = c as u32;
        self.ranges.iter().any(|(start, end)| {
            let start_code = *start as u32;
            let end_code = *end as u32;
            start_code <= code && code <= end_code
        })
    }
}

/// Generator for string values
#[derive(Debug, Clone)]
pub struct StringGenerator {
    char_generator: CharGenerator,
    min_length: usize,
    max_length: usize,
}

impl StringGenerator {
    /// Create a new string generator with ASCII printable characters
    pub fn ascii_printable(min_length: usize, max_length: usize) -> Self {
        Self {
            char_generator: CharGenerator::ascii_printable(),
            min_length,
            max_length,
        }
    }

    /// Create a new string generator with ASCII alphanumeric characters
    pub fn ascii_alphanumeric(min_length: usize, max_length: usize) -> Self {
        Self {
            char_generator: CharGenerator::ascii_alphanumeric(),
            min_length,
            max_length,
        }
    }

    /// Create a new string generator with a custom character generator
    pub fn with_char_generator(
        char_generator: CharGenerator,
        min_length: usize,
        max_length: usize,
    ) -> Self {
        Self {
            char_generator,
            min_length,
            max_length,
        }
    }
}

impl Generator<String> for StringGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> String {
        use rand::Rng;
        let max_len = self.max_length.min(config.size_hint * 2);
        let length = if self.min_length >= max_len {
            self.min_length
        } else {
            rng.r#gen_range(self.min_length..=max_len)
        };

        (0..length)
            .map(|_| self.char_generator.generate(rng, config))
            .collect()
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let value = value.clone();
        let mut shrinks = Vec::new();

        // 1. Try simple common strings first (if they meet min_length requirement)
        let simple_strings = ["", "a", "A", "0", "1", "test", "foo", "x"];
        for simple in simple_strings {
            if simple.len() >= self.min_length && simple.len() <= value.len() && simple != value {
                shrinks.push(simple.to_string());
            }
        }

        // 2. Shrink towards empty string (only if min_length allows it)
        if self.min_length == 0 && !value.is_empty() && !shrinks.contains(&"".to_string()) {
            shrinks.push("".to_string());
        }

        // 3. Shrink by removing characters from the end
        if value.len() > self.min_length {
            // Try removing half the string
            let half = value.len() / 2;
            if half >= self.min_length {
                shrinks.push(value.chars().take(half).collect());
            }

            // Remove from the end progressively
            for i in (self.min_length..value.len()).rev().step_by(2) {
                shrinks.push(value.chars().take(i).collect());
            }
        }

        // 4. Shrink individual characters to simpler forms
        if !value.is_empty() {
            let chars: Vec<char> = value.chars().collect();

            // Shrink each character toward simpler characters
            for (i, &c) in chars.iter().enumerate() {
                // Try shrinking toward lowercase 'a'
                if c != 'a' && c.is_alphabetic() {
                    let mut new_chars = chars.clone();
                    new_chars[i] = 'a';
                    shrinks.push(new_chars.into_iter().collect());
                }

                // Try shrinking toward '0' for digits
                if c != '0' && c.is_numeric() {
                    let mut new_chars = chars.clone();
                    new_chars[i] = '0';
                    shrinks.push(new_chars.into_iter().collect());
                }

                // Try lowercase version of uppercase letters
                if c.is_uppercase() {
                    let mut new_chars = chars.clone();
                    new_chars[i] = c.to_lowercase().next().unwrap();
                    shrinks.push(new_chars.into_iter().collect());
                }

                // Use the character generator's shrink method
                for shrunk_char in self.char_generator.shrink(&c) {
                    let mut new_chars = chars.clone();
                    new_chars[i] = shrunk_char;
                    let shrunk_string: String = new_chars.into_iter().collect();
                    if !shrinks.contains(&shrunk_string) {
                        shrinks.push(shrunk_string);
                    }
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

/// Generator for `Vec<T>` collections
#[derive(Debug)]
pub struct VecGenerator<T, G> {
    element_generator: G,
    min_length: usize,
    max_length: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G> VecGenerator<T, G>
where
    G: Generator<T>,
{
    /// Create a new vector generator
    pub fn new(element_generator: G, min_length: usize, max_length: usize) -> Self {
        Self {
            element_generator,
            min_length,
            max_length,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G> Generator<Vec<T>> for VecGenerator<T, G>
where
    G: Generator<T>,
    T: Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Vec<T> {
        use rand::Rng;
        let max_len = self.max_length.min(config.size_hint);
        let length = if self.min_length >= max_len {
            self.min_length
        } else {
            rng.r#gen_range(self.min_length..=max_len)
        };

        (0..length)
            .map(|_| self.element_generator.generate(rng, config))
            .collect()
    }

    fn shrink(&self, value: &Vec<T>) -> Box<dyn Iterator<Item = Vec<T>>> {
        let value = value.clone();
        let mut shrinks = Vec::new();

        // Shrink towards empty vector
        if value.len() > self.min_length {
            shrinks.push(Vec::new());
        }

        // Shrink by removing elements (structural shrinking)
        if value.len() > self.min_length {
            for i in (self.min_length..value.len()).rev() {
                shrinks.push(value.iter().take(i).cloned().collect());
            }
        }

        // Element-wise shrinking: try shrinking individual elements
        for i in 0..value.len() {
            let element_shrinks = self.element_generator.shrink(&value[i]);
            for shrunk_element in element_shrinks {
                let mut shrunk_vec = value.clone();
                shrunk_vec[i] = shrunk_element;
                shrinks.push(shrunk_vec);
            }
        }

        Box::new(shrinks.into_iter())
    }
}

/// Generator for `HashMap<K, V>` collections
#[derive(Debug)]
pub struct HashMapGenerator<K, V, KG, VG> {
    key_generator: KG,
    value_generator: VG,
    min_size: usize,
    max_size: usize,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V, KG, VG> HashMapGenerator<K, V, KG, VG>
where
    KG: Generator<K>,
    VG: Generator<V>,
{
    /// Create a new HashMap generator
    pub fn new(key_generator: KG, value_generator: VG, min_size: usize, max_size: usize) -> Self {
        Self {
            key_generator,
            value_generator,
            min_size,
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K, V, KG, VG> Generator<HashMap<K, V>> for HashMapGenerator<K, V, KG, VG>
where
    K: std::hash::Hash + Eq + Clone + 'static,
    V: Clone + 'static,
    KG: Generator<K>,
    VG: Generator<V>,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> HashMap<K, V> {
        use rand::Rng;
        let max_size = self.max_size.min(config.size_hint);
        let target_size = if self.min_size >= max_size {
            self.min_size
        } else {
            rng.r#gen_range(self.min_size..=max_size)
        };

        let mut map = HashMap::new();
        let mut attempts = 0;

        while map.len() < target_size && attempts < target_size * 10 {
            let key = self.key_generator.generate(rng, config);
            let value = self.value_generator.generate(rng, config);
            map.insert(key, value);
            attempts += 1;
        }

        map
    }

    fn shrink(&self, value: &HashMap<K, V>) -> Box<dyn Iterator<Item = HashMap<K, V>>> {
        let mut shrinks = Vec::new();

        // 1. Structural shrinking: remove entries

        // Shrink towards empty map
        if value.len() > self.min_size {
            shrinks.push(HashMap::new());
        }

        // Shrink by removing entries
        if value.len() > self.min_size {
            for (key, _) in value.iter().take(value.len() - self.min_size) {
                let mut smaller = value.clone();
                smaller.remove(key);
                shrinks.push(smaller);
            }
        }

        // 2. Element-wise shrinking: shrink keys and values

        // Shrink values while keeping keys the same
        for (key, val) in value.iter() {
            for shrunk_val in self.value_generator.shrink(val) {
                let mut shrunk_map = value.clone();
                shrunk_map.insert(key.clone(), shrunk_val);
                shrinks.push(shrunk_map);
            }
        }

        // Shrink keys while keeping values the same
        // This is more complex because changing a key means removing the old entry
        // and inserting a new one with the shrunk key
        for (key, val) in value.iter() {
            for shrunk_key in self.key_generator.shrink(key) {
                // Only proceed if the shrunk key doesn't already exist in the map
                // or if it's the same key (in which case it's not a real shrink)
                if !value.contains_key(&shrunk_key) {
                    let mut shrunk_map = value.clone();
                    shrunk_map.remove(key);
                    shrunk_map.insert(shrunk_key, val.clone());
                    shrinks.push(shrunk_map);
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_bool_generator() {
        let generator = BoolGenerator;
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate some values and verify they are booleans (type system ensures this)
        for _ in 0..10 {
            let _value = generator.generate(&mut rng, &config);
            // Value is guaranteed to be a bool by the type system
        }

        // Test shrinking
        let shrinks_true: Vec<_> = generator.shrink(&true).collect();
        assert_eq!(shrinks_true, vec![false]);

        let shrinks_false: Vec<_> = generator.shrink(&false).collect();
        assert!(shrinks_false.is_empty());
    }

    #[test]
    fn test_int_generator() {
        let generator = IntGenerator::new(1, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate values in range
        for _ in 0..20 {
            let value = generator.generate(&mut rng, &config);
            assert!((1..=10).contains(&value));
        }

        // Test shrinking
        let shrinks: Vec<_> = generator.shrink(&5).collect();
        assert!(!shrinks.is_empty());
        assert!(shrinks.iter().all(|&x| (1..5).contains(&x)));
    }

    #[test]
    fn test_float_generator() {
        let generator = FloatGenerator::new(0.0, 1.0);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate values in range
        for _ in 0..20 {
            let value = generator.generate(&mut rng, &config);
            assert!((0.0..=1.0).contains(&value));
        }

        // Test shrinking
        let shrinks: Vec<_> = generator.shrink(&0.5).collect();
        assert!(!shrinks.is_empty());
    }

    #[test]
    fn test_char_generator() {
        let generator = CharGenerator::ascii_alphanumeric();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate alphanumeric characters
        for _ in 0..20 {
            let value = generator.generate(&mut rng, &config);
            assert!(value.is_ascii_alphanumeric());
        }
    }

    #[test]
    fn test_string_generator() {
        let generator = StringGenerator::ascii_alphanumeric(2, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate strings of appropriate length
        for _ in 0..20 {
            let value = generator.generate(&mut rng, &config);
            assert!(value.len() >= 2 && value.len() <= 10);
            assert!(value.chars().all(|c| c.is_ascii_alphanumeric()));
        }

        // Test shrinking
        let test_string = "hello".to_string();
        let shrinks: Vec<_> = generator.shrink(&test_string).collect();
        assert!(!shrinks.is_empty());
    }

    #[test]
    fn test_vec_generator() {
        let element_gen = IntGenerator::new(1, 100);
        let generator = VecGenerator::new(element_gen, 1, 5);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate vectors of appropriate size
        for _ in 0..20 {
            let value = generator.generate(&mut rng, &config);
            assert!(!value.is_empty() && value.len() <= 5);
            assert!(value.iter().all(|&x| (1..=100).contains(&x)));
        }

        // Test shrinking
        let test_vec = vec![1, 2, 3, 4, 5];
        let shrinks: Vec<_> = generator.shrink(&test_vec).collect();
        assert!(!shrinks.is_empty());
    }

    #[test]
    fn test_hashmap_generator() {
        let key_gen = IntGenerator::new(1, 100);
        let value_gen = StringGenerator::ascii_alphanumeric(1, 5);
        let generator = HashMapGenerator::new(key_gen, value_gen, 1, 3);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate hashmaps of appropriate size
        for _ in 0..20 {
            let value = generator.generate(&mut rng, &config);
            assert!(!value.is_empty() && value.len() <= 3);
        }
    }

    #[test]
    fn test_hashmap_structural_shrinking() {
        // Test that HashMap shrinks by removing entries
        let key_gen = IntGenerator::new(1, 100);
        let value_gen = IntGenerator::new(1, 100);
        let generator = HashMapGenerator::new(key_gen, value_gen, 0, 5);

        let mut map = HashMap::new();
        map.insert(1, 10);
        map.insert(2, 20);
        map.insert(3, 30);

        let shrinks: Vec<HashMap<i32, i32>> = generator.shrink(&map).collect();

        // Should include empty map as a shrink
        assert!(
            shrinks.iter().any(|m| m.is_empty()),
            "Should shrink to empty map"
        );

        // Should include maps with fewer entries
        assert!(
            shrinks.iter().any(|m| m.len() < map.len()),
            "Should have smaller maps"
        );
    }

    #[test]
    fn test_hashmap_value_shrinking() {
        // Test that HashMap shrinks individual values
        let key_gen = IntGenerator::new(1, 100);
        let value_gen = IntGenerator::new(0, 100);
        let generator = HashMapGenerator::new(key_gen, value_gen, 0, 5);

        let mut map = HashMap::new();
        map.insert(1, 100);
        map.insert(2, 50);

        let shrinks: Vec<HashMap<i32, i32>> = generator.shrink(&map).collect();

        // Should have shrinks where values are smaller
        // Value 100 should shrink toward 0: 50, 25, 12, etc.
        assert!(
            shrinks
                .iter()
                .any(|m| { m.len() == map.len() && m.get(&1).map(|&v| v < 100).unwrap_or(false) }),
            "Should shrink value at key 1"
        );

        assert!(
            shrinks
                .iter()
                .any(|m| { m.len() == map.len() && m.get(&2).map(|&v| v < 50).unwrap_or(false) }),
            "Should shrink value at key 2"
        );
    }

    #[test]
    fn test_hashmap_key_shrinking() {
        // Test that HashMap shrinks individual keys
        let key_gen = IntGenerator::new(0, 100);
        let value_gen = IntGenerator::new(1, 10);
        let generator = HashMapGenerator::new(key_gen, value_gen, 0, 5);

        let mut map = HashMap::new();
        map.insert(100, 1);
        map.insert(50, 2);

        let shrinks: Vec<HashMap<i32, i32>> = generator.shrink(&map).collect();

        // Should have shrinks where keys are smaller
        // Key 100 should shrink toward 0: 50, 25, 12, etc.
        // The shrunk key must not collide with existing keys
        assert!(
            shrinks
                .iter()
                .any(|m| { m.len() == map.len() && m.contains_key(&25) && m.get(&25) == Some(&1) }),
            "Should shrink key 100 to smaller values like 25"
        );
    }

    #[test]
    fn test_hashmap_combined_shrinking() {
        // Test that HashMap does both structural and element-wise shrinking
        let key_gen = IntGenerator::new(0, 100);
        let value_gen = IntGenerator::new(0, 100);
        let generator = HashMapGenerator::new(key_gen, value_gen, 0, 5);

        let mut map = HashMap::new();
        map.insert(80, 90);
        map.insert(40, 45);

        let shrinks: Vec<HashMap<i32, i32>> = generator.shrink(&map).collect();

        // Should have structural shrinks
        assert!(
            shrinks.iter().any(|m| m.is_empty()),
            "Should have empty map"
        );
        assert!(
            shrinks.iter().any(|m| m.len() == 1),
            "Should have single-entry map"
        );

        // Should have value shrinks
        assert!(
            shrinks
                .iter()
                .any(|m| { m.len() == 2 && m.get(&80).map(|&v| v < 90).unwrap_or(false) }),
            "Should shrink values"
        );

        // Should have key shrinks (but avoiding collisions)
        // Check that we attempted to shrink keys by looking for maps that have
        // different keys but same size
        let has_key_shrinks = shrinks.iter().any(|m| {
            m.len() == 2
                && ((m.contains_key(&80) && !m.contains_key(&40))
                    || (!m.contains_key(&80) && m.contains_key(&40)))
        });
        assert!(has_key_shrinks, "Should have attempted to shrink keys");
    }

    #[test]
    fn test_hashset_element_shrinking() {
        // Test that HashSet shrinks individual elements
        let elem_gen = IntGenerator::new(0, 100);
        let generator = HashSetGenerator::new(elem_gen, 0, 5);

        let mut set = HashSet::new();
        set.insert(100);
        set.insert(50);
        set.insert(25);

        let shrinks: Vec<HashSet<i32>> = generator.shrink(&set).collect();

        // Should have structural shrinks (removing elements)
        assert!(
            shrinks.iter().any(|s| s.is_empty()),
            "Should shrink to empty set"
        );
        assert!(
            shrinks.iter().any(|s| s.len() < set.len()),
            "Should have smaller sets"
        );

        // Should have element-wise shrinks (shrinking individual elements)
        // Element 100 should shrink to 50, 25, etc.
        assert!(
            shrinks.iter().any(|s| {
                s.len() == set.len() && s.contains(&50) && s.contains(&25) && !s.contains(&100)
            }),
            "Should shrink element 100 to smaller values"
        );
    }

    #[test]
    fn test_full_range_generators() {
        let i32_gen = IntGenerator::<i32>::full_range();
        let f64_gen = FloatGenerator::<f64>::reasonable_range();

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Just test that they can generate values without panicking
        let _int_val = i32_gen.generate(&mut rng, &config);
        let _float_val = f64_gen.generate(&mut rng, &config);
    }

    #[test]
    fn test_string_shrinks_to_simple_strings() {
        // Test that strings shrink to common simple strings
        let generator = StringGenerator::ascii_printable(0, 10);
        let test_string = "complex".to_string();

        let shrinks: Vec<String> = generator.shrink(&test_string).collect();

        // Should include simple common strings
        assert!(
            shrinks.contains(&"".to_string()),
            "Should shrink to empty string"
        );
        assert!(shrinks.contains(&"a".to_string()), "Should shrink to 'a'");
        assert!(
            shrinks.contains(&"test".to_string()),
            "Should shrink to 'test'"
        );
    }

    #[test]
    fn test_string_character_simplification() {
        // Test that individual characters shrink to simpler forms
        let generator = StringGenerator::ascii_printable(0, 10);
        let test_string = "ZXY".to_string();

        let shrinks: Vec<String> = generator.shrink(&test_string).collect();

        // Should shrink uppercase to lowercase
        assert!(
            shrinks.contains(&"zXY".to_string())
                || shrinks.contains(&"ZxY".to_string())
                || shrinks.contains(&"ZXy".to_string()),
            "Should shrink some uppercase letters to lowercase"
        );

        // Should shrink letters toward 'a'
        assert!(
            shrinks.contains(&"aXY".to_string())
                || shrinks.contains(&"ZaY".to_string())
                || shrinks.contains(&"ZXa".to_string()),
            "Should shrink some letters toward 'a'"
        );
    }

    #[test]
    fn test_string_digit_simplification() {
        // Test that digits shrink toward '0'
        let generator = StringGenerator::ascii_alphanumeric(0, 10);
        let test_string = "abc789".to_string();

        let shrinks: Vec<String> = generator.shrink(&test_string).collect();

        // Should shrink digits toward '0'
        assert!(
            shrinks.iter().any(|s| s.contains('0')),
            "Should shrink some digits toward '0'"
        );
    }

    #[test]
    fn test_string_length_reduction() {
        // Test that strings shrink in length
        let generator = StringGenerator::ascii_printable(0, 20);
        let test_string = "verylongstring".to_string();

        let shrinks: Vec<String> = generator.shrink(&test_string).collect();

        // Should have shorter strings
        assert!(
            shrinks.iter().any(|s| s.len() < test_string.len()),
            "Should produce shorter strings"
        );

        // Should try removing half
        let half_len = test_string.len() / 2;
        assert!(
            shrinks.iter().any(|s| s.len() == half_len),
            "Should try removing half the string"
        );
    }

    #[test]
    fn test_string_respects_min_length() {
        // Test that shrinking respects minimum length
        let generator = StringGenerator::ascii_printable(3, 10);
        let test_string = "hello".to_string();

        let shrinks: Vec<String> = generator.shrink(&test_string).collect();

        // All shrinks should be at least min_length
        for shrink in &shrinks {
            assert!(
                shrink.len() >= 3,
                "Shrink '{}' should be at least {} chars, got {}",
                shrink,
                3,
                shrink.len()
            );
        }

        // Should not include empty string or single character
        assert!(
            !shrinks.contains(&"".to_string()),
            "Should not shrink below min_length"
        );
        assert!(
            !shrinks.contains(&"a".to_string()),
            "Should not shrink below min_length"
        );
    }

    #[test]
    fn test_string_combined_shrinking() {
        // Test that string uses all shrinking strategies together
        let generator = StringGenerator::ascii_alphanumeric(0, 20);
        let test_string = "HELLO123".to_string();

        let shrinks: Vec<String> = generator.shrink(&test_string).collect();

        // Should have structural shrinks (shorter strings)
        assert!(shrinks.iter().any(|s| s.len() < test_string.len()));

        // Should have character simplification shrinks
        assert!(shrinks.iter().any(|s| s.chars().any(|c| c.is_lowercase())));

        // Should have digit simplification
        assert!(shrinks.iter().any(|s| s.contains('0')));

        // Should have simple string attempts
        assert!(shrinks.contains(&"test".to_string()) || shrinks.contains(&"foo".to_string()));
    }
}

// Arbitrary implementations for primitive types

impl Arbitrary for bool {
    type Strategy = BoolStrategy;
    type Parameters = ();

    fn arbitrary() -> Self::Strategy {
        BoolStrategy
    }

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        BoolStrategy
    }
}

#[derive(Debug, Clone)]
pub struct BoolStrategy;

impl Strategy for BoolStrategy {
    type Value = bool;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> bool {
        BoolGenerator.generate(rng, config)
    }

    fn shrink(&self, value: &bool) -> Box<dyn Iterator<Item = bool>> {
        BoolGenerator.shrink(value)
    }
}

impl Generator<bool> for BoolStrategy {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> bool {
        BoolGenerator.generate(rng, config)
    }

    fn shrink(&self, value: &bool) -> Box<dyn Iterator<Item = bool>> {
        BoolGenerator.shrink(value)
    }
}

// Individual implementations for each integer type to avoid macro conflicts

impl Arbitrary for i32 {
    type Strategy = I32Strategy;
    type Parameters = (i32, i32);

    fn arbitrary() -> Self::Strategy {
        I32Strategy::full_range()
    }

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        I32Strategy::new(args.0, args.1)
    }
}

#[derive(Debug, Clone)]
pub struct I32Strategy {
    generator: IntGenerator<i32>,
}

impl I32Strategy {
    pub fn new(min: i32, max: i32) -> Self {
        Self {
            generator: IntGenerator::new(min, max),
        }
    }

    pub fn full_range() -> Self {
        Self {
            generator: IntGenerator::<i32>::full_range(),
        }
    }
}

impl Strategy for I32Strategy {
    type Value = i32;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> i32 {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &i32) -> Box<dyn Iterator<Item = i32>> {
        self.generator.shrink(value)
    }
}

impl Generator<i32> for I32Strategy {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> i32 {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &i32) -> Box<dyn Iterator<Item = i32>> {
        self.generator.shrink(value)
    }
}

impl Arbitrary for u32 {
    type Strategy = U32Strategy;
    type Parameters = (u32, u32);

    fn arbitrary() -> Self::Strategy {
        U32Strategy::full_range()
    }

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        U32Strategy::new(args.0, args.1)
    }
}

#[derive(Debug, Clone)]
pub struct U32Strategy {
    generator: IntGenerator<u32>,
}

impl U32Strategy {
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            generator: IntGenerator::new(min, max),
        }
    }

    pub fn full_range() -> Self {
        Self {
            generator: IntGenerator::<u32>::full_range(),
        }
    }
}

impl Strategy for U32Strategy {
    type Value = u32;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> u32 {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &u32) -> Box<dyn Iterator<Item = u32>> {
        self.generator.shrink(value)
    }
}

impl Generator<u32> for U32Strategy {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> u32 {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &u32) -> Box<dyn Iterator<Item = u32>> {
        self.generator.shrink(value)
    }
}

// Note: Only implementing i32 and u32 for now to avoid macro complexity

// Float implementations

impl Arbitrary for f64 {
    type Strategy = F64Strategy;
    type Parameters = (f64, f64);

    fn arbitrary() -> Self::Strategy {
        F64Strategy::reasonable_range()
    }

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        F64Strategy::new(args.0, args.1)
    }
}

#[derive(Debug, Clone)]
pub struct F64Strategy {
    generator: FloatGenerator<f64>,
}

impl F64Strategy {
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            generator: FloatGenerator::new(min, max),
        }
    }

    pub fn reasonable_range() -> Self {
        Self {
            generator: FloatGenerator::<f64>::reasonable_range(),
        }
    }
}

impl Strategy for F64Strategy {
    type Value = f64;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> f64 {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &f64) -> Box<dyn Iterator<Item = f64>> {
        self.generator.shrink(value)
    }
}

impl Generator<f64> for F64Strategy {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> f64 {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &f64) -> Box<dyn Iterator<Item = f64>> {
        self.generator.shrink(value)
    }
}

// Note: Only implementing f64 for now

impl Arbitrary for char {
    type Strategy = CharStrategy;
    type Parameters = Vec<(char, char)>;

    fn arbitrary() -> Self::Strategy {
        CharStrategy::ascii_printable()
    }

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        CharStrategy::with_ranges(args)
    }
}

#[derive(Debug, Clone)]
pub struct CharStrategy {
    generator: CharGenerator,
}

impl CharStrategy {
    pub fn ascii_printable() -> Self {
        Self {
            generator: CharGenerator::ascii_printable(),
        }
    }

    pub fn ascii_alphanumeric() -> Self {
        Self {
            generator: CharGenerator::ascii_alphanumeric(),
        }
    }

    pub fn with_ranges(ranges: Vec<(char, char)>) -> Self {
        Self {
            generator: CharGenerator::with_ranges(ranges),
        }
    }
}

impl Strategy for CharStrategy {
    type Value = char;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> char {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &char) -> Box<dyn Iterator<Item = char>> {
        self.generator.shrink(value)
    }
}

impl Generator<char> for CharStrategy {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> char {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &char) -> Box<dyn Iterator<Item = char>> {
        self.generator.shrink(value)
    }
}

impl Arbitrary for String {
    type Strategy = StringStrategy;
    type Parameters = (usize, usize);

    fn arbitrary() -> Self::Strategy {
        StringStrategy::ascii_printable(0, 20)
    }

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        StringStrategy::ascii_printable(args.0, args.1)
    }
}

#[derive(Debug, Clone)]
pub struct StringStrategy {
    generator: StringGenerator,
}

impl StringStrategy {
    pub fn ascii_printable(min_length: usize, max_length: usize) -> Self {
        Self {
            generator: StringGenerator::ascii_printable(min_length, max_length),
        }
    }

    pub fn ascii_alphanumeric(min_length: usize, max_length: usize) -> Self {
        Self {
            generator: StringGenerator::ascii_alphanumeric(min_length, max_length),
        }
    }

    pub fn with_char_strategy(
        char_strategy: CharStrategy,
        min_length: usize,
        max_length: usize,
    ) -> Self {
        Self {
            generator: StringGenerator::with_char_generator(
                char_strategy.generator,
                min_length,
                max_length,
            ),
        }
    }
}

impl Strategy for StringStrategy {
    type Value = String;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> String {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        self.generator.shrink(value)
    }
}

impl Generator<String> for StringStrategy {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> String {
        self.generator.generate(rng, config)
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        self.generator.shrink(value)
    }
}

// Vec implementation
impl<T: Arbitrary + Clone + 'static> Arbitrary for Vec<T> {
    type Strategy = VecStrategy<T>;
    type Parameters = (usize, usize);

    fn arbitrary() -> Self::Strategy {
        VecStrategy::new(0, 10)
    }

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        VecStrategy::new(args.0, args.1)
    }
}

#[derive(Debug, Clone)]
pub struct VecStrategy<T> {
    min_length: usize,
    max_length: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> VecStrategy<T> {
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            min_length,
            max_length,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Arbitrary + Clone + 'static> Strategy for VecStrategy<T> {
    type Value = Vec<T>;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Vec<T> {
        let length = rng.gen_range(self.min_length..=self.max_length.min(config.size_hint));
        (0..length)
            .map(|_| {
                let strategy = T::arbitrary();
                strategy.generate(rng, config)
            })
            .collect()
    }

    fn shrink(&self, value: &Vec<T>) -> Box<dyn Iterator<Item = Vec<T>>> {
        let value = value.clone();
        let mut shrinks = Vec::new();

        // Shrink towards empty vector
        if value.len() > self.min_length {
            shrinks.push(Vec::new());
        }

        // Shrink by removing elements
        if value.len() > self.min_length {
            for i in (self.min_length..value.len()).rev() {
                let shrunk: Vec<T> = value.iter().take(i).cloned().collect();
                shrinks.push(shrunk);
            }
        }

        Box::new(shrinks.into_iter())
    }
}

impl<T: Arbitrary + Clone + 'static> Generator<Vec<T>> for VecStrategy<T> {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Vec<T> {
        use rand::Rng;
        let length = rng.gen_range(self.min_length..=self.max_length.min(config.size_hint));
        (0..length)
            .map(|_| {
                let strategy = T::arbitrary();
                // We need to create a concrete RNG type to call strategy.generate
                // For now, let's use a simple approach with thread_rng
                let mut thread_rng = rand::thread_rng();
                strategy.generate(&mut thread_rng, config)
            })
            .collect()
    }

    fn shrink(&self, value: &Vec<T>) -> Box<dyn Iterator<Item = Vec<T>>> {
        let value = value.clone();
        let mut shrinks = Vec::new();

        // Shrink towards empty vector
        if !value.is_empty() {
            shrinks.push(Vec::new());
        }

        // Shrink by removing elements
        for i in 0..value.len() {
            let mut shrunk = value.clone();
            shrunk.remove(i);
            shrinks.push(shrunk);
        }

        Box::new(shrinks.into_iter())
    }
}

// Tuple implementations for property test macro support

impl<A: Arbitrary + Clone + 'static, B: Arbitrary + Clone + 'static> Arbitrary for (A, B) {
    type Strategy = TupleStrategy2<A, B>;
    type Parameters = ();

    fn arbitrary() -> Self::Strategy {
        TupleStrategy2::new()
    }

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        TupleStrategy2::new()
    }
}

#[derive(Debug, Clone)]
pub struct TupleStrategy2<A, B> {
    _phantom: std::marker::PhantomData<(A, B)>,
}

impl<A, B> Default for TupleStrategy2<A, B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A, B> TupleStrategy2<A, B> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<A: Arbitrary + Clone + 'static, B: Arbitrary + Clone + 'static> Strategy
    for TupleStrategy2<A, B>
{
    type Value = (A, B);

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> (A, B) {
        let a_strategy = A::arbitrary();
        let b_strategy = B::arbitrary();
        (
            a_strategy.generate(rng, config),
            b_strategy.generate(rng, config),
        )
    }

    fn shrink(&self, value: &(A, B)) -> Box<dyn Iterator<Item = (A, B)>> {
        let (a, b) = value.clone();
        let mut shrinks = Vec::new();

        // Shrink first element
        let a_strategy = A::arbitrary();
        for shrunk_a in a_strategy.shrink(&a) {
            shrinks.push((shrunk_a, b.clone()));
        }

        // Shrink second element
        let b_strategy = B::arbitrary();
        for shrunk_b in b_strategy.shrink(&b) {
            shrinks.push((a.clone(), shrunk_b));
        }

        Box::new(shrinks.into_iter())
    }
}

impl<A: Arbitrary + Clone + 'static, B: Arbitrary + Clone + 'static> Generator<(A, B)>
    for TupleStrategy2<A, B>
{
    fn generate(&self, _rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> (A, B) {
        let a_strategy = A::arbitrary();
        let b_strategy = B::arbitrary();
        // We need to create concrete RNG types to call strategy.generate
        let mut thread_rng = rand::thread_rng();
        let a_value = a_strategy.generate(&mut thread_rng, config);
        let b_value = b_strategy.generate(&mut thread_rng, config);
        (a_value, b_value)
    }

    fn shrink(&self, value: &(A, B)) -> Box<dyn Iterator<Item = (A, B)>> {
        // For tuples, we can try shrinking each component independently
        // This is a simplified approach
        let (a, b) = value.clone();
        let mut shrinks = Vec::new();

        // Try shrinking the first component
        let a_strategy = A::arbitrary();
        let a_shrinks: Vec<A> = a_strategy.shrink(&a).collect();
        for shrunk_a in a_shrinks {
            shrinks.push((shrunk_a, b.clone()));
        }

        // Try shrinking the second component
        let b_strategy = B::arbitrary();
        let b_shrinks: Vec<B> = b_strategy.shrink(&b).collect();
        for shrunk_b in b_shrinks {
            shrinks.push((a.clone(), shrunk_b));
        }

        Box::new(shrinks.into_iter())
    }
}

impl<A: Arbitrary + Clone + 'static, B: Arbitrary + Clone + 'static, C: Arbitrary + Clone + 'static>
    Arbitrary for (A, B, C)
{
    type Strategy = TupleStrategy3<A, B, C>;
    type Parameters = ();

    fn arbitrary() -> Self::Strategy {
        TupleStrategy3::new()
    }

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        TupleStrategy3::new()
    }
}

#[derive(Debug, Clone)]
pub struct TupleStrategy3<A, B, C> {
    _phantom: std::marker::PhantomData<(A, B, C)>,
}

impl<A, B, C> Default for TupleStrategy3<A, B, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A, B, C> TupleStrategy3<A, B, C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<A: Arbitrary + Clone + 'static, B: Arbitrary + Clone + 'static, C: Arbitrary + Clone + 'static>
    Strategy for TupleStrategy3<A, B, C>
{
    type Value = (A, B, C);

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> (A, B, C) {
        let a_strategy = A::arbitrary();
        let b_strategy = B::arbitrary();
        let c_strategy = C::arbitrary();
        (
            a_strategy.generate(rng, config),
            b_strategy.generate(rng, config),
            c_strategy.generate(rng, config),
        )
    }

    fn shrink(&self, value: &(A, B, C)) -> Box<dyn Iterator<Item = (A, B, C)>> {
        let (a, b, c) = value.clone();
        let mut shrinks = Vec::new();

        // Shrink first element
        let a_strategy = A::arbitrary();
        for shrunk_a in a_strategy.shrink(&a) {
            shrinks.push((shrunk_a, b.clone(), c.clone()));
        }

        // Shrink second element
        let b_strategy = B::arbitrary();
        for shrunk_b in b_strategy.shrink(&b) {
            shrinks.push((a.clone(), shrunk_b, c.clone()));
        }

        // Shrink third element
        let c_strategy = C::arbitrary();
        for shrunk_c in c_strategy.shrink(&c) {
            shrinks.push((a.clone(), b.clone(), shrunk_c));
        }

        Box::new(shrinks.into_iter())
    }
}

impl<A: Arbitrary + Clone + 'static, B: Arbitrary + Clone + 'static, C: Arbitrary + Clone + 'static>
    Generator<(A, B, C)> for TupleStrategy3<A, B, C>
{
    fn generate(&self, _rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> (A, B, C) {
        let a_strategy = A::arbitrary();
        let b_strategy = B::arbitrary();
        let c_strategy = C::arbitrary();
        // We need to create concrete RNG types to call strategy.generate
        let mut thread_rng = rand::thread_rng();
        let a_value = a_strategy.generate(&mut thread_rng, config);
        let b_value = b_strategy.generate(&mut thread_rng, config);
        let c_value = c_strategy.generate(&mut thread_rng, config);
        (a_value, b_value, c_value)
    }

    fn shrink(&self, value: &(A, B, C)) -> Box<dyn Iterator<Item = (A, B, C)>> {
        // For tuples, we can try shrinking each component independently
        let (a, b, c) = value.clone();
        let mut shrinks = Vec::new();

        // Try shrinking the first component
        let a_strategy = A::arbitrary();
        let a_shrinks: Vec<A> = a_strategy.shrink(&a).collect();
        for shrunk_a in a_shrinks {
            shrinks.push((shrunk_a, b.clone(), c.clone()));
        }

        // Try shrinking the second component
        let b_strategy = B::arbitrary();
        let b_shrinks: Vec<B> = b_strategy.shrink(&b).collect();
        for shrunk_b in b_shrinks {
            shrinks.push((a.clone(), shrunk_b, c.clone()));
        }

        // Try shrinking the third component
        let c_strategy = C::arbitrary();
        let c_shrinks: Vec<C> = c_strategy.shrink(&c).collect();
        for shrunk_c in c_shrinks {
            shrinks.push((a.clone(), b.clone(), shrunk_c));
        }

        Box::new(shrinks.into_iter())
    }
}
// ============================================================================
// Tuple Generators (for ergonomic API)
// ============================================================================

/// Generator for 2-tuples
pub struct TupleGenerator<T1, T2, G1, G2> {
    gen1: G1,
    gen2: G2,
    _phantom: std::marker::PhantomData<(T1, T2)>,
}

impl<T1, T2, G1, G2> TupleGenerator<T1, T2, G1, G2> {
    pub fn new(gen1: G1, gen2: G2) -> Self {
        Self {
            gen1,
            gen2,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T1, T2, G1, G2> Generator<(T1, T2)> for TupleGenerator<T1, T2, G1, G2>
where
    T1: Clone + 'static,
    T2: Clone + 'static,
    G1: Generator<T1>,
    G2: Generator<T2>,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> (T1, T2) {
        (
            self.gen1.generate(rng, config),
            self.gen2.generate(rng, config),
        )
    }

    fn shrink(&self, value: &(T1, T2)) -> Box<dyn Iterator<Item = (T1, T2)>> {
        let (v1, v2) = value.clone();
        let mut shrinks = Vec::new();

        // Shrink first component
        for s1 in self.gen1.shrink(&v1) {
            shrinks.push((s1, v2.clone()));
        }

        // Shrink second component
        for s2 in self.gen2.shrink(&v2) {
            shrinks.push((v1.clone(), s2));
        }

        Box::new(shrinks.into_iter())
    }
}

/// Generator for 3-tuples
pub struct Tuple3Generator<T1, T2, T3, G1, G2, G3> {
    gen1: G1,
    gen2: G2,
    gen3: G3,
    _phantom: std::marker::PhantomData<(T1, T2, T3)>,
}

impl<T1, T2, T3, G1, G2, G3> Tuple3Generator<T1, T2, T3, G1, G2, G3> {
    pub fn new(gen1: G1, gen2: G2, gen3: G3) -> Self {
        Self {
            gen1,
            gen2,
            gen3,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T1, T2, T3, G1, G2, G3> Generator<(T1, T2, T3)> for Tuple3Generator<T1, T2, T3, G1, G2, G3>
where
    T1: Clone + 'static,
    T2: Clone + 'static,
    T3: Clone + 'static,
    G1: Generator<T1>,
    G2: Generator<T2>,
    G3: Generator<T3>,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> (T1, T2, T3) {
        (
            self.gen1.generate(rng, config),
            self.gen2.generate(rng, config),
            self.gen3.generate(rng, config),
        )
    }

    fn shrink(&self, value: &(T1, T2, T3)) -> Box<dyn Iterator<Item = (T1, T2, T3)>> {
        let (v1, v2, v3) = value.clone();
        let mut shrinks = Vec::new();

        for s1 in self.gen1.shrink(&v1) {
            shrinks.push((s1, v2.clone(), v3.clone()));
        }

        for s2 in self.gen2.shrink(&v2) {
            shrinks.push((v1.clone(), s2, v3.clone()));
        }

        for s3 in self.gen3.shrink(&v3) {
            shrinks.push((v1.clone(), v2.clone(), s3));
        }

        Box::new(shrinks.into_iter())
    }
}

/// Generator for 4-tuples
pub struct Tuple4Generator<T1, T2, T3, T4, G1, G2, G3, G4> {
    gen1: G1,
    gen2: G2,
    gen3: G3,
    gen4: G4,
    _phantom: std::marker::PhantomData<(T1, T2, T3, T4)>,
}

impl<T1, T2, T3, T4, G1, G2, G3, G4> Tuple4Generator<T1, T2, T3, T4, G1, G2, G3, G4> {
    pub fn new(gen1: G1, gen2: G2, gen3: G3, gen4: G4) -> Self {
        Self {
            gen1,
            gen2,
            gen3,
            gen4,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T1, T2, T3, T4, G1, G2, G3, G4> Generator<(T1, T2, T3, T4)>
    for Tuple4Generator<T1, T2, T3, T4, G1, G2, G3, G4>
where
    T1: Clone + 'static,
    T2: Clone + 'static,
    T3: Clone + 'static,
    T4: Clone + 'static,
    G1: Generator<T1>,
    G2: Generator<T2>,
    G3: Generator<T3>,
    G4: Generator<T4>,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> (T1, T2, T3, T4) {
        (
            self.gen1.generate(rng, config),
            self.gen2.generate(rng, config),
            self.gen3.generate(rng, config),
            self.gen4.generate(rng, config),
        )
    }

    fn shrink(&self, value: &(T1, T2, T3, T4)) -> Box<dyn Iterator<Item = (T1, T2, T3, T4)>> {
        let (v1, v2, v3, v4) = value.clone();
        let mut shrinks = Vec::new();

        for s1 in self.gen1.shrink(&v1) {
            shrinks.push((s1, v2.clone(), v3.clone(), v4.clone()));
        }

        for s2 in self.gen2.shrink(&v2) {
            shrinks.push((v1.clone(), s2, v3.clone(), v4.clone()));
        }

        for s3 in self.gen3.shrink(&v3) {
            shrinks.push((v1.clone(), v2.clone(), s3, v4.clone()));
        }

        for s4 in self.gen4.shrink(&v4) {
            shrinks.push((v1.clone(), v2.clone(), v3.clone(), s4));
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Option Generator
// ============================================================================

/// Generator for `Option<T>`
pub struct OptionGenerator<T, G> {
    inner_gen: G,
    some_probability: f64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G> OptionGenerator<T, G> {
    /// Create a new Option generator with 50% probability of Some
    pub fn new(inner_gen: G) -> Self {
        Self {
            inner_gen,
            some_probability: 0.5,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create an Option generator with custom probability of Some
    pub fn with_probability(inner_gen: G, some_probability: f64) -> Self {
        Self {
            inner_gen,
            some_probability: some_probability.clamp(0.0, 1.0),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G> Generator<Option<T>> for OptionGenerator<T, G>
where
    T: Clone + 'static,
    G: Generator<T>,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Option<T> {
        use rand::Rng;
        let random_value: f64 = rng.r#gen();
        if random_value < self.some_probability {
            Some(self.inner_gen.generate(rng, config))
        } else {
            None
        }
    }

    fn shrink(&self, value: &Option<T>) -> Box<dyn Iterator<Item = Option<T>>> {
        match value {
            None => Box::new(std::iter::empty()),
            Some(v) => {
                let mut shrinks = vec![None]; // Always try None first
                for shrunk in self.inner_gen.shrink(v) {
                    shrinks.push(Some(shrunk));
                }
                Box::new(shrinks.into_iter())
            }
        }
    }
}

// Tuple generator implementations to support property_test macro with multiple parameters
impl<A, B, GA, GB> Generator<(A, B)> for (GA, GB)
where
    GA: Generator<A>,
    GB: Generator<B>,
    A: Clone + std::fmt::Debug + PartialEq + 'static,
    B: Clone + std::fmt::Debug + PartialEq + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> (A, B) {
        (self.0.generate(rng, config), self.1.generate(rng, config))
    }

    fn shrink(&self, value: &(A, B)) -> Box<dyn Iterator<Item = (A, B)>> {
        // Simple shrinking: shrink first element, then second
        let mut shrinks = Vec::new();

        // Shrink first element
        for a_shrunk in self.0.shrink(&value.0) {
            shrinks.push((a_shrunk, value.1.clone()));
        }

        // Shrink second element
        for b_shrunk in self.1.shrink(&value.1) {
            shrinks.push((value.0.clone(), b_shrunk));
        }

        Box::new(shrinks.into_iter())
    }
}

impl<A, B, C, GA, GB, GC> Generator<(A, B, C)> for (GA, GB, GC)
where
    GA: Generator<A>,
    GB: Generator<B>,
    GC: Generator<C>,
    A: Clone + std::fmt::Debug + PartialEq + 'static,
    B: Clone + std::fmt::Debug + PartialEq + 'static,
    C: Clone + std::fmt::Debug + PartialEq + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> (A, B, C) {
        (
            self.0.generate(rng, config),
            self.1.generate(rng, config),
            self.2.generate(rng, config),
        )
    }

    fn shrink(&self, value: &(A, B, C)) -> Box<dyn Iterator<Item = (A, B, C)>> {
        let mut shrinks = Vec::new();

        for a_shrunk in self.0.shrink(&value.0) {
            shrinks.push((a_shrunk, value.1.clone(), value.2.clone()));
        }

        for b_shrunk in self.1.shrink(&value.1) {
            shrinks.push((value.0.clone(), b_shrunk, value.2.clone()));
        }

        for c_shrunk in self.2.shrink(&value.2) {
            shrinks.push((value.0.clone(), value.1.clone(), c_shrunk));
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// HashSet Generator
// ============================================================================

/// Generator for HashSet collections
#[derive(Debug, Clone)]
pub struct HashSetGenerator<T, G> {
    element_generator: G,
    min_size: usize,
    max_size: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G> HashSetGenerator<T, G>
where
    T: std::hash::Hash + Eq + Clone,
    G: Generator<T>,
{
    /// Create a new HashSet generator
    pub fn new(element_generator: G, min_size: usize, max_size: usize) -> Self {
        Self {
            element_generator,
            min_size,
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G> Generator<HashSet<T>> for HashSetGenerator<T, G>
where
    T: std::hash::Hash + Eq + Clone + 'static,
    G: Generator<T> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> HashSet<T> {
        use rand::Rng;
        let size = rng.r#gen_range(self.min_size..=self.max_size);
        let mut set = HashSet::new();

        // Try to generate unique elements, but don't loop forever
        let mut attempts = 0;
        let max_attempts = size * 10;

        while set.len() < size && attempts < max_attempts {
            let element = self.element_generator.generate(rng, config);
            set.insert(element);
            attempts += 1;
        }

        set
    }

    fn shrink(&self, value: &HashSet<T>) -> Box<dyn Iterator<Item = HashSet<T>>> {
        let mut shrinks = Vec::new();

        // Shrink to empty set
        if !value.is_empty() {
            shrinks.push(HashSet::new());
        }

        // Shrink to smaller sets by removing elements
        if value.len() > 1 {
            for elem in value.iter().take(value.len() / 2) {
                let mut smaller = value.clone();
                smaller.remove(elem);
                shrinks.push(smaller);
            }
        }

        // Shrink individual elements
        for elem in value {
            for shrunk_elem in self.element_generator.shrink(elem) {
                let mut shrunk_set = value.clone();
                shrunk_set.remove(elem);
                shrunk_set.insert(shrunk_elem);
                if shrunk_set != *value {
                    shrinks.push(shrunk_set);
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// BTreeMap Generator
// ============================================================================

/// Generator for BTreeMap collections
#[derive(Debug, Clone)]
pub struct BTreeMapGenerator<K, V, KG, VG> {
    key_generator: KG,
    value_generator: VG,
    min_size: usize,
    max_size: usize,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V, KG, VG> BTreeMapGenerator<K, V, KG, VG>
where
    K: Ord + Clone,
    V: Clone,
    KG: Generator<K>,
    VG: Generator<V>,
{
    /// Create a new BTreeMap generator
    pub fn new(key_generator: KG, value_generator: VG, min_size: usize, max_size: usize) -> Self {
        Self {
            key_generator,
            value_generator,
            min_size,
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K, V, KG, VG> Generator<BTreeMap<K, V>> for BTreeMapGenerator<K, V, KG, VG>
where
    K: Ord + Clone + 'static,
    V: Clone + 'static,
    KG: Generator<K> + Clone + 'static,
    VG: Generator<V> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> BTreeMap<K, V> {
        use rand::Rng;
        let size = rng.r#gen_range(self.min_size..=self.max_size);
        let mut map = BTreeMap::new();

        // Try to generate unique keys
        let mut attempts = 0;
        let max_attempts = size * 10;

        while map.len() < size && attempts < max_attempts {
            let key = self.key_generator.generate(rng, config);
            let value = self.value_generator.generate(rng, config);
            map.insert(key, value);
            attempts += 1;
        }

        map
    }

    fn shrink(&self, value: &BTreeMap<K, V>) -> Box<dyn Iterator<Item = BTreeMap<K, V>>> {
        let mut shrinks = Vec::new();

        // Shrink to empty map
        if !value.is_empty() {
            shrinks.push(BTreeMap::new());
        }

        // Shrink by removing entries
        if value.len() > 1 {
            let keys_to_remove: Vec<_> = value.keys().take(value.len() / 2).cloned().collect();
            for key in keys_to_remove {
                let mut smaller = value.clone();
                smaller.remove(&key);
                shrinks.push(smaller);
            }
        }

        // Shrink values (keep keys the same)
        for (key, val) in value {
            for shrunk_val in self.value_generator.shrink(val) {
                let mut shrunk_map = value.clone();
                shrunk_map.insert(key.clone(), shrunk_val);
                shrinks.push(shrunk_map);
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// BTreeSet Generator
// ============================================================================

/// Generator for BTreeSet collections
#[derive(Debug, Clone)]
pub struct BTreeSetGenerator<T, G> {
    element_generator: G,
    min_size: usize,
    max_size: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G> BTreeSetGenerator<T, G>
where
    T: Ord + Clone,
    G: Generator<T>,
{
    /// Create a new BTreeSet generator
    pub fn new(element_generator: G, min_size: usize, max_size: usize) -> Self {
        Self {
            element_generator,
            min_size,
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G> Generator<BTreeSet<T>> for BTreeSetGenerator<T, G>
where
    T: Ord + Clone + 'static,
    G: Generator<T> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> BTreeSet<T> {
        use rand::Rng;
        let size = rng.r#gen_range(self.min_size..=self.max_size);
        let mut set = BTreeSet::new();

        // Try to generate unique elements
        let mut attempts = 0;
        let max_attempts = size * 10;

        while set.len() < size && attempts < max_attempts {
            let element = self.element_generator.generate(rng, config);
            set.insert(element);
            attempts += 1;
        }

        set
    }

    fn shrink(&self, value: &BTreeSet<T>) -> Box<dyn Iterator<Item = BTreeSet<T>>> {
        let mut shrinks = Vec::new();

        // Shrink to empty set
        if !value.is_empty() {
            shrinks.push(BTreeSet::new());
        }

        // Shrink to smaller sets
        if value.len() > 1 {
            let elems_to_remove: Vec<_> = value.iter().take(value.len() / 2).cloned().collect();
            for elem in elems_to_remove {
                let mut smaller = value.clone();
                smaller.remove(&elem);
                shrinks.push(smaller);
            }
        }

        // Shrink individual elements
        for elem in value {
            for shrunk_elem in self.element_generator.shrink(elem) {
                let mut shrunk_set = value.clone();
                shrunk_set.remove(elem);
                shrunk_set.insert(shrunk_elem.clone());
                if shrunk_set != *value {
                    shrinks.push(shrunk_set);
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Result Generator
// ============================================================================

/// Generator for `Result<T, E>` values
#[derive(Debug, Clone)]
pub struct ResultGenerator<T, E, TG, EG> {
    ok_generator: TG,
    err_generator: EG,
    ok_probability: f64,
    _phantom: std::marker::PhantomData<(T, E)>,
}

impl<T, E, TG, EG> ResultGenerator<T, E, TG, EG>
where
    T: Clone,
    E: Clone,
    TG: Generator<T>,
    EG: Generator<E>,
{
    /// Create a new Result generator with 50/50 Ok/Err probability
    pub fn new(ok_generator: TG, err_generator: EG) -> Self {
        Self {
            ok_generator,
            err_generator,
            ok_probability: 0.5,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a Result generator with custom Ok probability (0.0 to 1.0)
    pub fn with_ok_probability(ok_generator: TG, err_generator: EG, ok_probability: f64) -> Self {
        Self {
            ok_generator,
            err_generator,
            ok_probability: ok_probability.clamp(0.0, 1.0),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, E, TG, EG> Generator<Result<T, E>> for ResultGenerator<T, E, TG, EG>
where
    T: Clone + 'static,
    E: Clone + 'static,
    TG: Generator<T> + Clone + 'static,
    EG: Generator<E> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Result<T, E> {
        use rand::Rng;
        if rng.r#gen::<f64>() < self.ok_probability {
            Ok(self.ok_generator.generate(rng, config))
        } else {
            Err(self.err_generator.generate(rng, config))
        }
    }

    fn shrink(&self, value: &Result<T, E>) -> Box<dyn Iterator<Item = Result<T, E>>> {
        match value {
            Ok(t) => {
                let shrinks: Vec<_> = self.ok_generator.shrink(t).map(Ok).collect();
                Box::new(shrinks.into_iter())
            }
            Err(e) => {
                // First try to shrink to Ok(default-ish value) if possible
                let mut shrinks = vec![];

                // Then shrink the error value
                shrinks.extend(self.err_generator.shrink(e).map(Err));

                Box::new(shrinks.into_iter())
            }
        }
    }
}

// ============================================================================
// Unit () Generator
// ============================================================================

/// Generator for the unit type ()
#[derive(Debug, Clone, Copy)]
pub struct UnitGenerator;

impl Generator<()> for UnitGenerator {
    fn generate(&self, _rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) {
        // Nothing to generate for unit type
    }

    fn shrink(&self, _value: &()) -> Box<dyn Iterator<Item = ()>> {
        // Unit type has no shrinks
        Box::new(std::iter::empty())
    }
}

// ============================================================================
// Array Generator for fixed-size arrays
// ============================================================================

/// Generator for fixed-size arrays [T; N]
#[derive(Debug, Clone)]
pub struct ArrayGenerator<T, G, const N: usize> {
    element_generator: G,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, G, const N: usize> ArrayGenerator<T, G, N>
where
    G: Generator<T>,
{
    /// Create a new array generator
    pub fn new(element_generator: G) -> Self {
        Self {
            element_generator,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, G, const N: usize> Generator<[T; N]> for ArrayGenerator<T, G, N>
where
    T: Clone + Default + 'static,
    G: Generator<T> + Clone + 'static,
{
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> [T; N] {
        // Use Default::default() to initialize, then fill
        let mut arr = std::array::from_fn(|_| T::default());
        for elem in arr.iter_mut() {
            *elem = self.element_generator.generate(rng, config);
        }
        arr
    }

    fn shrink(&self, value: &[T; N]) -> Box<dyn Iterator<Item = [T; N]>> {
        let mut shrinks = Vec::new();

        // Shrink individual elements
        for i in 0..N {
            for shrunk_elem in self.element_generator.shrink(&value[i]) {
                let mut shrunk_arr = value.clone();
                shrunk_arr[i] = shrunk_elem;
                shrinks.push(shrunk_arr);
            }
        }

        Box::new(shrinks.into_iter())
    }
}

#[cfg(test)]
mod new_generator_tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_hashset_generator() {
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let generator = HashSetGenerator::new(IntGenerator::new(1, 100), 0, 10);
        let set = generator.generate(&mut rng, &config);

        assert!(set.len() <= 10);
        for elem in &set {
            assert!(*elem >= 1 && *elem <= 100);
        }
    }

    #[test]
    fn test_btreemap_generator() {
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let generator = BTreeMapGenerator::new(
            IntGenerator::new(1, 10),
            StringGenerator::ascii_printable(1, 5),
            0,
            5,
        );
        let map = generator.generate(&mut rng, &config);

        assert!(map.len() <= 5);
        for (key, value) in &map {
            assert!(*key >= 1 && *key <= 10);
            assert!(!value.is_empty() && value.len() <= 5);
        }
    }

    #[test]
    fn test_btreeset_generator() {
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let generator = BTreeSetGenerator::new(IntGenerator::new(1, 50), 0, 8);
        let set = generator.generate(&mut rng, &config);

        assert!(set.len() <= 8);
        for elem in &set {
            assert!(*elem >= 1 && *elem <= 50);
        }
    }

    #[test]
    fn test_result_generator() {
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let generator = ResultGenerator::new(
            IntGenerator::new(1, 100),
            StringGenerator::ascii_printable(1, 10),
        );

        // Generate multiple to test both Ok and Err cases
        let mut ok_count = 0;
        let mut err_count = 0;

        for _ in 0..100 {
            match generator.generate(&mut rng, &config) {
                Ok(n) => {
                    assert!((1..=100).contains(&n));
                    ok_count += 1;
                }
                Err(s) => {
                    assert!(!s.is_empty() && s.len() <= 10);
                    err_count += 1;
                }
            }
        }

        // Should have both Ok and Err with 50/50 probability
        assert!(ok_count > 0);
        assert!(err_count > 0);
    }

    #[test]
    fn test_unit_generator() {
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let generator = UnitGenerator;
        generator.generate(&mut rng, &config);
    }

    #[test]
    fn test_array_generator() {
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let generator = ArrayGenerator::<i32, _, 5>::new(IntGenerator::new(1, 10));
        let arr = generator.generate(&mut rng, &config);

        assert_eq!(arr.len(), 5);
        for elem in &arr {
            assert!(*elem >= 1 && *elem <= 10);
        }
    }

    #[test]
    fn test_hashset_shrinking() {
        let generator = HashSetGenerator::new(IntGenerator::new(1, 100), 0, 10);

        let mut set = HashSet::new();
        set.insert(42);
        set.insert(43);
        set.insert(44);

        let shrinks: Vec<_> = generator.shrink(&set).collect();

        // Should have empty set as first shrink
        assert!(shrinks.iter().any(|s| s.is_empty()));
    }

    #[test]
    fn test_result_shrinking() {
        let generator = ResultGenerator::new(
            IntGenerator::new(0, 100),
            StringGenerator::ascii_printable(0, 10),
        );

        let ok_value: Result<i32, String> = Ok(50);
        let shrinks: Vec<_> = generator.shrink(&ok_value).collect();

        // Should shrink the Ok value towards 0
        assert!(!shrinks.is_empty());
    }
}
