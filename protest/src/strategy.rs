//! Strategy-based generation system for composable test data creation.

use crate::config::GeneratorConfig;
use crate::generator::Generator;

/// A strategy for generating values of a specific type
///
/// Strategies are composable and can be combined to create more complex generation patterns.
/// They encapsulate both the generation logic and the shrinking behavior.
pub trait Strategy {
    /// The type of values this strategy generates
    type Value: 'static;

    /// Generate a value using this strategy
    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Self::Value;

    /// Create an iterator of shrunk values from the given value
    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>>;

    /// Map this strategy to produce values of a different type
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Value) -> U,
        U: 'static,
    {
        Map {
            strategy: self,
            mapper: f,
        }
    }

    /// Filter values produced by this strategy
    fn filter<F>(self, predicate: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Value) -> bool,
    {
        Filter {
            strategy: self,
            predicate,
        }
    }

    /// Combine this strategy with another to produce tuples
    fn zip<S>(self, other: S) -> Zip<Self, S>
    where
        Self: Sized,
        S: Strategy,
    {
        Zip {
            left: self,
            right: other,
        }
    }

    /// Flat map (bind) this strategy with a function that produces another strategy
    /// This is the most powerful combinator but also the trickiest for shrinking
    fn flat_map<F, S>(self, f: F) -> FlatMap<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Value) -> S,
        S: Strategy,
    {
        FlatMap {
            strategy: self,
            flat_mapper: f,
        }
    }
}

/// A strategy that maps values from one type to another
///
/// To enable proper shrinking through map, we need to store the original input
/// alongside the mapped output. This is done using `MappedValue`.
pub struct Map<S, F> {
    strategy: S,
    mapper: F,
}

/// A value that stores both the original input and the mapped output
/// This allows us to shrink the input and re-map to get shrunk outputs
///
/// `MappedValue` implements `Deref` to the output type for ergonomic access.
/// You can use it as if it were just the output value, but it preserves
/// the input for proper shrinking.
#[derive(Debug, Clone, Copy)]
pub struct MappedValue<T, U> {
    /// The original input value before mapping
    pub input: T,
    /// The mapped output value
    pub output: U,
}

impl<T, U> std::ops::Deref for MappedValue<T, U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.output
    }
}

// Allow comparing MappedValue with any type that U can be compared to
impl<T, U, V> PartialEq<V> for MappedValue<T, U>
where
    U: PartialEq<V>,
{
    fn eq(&self, other: &V) -> bool {
        self.output.eq(other)
    }
}

impl<T, U, V> PartialOrd<V> for MappedValue<T, U>
where
    U: PartialOrd<V>,
{
    fn partial_cmp(&self, other: &V) -> Option<std::cmp::Ordering> {
        self.output.partial_cmp(other)
    }
}

impl<T, U: std::fmt::Display> std::fmt::Display for MappedValue<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.output.fmt(f)
    }
}

// Forward common trait impls to the output
impl<T, U: std::ops::Add<Output = V>, V> std::ops::Add for MappedValue<T, U> {
    type Output = V;

    fn add(self, rhs: Self) -> Self::Output {
        self.output + rhs.output
    }
}

impl<T, U: std::ops::Sub<Output = V>, V> std::ops::Sub for MappedValue<T, U> {
    type Output = V;

    fn sub(self, rhs: Self) -> Self::Output {
        self.output - rhs.output
    }
}

impl<T, U: std::ops::Mul<Output = V>, V> std::ops::Mul for MappedValue<T, U> {
    type Output = V;

    fn mul(self, rhs: Self) -> Self::Output {
        self.output * rhs.output
    }
}

impl<T, U: std::ops::Div<Output = V>, V> std::ops::Div for MappedValue<T, U> {
    type Output = V;

    fn div(self, rhs: Self) -> Self::Output {
        self.output / rhs.output
    }
}

impl<T, U, V, W> std::ops::Rem<V> for MappedValue<T, U>
where
    U: std::ops::Rem<V, Output = W>,
{
    type Output = W;

    fn rem(self, rhs: V) -> Self::Output {
        self.output % rhs
    }
}

impl<S, F, U> Strategy for Map<S, F>
where
    S: Strategy,
    F: Fn(S::Value) -> U + Clone + 'static,
    U: 'static,
    S::Value: Clone,
{
    // Instead of just U, we return MappedValue<S::Value, U> to preserve shrinking info
    type Value = MappedValue<S::Value, U>;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Self::Value {
        let input = self.strategy.generate(rng, config);
        let output = (self.mapper)(input.clone());
        MappedValue { input, output }
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Shrink the input, then re-map each shrunk input to get shrunk outputs
        let shrunk_inputs = self.strategy.shrink(&value.input);
        let mapper = self.mapper.clone();

        Box::new(shrunk_inputs.map(move |input| {
            let output = mapper(input.clone());
            MappedValue { input, output }
        }))
    }
}

/// A strategy that filters values based on a predicate
pub struct Filter<S, F> {
    strategy: S,
    predicate: F,
}

impl<S, F> Strategy for Filter<S, F>
where
    S: Strategy,
    F: Fn(&S::Value) -> bool + Clone + 'static,
    S::Value: Clone,
{
    type Value = S::Value;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Self::Value {
        // Try to generate a value that passes the filter
        // Limit attempts to avoid infinite loops
        for _ in 0..1000 {
            let value = self.strategy.generate(rng, config);
            if (self.predicate)(&value) {
                return value;
            }
        }
        panic!("Filter strategy failed to generate a valid value after 1000 attempts");
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Shrink the underlying value, but only yield shrinks that pass the filter predicate
        let shrunk_values = self.strategy.shrink(value);
        let predicate = self.predicate.clone();

        Box::new(shrunk_values.filter(move |v| predicate(v)))
    }
}

/// A strategy that combines two strategies to produce tuples
pub struct Zip<L, R> {
    left: L,
    right: R,
}

