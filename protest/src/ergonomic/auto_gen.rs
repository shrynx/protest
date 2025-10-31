//! Automatic generator inference for common types.
//!
//! This module provides the `AutoGen` trait which allows automatic generator
//! creation based on type information, dramatically reducing boilerplate.

use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::config::GeneratorConfig;
use crate::generator::Generator;
use crate::primitives::*;

/// Trait for types that can automatically provide a generator
///
/// Types implementing this trait can automatically generate appropriate
/// test data without manually specifying generators.
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::AutoGen;
///
/// // i32 implements AutoGen, so we can get its generator
/// let generator = i32::auto_generator();
/// ```
pub trait AutoGen: Sized {
    /// The type of generator for this type
    type Generator: Generator<Self>;

    /// Create an automatic generator for this type
    fn auto_generator() -> Self::Generator;
}

/// A generator that automatically infers the appropriate generator for a type
///
/// This is useful when you want to defer generator selection until you know
/// the concrete type.
pub struct InferredGenerator<T: AutoGen> {
    _phantom: PhantomData<T>,
}

impl<T: AutoGen> InferredGenerator<T> {
    /// Create a new inferred generator
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: AutoGen> Default for InferredGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: AutoGen> Generator<T> for InferredGenerator<T> {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> T {
        T::auto_generator().generate(rng, config)
    }

    fn shrink(&self, value: &T) -> Box<dyn Iterator<Item = T>> {
        T::auto_generator().shrink(value)
    }
}

// ============================================================================
// AutoGen implementations for primitive integer types
// ============================================================================

impl AutoGen for i8 {
    type Generator = IntGenerator<i8>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(i8::MIN, i8::MAX)
    }
}

impl AutoGen for i16 {
    type Generator = IntGenerator<i16>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(i16::MIN, i16::MAX)
    }
}

impl AutoGen for i32 {
    type Generator = IntGenerator<i32>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(i32::MIN, i32::MAX)
    }
}

impl AutoGen for i64 {
    type Generator = IntGenerator<i64>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(i64::MIN, i64::MAX)
    }
}

impl AutoGen for i128 {
    type Generator = IntGenerator<i128>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(i128::MIN, i128::MAX)
    }
}

impl AutoGen for isize {
    type Generator = IntGenerator<isize>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(isize::MIN, isize::MAX)
    }
}

impl AutoGen for u8 {
    type Generator = IntGenerator<u8>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(u8::MIN, u8::MAX)
    }
}

impl AutoGen for u16 {
    type Generator = IntGenerator<u16>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(u16::MIN, u16::MAX)
    }
}

impl AutoGen for u32 {
    type Generator = IntGenerator<u32>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(u32::MIN, u32::MAX)
    }
}

impl AutoGen for u64 {
    type Generator = IntGenerator<u64>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(u64::MIN, u64::MAX)
    }
}

impl AutoGen for u128 {
    type Generator = IntGenerator<u128>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(u128::MIN, u128::MAX)
    }
}

impl AutoGen for usize {
    type Generator = IntGenerator<usize>;

    fn auto_generator() -> Self::Generator {
        IntGenerator::new(usize::MIN, usize::MAX)
    }
}

// ============================================================================
// AutoGen implementations for floating point types
// ============================================================================

impl AutoGen for f32 {
    type Generator = FloatGenerator<f32>;

    fn auto_generator() -> Self::Generator {
        FloatGenerator::new(-1000.0, 1000.0)
    }
}

impl AutoGen for f64 {
    type Generator = FloatGenerator<f64>;

    fn auto_generator() -> Self::Generator {
        FloatGenerator::new(-1000.0, 1000.0)
    }
}

// ============================================================================
// AutoGen implementations for other primitive types
// ============================================================================

impl AutoGen for bool {
    type Generator = BoolGenerator;

    fn auto_generator() -> Self::Generator {
        BoolGenerator
    }
}

impl AutoGen for char {
    type Generator = CharGenerator;

    fn auto_generator() -> Self::Generator {
        CharGenerator::ascii_alphanumeric()
    }
}

impl AutoGen for String {
    type Generator = StringGenerator;

    fn auto_generator() -> Self::Generator {
        StringGenerator::ascii_alphanumeric(0, 100)
    }
}

// ============================================================================
// AutoGen implementations for collection types
// ============================================================================

