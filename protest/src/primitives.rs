//! Generators for primitive types and basic collections.

use std::collections::HashMap;

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

        // Shrink towards empty string
        if value.len() > self.min_length {
            shrinks.push("".to_string());
        }

        // Shrink by removing characters
        if value.len() > self.min_length {
            // Remove from the end
            for i in (self.min_length..value.len()).rev() {
                shrinks.push(value.chars().take(i).collect());
            }
        }

        // Shrink individual characters
        if !value.is_empty() {
            let chars: Vec<char> = value.chars().collect();
            for (i, &c) in chars.iter().enumerate() {
                for shrunk_char in self.char_generator.shrink(&c) {
                    let mut new_chars = chars.clone();
                    new_chars[i] = shrunk_char;
                    shrinks.push(new_chars.into_iter().collect());
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

        // Shrink by removing elements
        if value.len() > self.min_length {
            for i in (self.min_length..value.len()).rev() {
                shrinks.push(value.iter().take(i).cloned().collect());
            }
        }

        Box::new(shrinks.into_iter())
    }
}

/// Generator for HashMap<K, V> collections
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
    fn test_full_range_generators() {
        let i32_gen = IntGenerator::<i32>::full_range();
        let f64_gen = FloatGenerator::<f64>::reasonable_range();

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Just test that they can generate values without panicking
        let _int_val = i32_gen.generate(&mut rng, &config);
        let _float_val = f64_gen.generate(&mut rng, &config);
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

/// Generator for Option<T>
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