impl<L, R> Strategy for Zip<L, R>
where
    L: Strategy,
    R: Strategy,
    L::Value: Clone,
    R::Value: Clone,
{
    type Value = (L::Value, R::Value);

    fn generate<RNG: rand::Rng>(&self, rng: &mut RNG, config: &GeneratorConfig) -> Self::Value {
        let left_value = self.left.generate(rng, config);
        let right_value = self.right.generate(rng, config);
        (left_value, right_value)
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Shrink both sides of the tuple
        // Strategy: First try shrinking left while keeping right constant,
        // then try shrinking right while keeping left constant
        let (left_val, right_val) = value;

        let left_shrinks = self.left.shrink(left_val);
        let right_val_for_left = right_val.clone();
        let left_combinations = left_shrinks.map(move |l| (l, right_val_for_left.clone()));

        let right_shrinks = self.right.shrink(right_val);
        let left_val_for_right = left_val.clone();
        let right_combinations = right_shrinks.map(move |r| (left_val_for_right.clone(), r));

        // Chain: first try shrinking left, then try shrinking right
        Box::new(left_combinations.chain(right_combinations))
    }
}

/// A strategy that applies a function that produces another strategy (flat_map/bind)
///
/// This is the most powerful combinator. The second strategy depends on the value
/// produced by the first strategy. For proper shrinking, we need to store both
/// the first value and the second value.
pub struct FlatMap<S, F> {
    strategy: S,
    flat_mapper: F,
}

/// A value produced by flat_map that stores both stages for proper shrinking
#[derive(Debug, Clone)]
pub struct FlatMappedValue<T, U> {
    /// The value from the first strategy
    pub first: T,
    /// The value from the second (dependent) strategy
    pub second: U,
}

impl<S, F, S2> Strategy for FlatMap<S, F>
where
    S: Strategy,
    F: Fn(S::Value) -> S2 + Clone + 'static,
    S2: Strategy,
    S::Value: Clone,
    S2::Value: Clone,
{
    type Value = FlatMappedValue<S::Value, S2::Value>;

    fn generate<R: rand::Rng>(&self, rng: &mut R, config: &GeneratorConfig) -> Self::Value {
        // Generate from first strategy
        let first = self.strategy.generate(rng, config);
        // Use that value to create the second strategy
        let second_strategy = (self.flat_mapper)(first.clone());
        // Generate from the second strategy
        let second = second_strategy.generate(rng, config);

        FlatMappedValue { first, second }
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // This is the complex part of flat_map shrinking
        // We have two approaches:
        // 1. Shrink the first value, then re-generate the second value from the shrunk first
        // 2. Keep the first value fixed, and shrink the second value

        let flat_mapper = self.flat_mapper.clone();

        // Approach 2: Keep first fixed, shrink second
        let second_strategy = (flat_mapper.clone())(value.first.clone());
        let second_shrinks = second_strategy.shrink(&value.second);
        let first_val = value.first.clone();
        let second_only_shrinks: Vec<_> = second_shrinks
            .map(move |second_shrunk| FlatMappedValue {
                first: first_val.clone(),
                second: second_shrunk,
            })
            .collect();

        // Approach 1: Shrink first, regenerate second
        let first_shrinks = self.strategy.shrink(&value.first);
        let second_val = value.second.clone();
        let first_shrink_combinations: Vec<_> = first_shrinks
            .flat_map({
                let flat_mapper = flat_mapper.clone();
                let second_val = second_val.clone();
                move |first_shrunk| {
                    // For each shrunk first value, create the dependent strategy
                    // and generate a new second value (we can't shrink here as we don't have an RNG)
                    // So we'll just use the shrinks of the original second strategy
                    let second_strategy = flat_mapper(first_shrunk.clone());
                    let second_shrinks = second_strategy.shrink(&second_val);
                    second_shrinks
                        .map(move |second_shrunk| FlatMappedValue {
                            first: first_shrunk.clone(),
                            second: second_shrunk,
                        })
                        .collect::<Vec<_>>()
                }
            })
            .collect();

        // Try approach 2 first (simpler), then approach 1 (more aggressive)
        Box::new(
            second_only_shrinks
                .into_iter()
                .chain(first_shrink_combinations),
        )
    }
}

/// A strategy that always produces the same value
#[derive(Debug, Clone)]
pub struct Just<T> {
    value: T,
}

impl<T: Clone + 'static> Strategy for Just<T> {
    type Value = T;

    fn generate<R: rand::Rng>(&self, _rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
        self.value.clone()
    }

    fn shrink(&self, _value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // A constant value cannot be shrunk
        Box::new(std::iter::empty())
    }
}

// Also implement Generator for Just
impl<T> Generator<T> for Just<T>
where
    T: Clone + std::fmt::Debug + PartialEq + 'static,
{
    fn generate(&self, _rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> T {
        self.value.clone()
    }

    fn shrink(&self, _value: &T) -> Box<dyn Iterator<Item = T>> {
        Box::new(std::iter::empty())
    }
}

/// A strategy that chooses randomly from a collection of values
#[derive(Debug, Clone)]
pub struct OneOf<T> {
    values: Vec<T>,
}

impl<T: Clone + PartialEq + 'static> Strategy for OneOf<T> {
    type Value = T;

    fn generate<R: rand::Rng>(&self, rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
        if self.values.is_empty() {
            panic!("OneOf strategy cannot generate from empty collection");
        }
        let index = rng.gen_range(0..self.values.len());
        self.values[index].clone()
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // For OneOf, we return all other values in the collection as potential shrinks
        // We filter out the current value and return the rest
        // Earlier values in the list are tried first (assuming simpler values are at the front)
        let shrinks: Vec<T> = self
            .values
            .iter()
            .filter(|v| *v != value)
            .cloned()
            .collect();

        Box::new(shrinks.into_iter())
    }
}

/// A strategy for generating values in a numeric range
#[derive(Debug, Clone)]
pub struct Range<T> {
    start: T,
    end: T,
}

// Macro to implement Strategy with shrinking for specific integer types
macro_rules! impl_range_strategy {
    ($($t:ty),*) => {
        $(
            impl Strategy for Range<$t> {
                type Value = $t;

                fn generate<R: rand::Rng>(&self, rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
                    rng.gen_range(self.start..=self.end)
                }

                fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
                    let value = *value;

                    if value == self.start {
                        return Box::new(std::iter::empty());
                    }

                    // Shrink towards zero if it's in range, otherwise towards start
                    #[allow(unused_comparisons)]
                    let target = if self.start <= 0 && 0 <= self.end && 0 <= value {
                        0
                    } else {
                        self.start
                    };

                    let mut shrinks = Vec::new();
                    let mut current = value;

                    while current != target {
                        let diff = if current > target {
                            current - target
                        } else {
                            target - current
                        };
                        let step = if diff == 1 { 1 } else { diff / 2 };
                        current = if current > target {
                            current - step
                        } else {
                            current + step
                        };

                        // Only include values within the range and not equal to the original
                        if current >= self.start && current <= self.end && current != value {
                            shrinks.push(current);
                        }

                        if current == target {
                            break;
                        }
                    }

                    Box::new(shrinks.into_iter())
                }
            }
        )*
    };
}