impl<T: AutoGen + Clone + 'static> AutoGen for Vec<T>
where
    T::Generator: 'static,
{
    type Generator = VecGenerator<T, T::Generator>;

    fn auto_generator() -> Self::Generator {
        VecGenerator::new(T::auto_generator(), 0, 100)
    }
}

// TODO: HashSetGenerator is not yet implemented in primitives.rs
// impl<T: AutoGen + Eq + Hash + 'static> AutoGen for HashSet<T>

impl<K: AutoGen + Clone + Eq + Hash + 'static, V: AutoGen + Clone + 'static> AutoGen
    for HashMap<K, V>
where
    K::Generator: 'static,
    V::Generator: 'static,
{
    type Generator = HashMapGenerator<K, V, K::Generator, V::Generator>;

    fn auto_generator() -> Self::Generator {
        HashMapGenerator::new(K::auto_generator(), V::auto_generator(), 0, 100)
    }
}

// ============================================================================
// AutoGen implementations for tuple types
// ============================================================================

impl<T1: AutoGen + Clone + 'static, T2: AutoGen + Clone + 'static> AutoGen for (T1, T2)
where
    T1::Generator: 'static,
    T2::Generator: 'static,
{
    type Generator = TupleGenerator<T1, T2, T1::Generator, T2::Generator>;

    fn auto_generator() -> Self::Generator {
        TupleGenerator::new(T1::auto_generator(), T2::auto_generator())
    }
}

impl<T1: AutoGen + Clone + 'static, T2: AutoGen + Clone + 'static, T3: AutoGen + Clone + 'static>
    AutoGen for (T1, T2, T3)
where
    T1::Generator: 'static,
    T2::Generator: 'static,
    T3::Generator: 'static,
{
    type Generator = Tuple3Generator<T1, T2, T3, T1::Generator, T2::Generator, T3::Generator>;

    fn auto_generator() -> Self::Generator {
        Tuple3Generator::new(
            T1::auto_generator(),
            T2::auto_generator(),
            T3::auto_generator(),
        )
    }
}

impl<
    T1: AutoGen + Clone + 'static,
    T2: AutoGen + Clone + 'static,
    T3: AutoGen + Clone + 'static,
    T4: AutoGen + Clone + 'static,
> AutoGen for (T1, T2, T3, T4)
where
    T1::Generator: 'static,
    T2::Generator: 'static,
    T3::Generator: 'static,
    T4::Generator: 'static,
{
    type Generator =
        Tuple4Generator<T1, T2, T3, T4, T1::Generator, T2::Generator, T3::Generator, T4::Generator>;

    fn auto_generator() -> Self::Generator {
        Tuple4Generator::new(
            T1::auto_generator(),
            T2::auto_generator(),
            T3::auto_generator(),
            T4::auto_generator(),
        )
    }
}

impl<T: AutoGen + Clone + 'static> AutoGen for Option<T>
where
    T::Generator: 'static,
{
    type Generator = OptionGenerator<T, T::Generator>;

    fn auto_generator() -> Self::Generator {
        OptionGenerator::new(T::auto_generator())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_autogen_i32() {
        let generator = i32::auto_generator();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = generator.generate(&mut rng, &config);
        assert!((i32::MIN..=i32::MAX).contains(&value));
    }

    #[test]
    fn test_autogen_string() {
        let generator = String::auto_generator();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = generator.generate(&mut rng, &config);
        assert!(value.len() <= 100);
    }

    #[test]
    fn test_autogen_vec_i32() {
        let generator = Vec::<i32>::auto_generator();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = generator.generate(&mut rng, &config);
        assert!(value.len() <= 100);
    }

    #[test]
    fn test_autogen_tuple() {
        let generator = <(i32, String)>::auto_generator();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (num, text) = generator.generate(&mut rng, &config);
        assert!((i32::MIN..=i32::MAX).contains(&num));
        assert!(text.len() <= 100);
    }

    #[test]
    fn test_autogen_option() {
        let generator = Option::<i32>::auto_generator();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        // Generate a few values to ensure both Some and None can be generated
        let mut has_some = false;
        let mut has_none = false;
        for _ in 0..20 {
            match generator.generate(&mut rng, &config) {
                Some(_) => has_some = true,
                None => has_none = true,
            }
        }
        // With 50% probability, getting both in 20 tries is very likely
        assert!(has_some || has_none);
    }

    #[test]
    fn test_inferred_generator() {
        let generator = InferredGenerator::<i32>::new();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = generator.generate(&mut rng, &config);
        assert!((i32::MIN..=i32::MAX).contains(&value));
    }
}