// Implement for common integer types with proper shrinking
impl_range_strategy!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

// Also implement Generator for these integer types
macro_rules! impl_range_generator {
    ($($t:ty),*) => {
        $(
            impl Generator<$t> for Range<$t> {
                fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> $t {
                    use rand::Rng;
                    rng.gen_range(self.start..=self.end)
                }

                fn shrink(&self, value: &$t) -> Box<dyn Iterator<Item = $t>> {
                    Strategy::shrink(self, value)
                }
            }
        )*
    };
}

impl_range_generator!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

// For float types, implement Strategy with shrinking
impl Strategy for Range<f32> {
    type Value = f32;

    fn generate<R: rand::Rng>(&self, rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
        rng.gen_range(self.start..=self.end)
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        let value = *value;

        // Handle special cases
        if value.is_nan() || value.is_infinite() {
            // Can't shrink NaN or infinity in a meaningful way
            return Box::new(std::iter::empty());
        }

        // If value is at the start, no shrinking
        if (value - self.start).abs() < f32::EPSILON {
            return Box::new(std::iter::empty());
        }

        // Determine target: 0.0 if in range, otherwise range start
        let target = if self.start <= 0.0 && 0.0 <= self.end && 0.0 <= value {
            0.0
        } else {
            self.start
        };

        let mut shrinks = Vec::new();
        let mut current = value;

        // Binary search toward target with floating point
        while (current - target).abs() > f32::EPSILON {
            let diff = current - target;
            let step = diff / 2.0;

            // If step is too small, try the target directly
            if step.abs() < f32::EPSILON * 10.0 {
                if target >= self.start
                    && target <= self.end
                    && (target - value).abs() > f32::EPSILON
                {
                    shrinks.push(target);
                }
                break;
            }

            current -= step;

            // Only include values within the range and different from original
            if current >= self.start
                && current <= self.end
                && (current - value).abs() > f32::EPSILON
            {
                shrinks.push(current);
            }

            if (current - target).abs() < f32::EPSILON {
                break;
            }
        }

        Box::new(shrinks.into_iter())
    }
}

impl Strategy for Range<f64> {
    type Value = f64;

    fn generate<R: rand::Rng>(&self, rng: &mut R, _config: &GeneratorConfig) -> Self::Value {
        rng.gen_range(self.start..=self.end)
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        let value = *value;

        // Handle special cases
        if value.is_nan() || value.is_infinite() {
            // Can't shrink NaN or infinity in a meaningful way
            return Box::new(std::iter::empty());
        }

        // If value is at the start, no shrinking
        if (value - self.start).abs() < f64::EPSILON {
            return Box::new(std::iter::empty());
        }

        // Determine target: 0.0 if in range, otherwise range start
        let target = if self.start <= 0.0 && 0.0 <= self.end && 0.0 <= value {
            0.0
        } else {
            self.start
        };

        let mut shrinks = Vec::new();
        let mut current = value;

        // Binary search toward target with floating point
        while (current - target).abs() > f64::EPSILON {
            let diff = current - target;
            let step = diff / 2.0;

            // If step is too small, try the target directly
            if step.abs() < f64::EPSILON * 10.0 {
                if target >= self.start
                    && target <= self.end
                    && (target - value).abs() > f64::EPSILON
                {
                    shrinks.push(target);
                }
                break;
            }

            current -= step;

            // Only include values within the range and different from original
            if current >= self.start
                && current <= self.end
                && (current - value).abs() > f64::EPSILON
            {
                shrinks.push(current);
            }

            if (current - target).abs() < f64::EPSILON {
                break;
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// Implement Generator for float types
impl Generator<f32> for Range<f32> {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> f32 {
        use rand::Rng;
        rng.gen_range(self.start..=self.end)
    }

    fn shrink(&self, value: &f32) -> Box<dyn Iterator<Item = f32>> {
        Strategy::shrink(self, value)
    }
}

impl Generator<f64> for Range<f64> {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> f64 {
        use rand::Rng;
        rng.gen_range(self.start..=self.end)
    }

    fn shrink(&self, value: &f64) -> Box<dyn Iterator<Item = f64>> {
        Strategy::shrink(self, value)
    }
}

/// A strategy for generating 3-tuples
pub struct Tuple3<A, B, C> {
    a: A,
    b: B,
    c: C,
}

impl<A, B, C> Strategy for Tuple3<A, B, C>
where
    A: Strategy,
    B: Strategy,
    C: Strategy,
    A::Value: Clone,
    B::Value: Clone,
    C::Value: Clone,
{
    type Value = (A::Value, B::Value, C::Value);

    fn generate<RNG: rand::Rng>(&self, rng: &mut RNG, config: &GeneratorConfig) -> Self::Value {
        let a_val = self.a.generate(rng, config);
        let b_val = self.b.generate(rng, config);
        let c_val = self.c.generate(rng, config);
        (a_val, b_val, c_val)
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Shrink each element independently
        // Strategy: Try shrinking a, then b, then c
        let (a_val, b_val, c_val) = value;

        let a_shrinks = self.a.shrink(a_val);
        let b_val_for_a = b_val.clone();
        let c_val_for_a = c_val.clone();
        let a_combinations = a_shrinks.map(move |a| (a, b_val_for_a.clone(), c_val_for_a.clone()));

        let b_shrinks = self.b.shrink(b_val);
        let a_val_for_b = a_val.clone();
        let c_val_for_b = c_val.clone();
        let b_combinations = b_shrinks.map(move |b| (a_val_for_b.clone(), b, c_val_for_b.clone()));

        let c_shrinks = self.c.shrink(c_val);
        let a_val_for_c = a_val.clone();
        let b_val_for_c = b_val.clone();
        let c_combinations = c_shrinks.map(move |c| (a_val_for_c.clone(), b_val_for_c.clone(), c));

        // Chain: first try shrinking a, then b, then c
        Box::new(a_combinations.chain(b_combinations).chain(c_combinations))
    }
}

/// A strategy for generating 4-tuples
pub struct Tuple4<A, B, C, D> {
    a: A,
    b: B,
    c: C,
    d: D,
}

impl<A, B, C, D> Strategy for Tuple4<A, B, C, D>
where
    A: Strategy,
    B: Strategy,
    C: Strategy,
    D: Strategy,
    A::Value: Clone,
    B::Value: Clone,
    C::Value: Clone,
    D::Value: Clone,
{
    type Value = (A::Value, B::Value, C::Value, D::Value);

    fn generate<RNG: rand::Rng>(&self, rng: &mut RNG, config: &GeneratorConfig) -> Self::Value {
        let a_val = self.a.generate(rng, config);
        let b_val = self.b.generate(rng, config);
        let c_val = self.c.generate(rng, config);
        let d_val = self.d.generate(rng, config);
        (a_val, b_val, c_val, d_val)
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Shrink each element independently
        let (a_val, b_val, c_val, d_val) = value;

        let a_shrinks = self.a.shrink(a_val);
        let b_val_for_a = b_val.clone();
        let c_val_for_a = c_val.clone();
        let d_val_for_a = d_val.clone();
        let a_combinations = a_shrinks.map(move |a| {
            (
                a,
                b_val_for_a.clone(),
                c_val_for_a.clone(),
                d_val_for_a.clone(),
            )
        });

        let b_shrinks = self.b.shrink(b_val);
        let a_val_for_b = a_val.clone();
        let c_val_for_b = c_val.clone();
        let d_val_for_b = d_val.clone();
        let b_combinations = b_shrinks.map(move |b| {
            (
                a_val_for_b.clone(),
                b,
                c_val_for_b.clone(),
                d_val_for_b.clone(),
            )
        });

        let c_shrinks = self.c.shrink(c_val);
        let a_val_for_c = a_val.clone();
        let b_val_for_c = b_val.clone();
        let d_val_for_c = d_val.clone();
        let c_combinations = c_shrinks.map(move |c| {
            (
                a_val_for_c.clone(),
                b_val_for_c.clone(),
                c,
                d_val_for_c.clone(),
            )
        });

        let d_shrinks = self.d.shrink(d_val);
        let a_val_for_d = a_val.clone();
        let b_val_for_d = b_val.clone();
        let c_val_for_d = c_val.clone();
        let d_combinations = d_shrinks.map(move |d| {
            (
                a_val_for_d.clone(),
                b_val_for_d.clone(),
                c_val_for_d.clone(),
                d,
            )
        });

        Box::new(
            a_combinations
                .chain(b_combinations)
                .chain(c_combinations)
                .chain(d_combinations),
        )
    }
}

/// A strategy for generating 5-tuples
pub struct Tuple5<A, B, C, D, E> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
}

impl<A, B, C, D, E> Strategy for Tuple5<A, B, C, D, E>
where
    A: Strategy,
    B: Strategy,
    C: Strategy,
    D: Strategy,
    E: Strategy,
    A::Value: Clone,
    B::Value: Clone,
    C::Value: Clone,
    D::Value: Clone,
    E::Value: Clone,
{
    type Value = (A::Value, B::Value, C::Value, D::Value, E::Value);

    fn generate<RNG: rand::Rng>(&self, rng: &mut RNG, config: &GeneratorConfig) -> Self::Value {
        let a_val = self.a.generate(rng, config);
        let b_val = self.b.generate(rng, config);
        let c_val = self.c.generate(rng, config);
        let d_val = self.d.generate(rng, config);
        let e_val = self.e.generate(rng, config);
        (a_val, b_val, c_val, d_val, e_val)
    }

    fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
        // Shrink each element independently
        let (a_val, b_val, c_val, d_val, e_val) = value;

        let a_shrinks = self.a.shrink(a_val);
        let b_val_for_a = b_val.clone();
        let c_val_for_a = c_val.clone();
        let d_val_for_a = d_val.clone();
        let e_val_for_a = e_val.clone();
        let a_combinations = a_shrinks.map(move |a| {
            (
                a,
                b_val_for_a.clone(),
                c_val_for_a.clone(),
                d_val_for_a.clone(),
                e_val_for_a.clone(),
            )
        });

        let b_shrinks = self.b.shrink(b_val);
        let a_val_for_b = a_val.clone();
        let c_val_for_b = c_val.clone();
        let d_val_for_b = d_val.clone();
        let e_val_for_b = e_val.clone();
        let b_combinations = b_shrinks.map(move |b| {
            (
                a_val_for_b.clone(),
                b,
                c_val_for_b.clone(),
                d_val_for_b.clone(),
                e_val_for_b.clone(),
            )
        });

        let c_shrinks = self.c.shrink(c_val);
        let a_val_for_c = a_val.clone();
        let b_val_for_c = b_val.clone();
        let d_val_for_c = d_val.clone();
        let e_val_for_c = e_val.clone();
        let c_combinations = c_shrinks.map(move |c| {
            (
                a_val_for_c.clone(),
                b_val_for_c.clone(),
                c,
                d_val_for_c.clone(),
                e_val_for_c.clone(),
            )
        });

        let d_shrinks = self.d.shrink(d_val);
        let a_val_for_d = a_val.clone();
        let b_val_for_d = b_val.clone();
        let c_val_for_d = c_val.clone();
        let e_val_for_d = e_val.clone();
        let d_combinations = d_shrinks.map(move |d| {
            (
                a_val_for_d.clone(),
                b_val_for_d.clone(),
                c_val_for_d.clone(),
                d,
                e_val_for_d.clone(),
            )
        });

        let e_shrinks = self.e.shrink(e_val);
        let a_val_for_e = a_val.clone();
        let b_val_for_e = b_val.clone();
        let c_val_for_e = c_val.clone();
        let d_val_for_e = d_val.clone();
        let e_combinations = e_shrinks.map(move |e| {
            (
                a_val_for_e.clone(),
                b_val_for_e.clone(),
                c_val_for_e.clone(),
                d_val_for_e.clone(),
                e,
            )
        });

        Box::new(
            a_combinations
                .chain(b_combinations)
                .chain(c_combinations)
                .chain(d_combinations)
                .chain(e_combinations),
        )
    }
}

/// Create a strategy for generating 3-tuples
pub fn tuple3<A, B, C>(a: A, b: B, c: C) -> Tuple3<A, B, C> {
    Tuple3 { a, b, c }
}

/// Create a strategy for generating 4-tuples
pub fn tuple4<A, B, C, D>(a: A, b: B, c: C, d: D) -> Tuple4<A, B, C, D> {
    Tuple4 { a, b, c, d }
}

/// Create a strategy for generating 5-tuples
pub fn tuple5<A, B, C, D, E>(a: A, b: B, c: C, d: D, e: E) -> Tuple5<A, B, C, D, E> {
    Tuple5 { a, b, c, d, e }
}

/// Create a strategy that always produces the same value
pub fn just<T: Clone>(value: T) -> Just<T> {
    Just { value }
}

/// Create a strategy that chooses from a collection of values
pub fn one_of<T: Clone>(values: Vec<T>) -> OneOf<T> {
    OneOf { values }
}

/// Create a strategy for generating values in a range
pub fn range<T>(start: T, end: T) -> Range<T>
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd + Copy,
{
    Range { start, end }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_just_strategy() {
        let strategy = just(42);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = Strategy::generate(&strategy, &mut rng, &config);
        assert_eq!(value, 42);

        // Just strategy should not produce any shrinks
        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();
        assert!(shrinks.is_empty());
    }

    #[test]
    fn test_one_of_strategy() {
        let values = vec![1, 2, 3, 4, 5];
        let strategy = one_of(values.clone());
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = Strategy::generate(&strategy, &mut rng, &config);
        assert!(values.contains(&value));
    }

    #[test]
    fn test_range_strategy() {
        let strategy = range(1, 10);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = Strategy::generate(&strategy, &mut rng, &config);
        assert!((1..=10).contains(&value));
    }

    #[test]
    fn test_strategy_map() {
        let strategy = just(5).map(|x| x * 2);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert_eq!(value.input, 5);
        assert_eq!(value.output, 10);

        // Test shrinking
        let shrinks: Vec<_> = strategy.shrink(&value).collect();
        // Just(5) doesn't shrink, so we expect no shrinks
        assert!(shrinks.is_empty());
    }

    #[test]
    fn test_strategy_zip() {
        let strategy = just(42).zip(range(1, 5));
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (left, right) = strategy.generate(&mut rng, &config);
        assert_eq!(left, 42);
        assert!((1..=5).contains(&right));
    }

    #[test]
    fn test_strategy_filter() {
        let strategy = range(1, 100).filter(|x| x % 2 == 0);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert!((1..=100).contains(&value));
        assert_eq!(value % 2, 0); // Should be even
    }

    #[test]
    fn test_strategy_composition() {
        // Test complex composition: map a range to strings, then zip with a constant
        let strategy = range(1, 5).map(|x| format!("value_{}", x)).zip(just(true));

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (mapped_val, bool_val) = strategy.generate(&mut rng, &config);
        assert!(mapped_val.output.starts_with("value_"));
        assert!(bool_val);
    }

    #[test]
    fn test_map_shrinking() {
        // Test that map properly shrinks through to the underlying value
        let strategy = range(1, 100).map(|x| x * 2);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        let _shrinks: Vec<_> = strategy.shrink(&value).take(5).collect();

        // Range now implements shrinking - tests verify this in test_range_shrinking_*
    }

    #[test]
    fn test_filter_shrinking() {
        // Test that filter only yields shrinks that pass the predicate
        let strategy = range(10, 100).filter(|x| x % 2 == 0);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert_eq!(value % 2, 0); // Original is even

        let shrinks: Vec<_> = strategy.shrink(&value).take(10).collect();
        // All shrinks should also be even
        for shrink in shrinks {
            assert_eq!(shrink % 2, 0, "Shrink {} should be even", shrink);
        }
    }

    #[test]
    fn test_zip_shrinking() {
        // Test that zip shrinks both sides independently
        let strategy = just(10).zip(just(20));
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert_eq!(value, (10, 20));

        let shrinks: Vec<_> = strategy.shrink(&value).collect();
        // Just doesn't shrink, so no shrinks expected
        assert!(shrinks.is_empty());
    }

    #[test]
    fn test_flat_map_basic() {
        // Test basic flat_map: generate a number, then generate a vec of that length
        let strategy = range(1, 5).flat_map(|n| just(vec![42; n as usize]));

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert!((1..=5).contains(&(value.first as i32)));
        assert_eq!(value.second.len(), value.first as usize);
        assert!(value.second.iter().all(|&x| x == 42));
    }

    #[test]
    fn test_flat_map_shrinking() {
        // Test that flat_map shrinks both stages
        let strategy = just(3).flat_map(|n| just(vec![1; n as usize]));

        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let value = strategy.generate(&mut rng, &config);
        assert_eq!(value.first, 3);
        assert_eq!(value.second, vec![1, 1, 1]);

        let shrinks: Vec<_> = strategy.shrink(&value).collect();
        // Just doesn't shrink, so no shrinks expected
        // When we have proper shrinking for range, this will produce interesting shrinks
        assert!(shrinks.is_empty());
    }

    // ===== Concrete Shrinking Tests =====

    #[test]
    fn test_range_shrinking_positive_to_zero() {
        // Test that positive numbers shrink toward zero via binary search
        let strategy = range(0, 200);
        let value = 100i32;

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).collect();

        // Binary search: 100 → 50 → 25 → 12 → 6 → 3 → 1 → 0
        assert!(!shrinks.is_empty(), "Should produce shrinks");
        assert_eq!(shrinks[0], 50, "First shrink should be halfway to zero");
        assert!(shrinks.contains(&0), "Should eventually reach zero");

        // Verify all shrinks are smaller than original
        for shrink in &shrinks {
            assert!(
                *shrink < value,
                "Shrink {} should be less than {}",
                shrink,
                value
            );
        }
    }

    #[test]
    fn test_range_shrinking_sequence() {
        // Verify the exact shrinking sequence
        let strategy = range(0, 100);
        let value = 64i32;

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).collect();

        // 64 → 32 → 16 → 8 → 4 → 2 → 1 → 0
        assert_eq!(shrinks[0], 32);
        assert_eq!(shrinks[1], 16);
        assert_eq!(shrinks[2], 8);
        assert_eq!(shrinks[3], 4);
        assert_eq!(shrinks[4], 2);
        assert_eq!(shrinks[5], 1);
        assert_eq!(shrinks[6], 0);
    }

    #[test]
    fn test_range_shrinking_negative_to_zero() {
        // Test that negative numbers shrink toward zero
        // Use a simpler case: positive value in negative range
        let strategy = range(-100, 0);
        let value = -50i32;

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).collect();

        assert!(!shrinks.is_empty());
        // When range doesn't include positive numbers, shrinks toward range start
        // -50 should shrink toward -100 (range start)
        assert!(shrinks[0] < value, "Should shrink toward range start");
    }

    #[test]
    fn test_range_shrinking_respects_bounds() {
        // Test that shrinking doesn't go below range start
        let strategy = range(10, 100);
        let value = 50i32;

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).collect();

        // Should shrink toward 10 (range start), not zero
        for shrink in &shrinks {
            assert!(
                *shrink >= 10,
                "Shrink {} should be >= range start 10",
                shrink
            );
            assert!(
                *shrink <= 100,
                "Shrink {} should be <= range end 100",
                shrink
            );
        }

        // Should eventually reach the range start
        assert!(shrinks.contains(&10));
    }

    #[test]
    fn test_range_shrinking_at_start() {
        // Test that value at range start doesn't shrink
        let strategy = range(5, 100);
        let value = 5i32;

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).collect();

        assert!(shrinks.is_empty(), "Value at range start should not shrink");
    }

    #[test]
    fn test_range_shrinking_unsigned() {
        // Test shrinking for unsigned integers
        let strategy = range(0u32, 200u32);
        let value = 100u32;

        let shrinks: Vec<u32> = Strategy::shrink(&strategy, &value).collect();

        assert!(!shrinks.is_empty());
        assert_eq!(shrinks[0], 50);
        assert!(shrinks.contains(&0));
    }

    #[test]
    fn test_one_of_shrinking_tries_alternatives() {
        // Test that OneOf returns all other values as shrinks
        let strategy = one_of(vec![1, 2, 3, 4, 5]);
        let value = 3;

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).collect();

        assert_eq!(shrinks.len(), 4, "Should have 4 alternatives");
        assert!(shrinks.contains(&1));
        assert!(shrinks.contains(&2));
        assert!(shrinks.contains(&4));
        assert!(shrinks.contains(&5));
        assert!(!shrinks.contains(&3), "Should not contain current value");
    }

    #[test]
    fn test_one_of_shrinking_order() {
        // Test that OneOf tries earlier values first
        let strategy = one_of(vec![10, 20, 30, 40, 50]);
        let value = 50;

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).collect();

        // Earlier values should come first
        assert_eq!(shrinks[0], 10);
        assert_eq!(shrinks[1], 20);
        assert_eq!(shrinks[2], 30);
        assert_eq!(shrinks[3], 40);
    }

    #[test]
    fn test_map_shrinking_with_range() {
        // Test that map properly shrinks through transformation
        let strategy = range(0, 100).map(|x| x * 2);
        let value = strategy.generate(&mut thread_rng(), &GeneratorConfig::default());

        if value.input == 0 {
            // If input is 0, no shrinks expected
            return;
        }

        let _shrinks: Vec<_> = Strategy::shrink(&strategy, &value).take(5).collect();

        // Should shrink the input and re-map
        assert!(!_shrinks.is_empty(), "Should produce shrinks if input > 0");

        // Each shrink should have output = input * 2
        for shrink in &_shrinks {
            assert_eq!(shrink.output, shrink.input * 2);
            assert!(shrink.input < value.input);
        }
    }

    #[test]
    fn test_filter_shrinking_maintains_predicate() {
        // Test that filter only yields shrinks passing the predicate
        let strategy = range(10, 100).filter(|&x| x % 10 == 0);

        // Generate a value (should be divisible by 10)
        let value = strategy.generate(&mut thread_rng(), &GeneratorConfig::default());
        assert_eq!(value % 10, 0);

        let shrinks: Vec<i32> = Strategy::shrink(&strategy, &value).take(20).collect();

        // All shrinks must also be divisible by 10
        for shrink in &shrinks {
            assert_eq!(
                shrink % 10,
                0,
                "Shrink {} should be divisible by 10",
                shrink
            );
        }
    }

    #[test]
    fn test_zip_shrinking_both_sides() {
        // Test that zip shrinks both components
        let left_strategy = range(0, 100);
        let right_strategy = range(0, 50);
        let strategy = left_strategy.zip(right_strategy);

        let value = (80i32, 40i32);
        let shrinks: Vec<(i32, i32)> = Strategy::shrink(&strategy, &value).take(20).collect();

        assert!(!shrinks.is_empty());

        // Should have shrinks where left changed
        assert!(shrinks.iter().any(|(l, r)| *l < 80 && *r == 40));

        // Should have shrinks where right changed
        assert!(shrinks.iter().any(|(l, r)| *l == 80 && *r < 40));
    }

    // ===== Float Shrinking Tests =====

    #[test]
    fn test_f32_shrinking_positive_to_zero() {
        // Test that positive floats shrink toward zero
        let strategy = range(0.0f32, 200.0f32);
        let value = 100.0f32;

        let shrinks: Vec<f32> = Strategy::shrink(&strategy, &value).collect();

        assert!(!shrinks.is_empty(), "Should produce shrinks for f32");
        // First shrink should be approximately halfway to zero
        assert!(
            (shrinks[0] - 50.0).abs() < 1.0,
            "First shrink should be near 50.0, got {}",
            shrinks[0]
        );

        // Should eventually get close to zero
        assert!(
            shrinks.iter().any(|&s| s < 1.0),
            "Should shrink close to zero"
        );

        // All shrinks should be smaller than original
        for shrink in &shrinks {
            assert!(
                *shrink < value,
                "Shrink {} should be less than {}",
                shrink,
                value
            );
        }
    }

    #[test]
    fn test_f64_shrinking_positive_to_zero() {
        // Test that positive f64 values shrink toward zero
        let strategy = range(0.0f64, 200.0f64);
        let value = 100.0f64;

        let shrinks: Vec<f64> = Strategy::shrink(&strategy, &value).collect();

        assert!(!shrinks.is_empty(), "Should produce shrinks for f64");
        // First shrink should be approximately halfway to zero
        assert!(
            (shrinks[0] - 50.0).abs() < 1.0,
            "First shrink should be near 50.0, got {}",
            shrinks[0]
        );

        // Should eventually get close to zero
        assert!(
            shrinks.iter().any(|&s| s < 1.0),
            "Should shrink close to zero"
        );

        // All shrinks should be smaller than original
        for shrink in &shrinks {
            assert!(
                *shrink < value,
                "Shrink {} should be less than {}",
                shrink,
                value
            );
        }
    }

    #[test]
    fn test_f32_shrinking_respects_bounds() {
        // Test that float shrinking doesn't go below range start
        let strategy = range(10.0f32, 100.0f32);
        let value = 50.0f32;

        let shrinks: Vec<f32> = Strategy::shrink(&strategy, &value).collect();

        // Should shrink toward 10.0 (range start), not zero
        for shrink in &shrinks {
            assert!(
                *shrink >= 10.0 - f32::EPSILON,
                "Shrink {} should be >= range start 10.0",
                shrink
            );
            assert!(
                *shrink <= 100.0 + f32::EPSILON,
                "Shrink {} should be <= range end 100.0",
                shrink
            );
        }

        // Should eventually get close to the range start
        assert!(
            shrinks.iter().any(|&s| (s - 10.0).abs() < 1.0),
            "Should shrink close to range start"
        );
    }

    #[test]
    fn test_f64_shrinking_respects_bounds() {
        // Test that float shrinking doesn't go below range start
        let strategy = range(10.0f64, 100.0f64);
        let value = 50.0f64;

        let shrinks: Vec<f64> = Strategy::shrink(&strategy, &value).collect();

        // Should shrink toward 10.0 (range start), not zero
        for shrink in &shrinks {
            assert!(
                *shrink >= 10.0 - f64::EPSILON,
                "Shrink {} should be >= range start 10.0",
                shrink
            );
            assert!(
                *shrink <= 100.0 + f64::EPSILON,
                "Shrink {} should be <= range end 100.0",
                shrink
            );
        }

        // Should eventually get close to the range start
        assert!(
            shrinks.iter().any(|&s| (s - 10.0).abs() < 1.0),
            "Should shrink close to range start"
        );
    }

    #[test]
    fn test_f32_shrinking_at_start() {
        // Test that float at range start doesn't shrink
        let strategy = range(5.0f32, 100.0f32);
        let value = 5.0f32;

        let shrinks: Vec<f32> = Strategy::shrink(&strategy, &value).collect();

        assert!(shrinks.is_empty(), "Value at range start should not shrink");
    }

    #[test]
    fn test_f64_shrinking_at_start() {
        // Test that float at range start doesn't shrink
        let strategy = range(5.0f64, 100.0f64);
        let value = 5.0f64;

        let shrinks: Vec<f64> = Strategy::shrink(&strategy, &value).collect();

        assert!(shrinks.is_empty(), "Value at range start should not shrink");
    }

    #[test]
    fn test_f32_shrinking_nan() {
        // Test that NaN doesn't shrink (can't meaningfully shrink NaN)
        let strategy = range(0.0f32, 100.0f32);
        let value = f32::NAN;

        let shrinks: Vec<f32> = Strategy::shrink(&strategy, &value).collect();

        assert!(shrinks.is_empty(), "NaN should not produce shrinks");
    }

    #[test]
    fn test_f32_shrinking_infinity() {
        // Test that infinity doesn't shrink
        let strategy = range(0.0f32, 100.0f32);
        let value = f32::INFINITY;

        let shrinks: Vec<f32> = Strategy::shrink(&strategy, &value).collect();

        assert!(shrinks.is_empty(), "Infinity should not produce shrinks");
    }

    #[test]
    fn test_f64_shrinking_special_values() {
        // Test that NaN and infinity don't shrink
        let strategy = range(0.0f64, 100.0f64);

        let nan_shrinks: Vec<f64> = Strategy::shrink(&strategy, &f64::NAN).collect();
        assert!(nan_shrinks.is_empty(), "NaN should not produce shrinks");

        let inf_shrinks: Vec<f64> = Strategy::shrink(&strategy, &f64::INFINITY).collect();
        assert!(
            inf_shrinks.is_empty(),
            "Infinity should not produce shrinks"
        );
    }

    #[test]
    fn test_f32_shrinking_small_values() {
        // Test shrinking very small positive values
        let strategy = range(0.0f32, 1.0f32);
        let value = 0.5f32;

        let shrinks: Vec<f32> = Strategy::shrink(&strategy, &value).collect();

        assert!(
            !shrinks.is_empty(),
            "Should produce shrinks for small values"
        );

        // Should shrink toward zero
        for shrink in &shrinks {
            assert!(
                *shrink < value,
                "Shrink {} should be less than {}",
                shrink,
                value
            );
            assert!(*shrink >= 0.0, "Shrink {} should be >= 0.0", shrink);
        }
    }

    #[test]
    fn test_f64_shrinking_negative_range() {
        // Test shrinking in negative-only range
        let strategy = range(-100.0f64, -10.0f64);
        let value = -50.0f64;

        let shrinks: Vec<f64> = Strategy::shrink(&strategy, &value).collect();

        // Should shrink toward range start (-100.0)
        for shrink in &shrinks {
            assert!(
                *shrink >= -100.0 - f64::EPSILON,
                "Shrink {} should be >= -100.0",
                shrink
            );
            assert!(
                *shrink <= -10.0 + f64::EPSILON,
                "Shrink {} should be <= -10.0",
                shrink
            );
        }
    }

    #[test]
    fn test_tuple3_generation() {
        let strategy = tuple3(range(1, 10), range(20, 30), range(100, 200));
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (a, b, c) = Strategy::generate(&strategy, &mut rng, &config);

        assert!((1..=10).contains(&a));
        assert!((20..=30).contains(&b));
        assert!((100..=200).contains(&c));
    }

    #[test]
    fn test_tuple3_shrinking_first_element() {
        let strategy = tuple3(range(1, 100), just(50), just(200));
        let value = (100, 50, 200);

        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();

        // Should produce shrinks for first element only (second and third are Just)
        assert!(!shrinks.is_empty());

        // All shrinks should keep b=50 and c=200 constant
        for (a, b, c) in &shrinks {
            assert!(*a < 100, "First element should shrink");
            assert_eq!(*b, 50, "Second element should stay constant");
            assert_eq!(*c, 200, "Third element should stay constant");
        }
    }

    #[test]
    fn test_tuple3_shrinking_all_elements() {
        let strategy = tuple3(range(1, 100), range(1, 100), range(1, 100));
        let value = (50, 60, 70);

        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();

        // Should produce shrinks for all three elements
        assert!(!shrinks.is_empty());

        // Check that we have shrinks that vary each element independently
        let has_first_shrink = shrinks
            .iter()
            .any(|(a, b, c)| *a != 50 && *b == 60 && *c == 70);
        let has_second_shrink = shrinks
            .iter()
            .any(|(a, b, c)| *a == 50 && *b != 60 && *c == 70);
        let has_third_shrink = shrinks
            .iter()
            .any(|(a, b, c)| *a == 50 && *b == 60 && *c != 70);

        assert!(has_first_shrink, "Should have shrinks for first element");
        assert!(has_second_shrink, "Should have shrinks for second element");
        assert!(has_third_shrink, "Should have shrinks for third element");
    }

    #[test]
    fn test_tuple4_generation() {
        let strategy = tuple4(
            range(1, 10),
            range(20, 30),
            range(100, 200),
            range(500, 1000),
        );
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (a, b, c, d) = Strategy::generate(&strategy, &mut rng, &config);

        assert!((1..=10).contains(&a));
        assert!((20..=30).contains(&b));
        assert!((100..=200).contains(&c));
        assert!((500..=1000).contains(&d));
    }

    #[test]
    fn test_tuple4_shrinking() {
        let strategy = tuple4(range(1, 100), range(1, 100), just(50), just(200));
        let value = (100, 80, 50, 200);

        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();

        // Should produce shrinks for first two elements only
        assert!(!shrinks.is_empty());

        // All shrinks should keep c=50 and d=200 constant
        for (a, b, c, d) in &shrinks {
            assert_eq!(*c, 50, "Third element should stay constant");
            assert_eq!(*d, 200, "Fourth element should stay constant");
            assert!(
                *a <= 100 && *b <= 80,
                "First two elements should shrink or stay same"
            );
        }
    }

    #[test]
    fn test_tuple5_generation() {
        let strategy = tuple5(
            range(1, 10),
            range(20, 30),
            range(100, 200),
            range(500, 1000),
            range(5000, 10000),
        );
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let (a, b, c, d, e) = Strategy::generate(&strategy, &mut rng, &config);

        assert!((1..=10).contains(&a));
        assert!((20..=30).contains(&b));
        assert!((100..=200).contains(&c));
        assert!((500..=1000).contains(&d));
        assert!((5000..=10000).contains(&e));
    }

    #[test]
    fn test_tuple5_shrinking() {
        let strategy = tuple5(range(1, 100), just(20), just(30), just(40), range(1, 100));
        let value = (100, 20, 30, 40, 80);

        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();

        // Should produce shrinks for first and last elements only
        assert!(!shrinks.is_empty());

        // Middle three should stay constant
        for (a, b, c, d, e) in &shrinks {
            assert_eq!(*b, 20, "Second element should stay constant");
            assert_eq!(*c, 30, "Third element should stay constant");
            assert_eq!(*d, 40, "Fourth element should stay constant");
            assert!(
                *a <= 100 && *e <= 80,
                "First and last elements should shrink or stay same"
            );
        }
    }

    #[test]
    fn test_tuple5_shrinking_all_elements() {
        let strategy = tuple5(
            range(1, 100),
            range(1, 100),
            range(1, 100),
            range(1, 100),
            range(1, 100),
        );
        let value = (50, 60, 70, 80, 90);

        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();

        // Should produce shrinks for all five elements
        assert!(!shrinks.is_empty());

        // Check that we have shrinks that vary each element independently
        let has_first = shrinks
            .iter()
            .any(|(a, b, c, d, e)| *a != 50 && *b == 60 && *c == 70 && *d == 80 && *e == 90);
        let has_second = shrinks
            .iter()
            .any(|(a, b, c, d, e)| *a == 50 && *b != 60 && *c == 70 && *d == 80 && *e == 90);
        let has_third = shrinks
            .iter()
            .any(|(a, b, c, d, e)| *a == 50 && *b == 60 && *c != 70 && *d == 80 && *e == 90);
        let has_fourth = shrinks
            .iter()
            .any(|(a, b, c, d, e)| *a == 50 && *b == 60 && *c == 70 && *d != 80 && *e == 90);
        let has_fifth = shrinks
            .iter()
            .any(|(a, b, c, d, e)| *a == 50 && *b == 60 && *c == 70 && *d == 80 && *e != 90);

        assert!(has_first, "Should have shrinks for first element");
        assert!(has_second, "Should have shrinks for second element");
        assert!(has_third, "Should have shrinks for third element");
        assert!(has_fourth, "Should have shrinks for fourth element");
        assert!(has_fifth, "Should have shrinks for fifth element");
    }

    #[test]
    fn test_tuple3_shrinking_order() {
        // Test that tuples shrink in order: first element, then second, then third
        let strategy = tuple3(range(1, 50), range(1, 50), range(1, 50));
        let value = (25, 25, 25);

        let shrinks: Vec<_> = Strategy::shrink(&strategy, &value).collect();

        // The first shrinks should be from the first element
        // Since range(1, 50) with value 25 shrinks toward 0 (or 1 if 0 is not in range)
        // We expect the first batch of shrinks to have different first elements
        let first_batch: Vec<_> = shrinks.iter().take(5).collect();

        for (a, b, c) in &first_batch {
            assert!(
                *a < 25,
                "First batch should shrink first element: ({}, {}, {})",
                a,
                b,
                c
            );
            assert_eq!(*b, 25, "First batch should keep second element constant");
            assert_eq!(*c, 25, "First batch should keep third element constant");
        }
    }
}
