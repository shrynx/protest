//! Common property testing patterns.
//!
//! This module provides convenient functions for testing common mathematical
//! and structural properties like commutativity, associativity, and round-trip encoding.

use crate::ergonomic::closure_property::ClosureProperty;

/// Test that a binary operation is commutative: `f(a, b) == f(b, a)`
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::commutative;
/// use protest::{check, range};
///
/// // Test that addition is commutative
/// let property = commutative(|a: i32, b: i32| a.wrapping_add(b));
/// let result = check(
///     (range(-100, 100), range(-100, 100)),
///     property
/// );
/// assert!(result.is_ok());
/// ```
pub fn commutative<T, R, F>(op: F) -> ClosureProperty<impl Fn((T, T)) -> bool>
where
    F: Fn(T, T) -> R + Clone + 'static,
    T: Clone + 'static,
    R: PartialEq + 'static,
{
    ClosureProperty::new(move |(a, b): (T, T)| {
        let ab = op(a.clone(), b.clone());
        let ba = op(b, a);
        ab == ba
    })
}

/// Test that a binary operation is associative: `f(f(a, b), c) == f(a, f(b, c))`
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::associative;
/// use protest::{check, range};
///
/// // Test that addition is associative
/// let property = associative(|a: i32, b: i32| a.wrapping_add(b));
/// let result = check(
///     (range(-100, 100), range(-100, 100), range(-100, 100)),
///     property
/// );
/// assert!(result.is_ok());
/// ```
pub fn associative<T, F>(op: F) -> ClosureProperty<impl Fn((T, T, T)) -> bool>
where
    F: Fn(T, T) -> T + Clone + 'static,
    T: PartialEq + Clone + 'static,
{
    ClosureProperty::new(move |(a, b, c): (T, T, T)| {
        let ab_c = op(op(a.clone(), b.clone()), c.clone());
        let a_bc = op(a, op(b, c));
        ab_c == a_bc
    })
}

/// Test that a function is idempotent: `f(f(x)) == f(x)`
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::idempotent;
/// use protest::{check, range};
///
/// // Test that abs is idempotent
/// let property = idempotent(|x: i32| x.abs());
/// let result = check(range(-100, 100), property);
/// assert!(result.is_ok());
/// ```
pub fn idempotent<T, F>(f: F) -> ClosureProperty<impl Fn(T) -> bool>
where
    F: Fn(T) -> T + Clone + 'static,
    T: PartialEq + Clone + 'static,
{
    ClosureProperty::new(move |x: T| {
        let fx = f(x.clone());
        let ffx = f(fx.clone());
        fx == ffx
    })
}

/// Test a round-trip property: `decode(encode(x)) == x`
///
/// This is useful for testing serialization/deserialization, encoding/decoding,
/// or any pair of inverse operations.
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::round_trip;
/// use protest::{check, range};
///
/// // Test that converting to string and parsing back works
/// let property = round_trip(
///     |x: i32| x.to_string(),
///     |s: String| s.parse::<i32>().unwrap()
/// );
/// let result = check(range(-100, 100), property);
/// assert!(result.is_ok());
/// ```
pub fn round_trip<T, U, E, D>(encode: E, decode: D) -> ClosureProperty<impl Fn(T) -> bool>
where
    E: Fn(T) -> U + Clone + 'static,
    D: Fn(U) -> T + Clone + 'static,
    T: PartialEq + Clone + 'static,
    U: Clone + 'static,
{
    ClosureProperty::new(move |input: T| {
        let encoded = encode(input.clone());
        let decoded = decode(encoded);
        decoded == input
    })
}

/// Test that two functions are inverses: `f(g(x)) == x && g(f(x)) == x`
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::inverse;
/// use protest::{check, range};
///
/// // Test that adding and subtracting are inverses
/// let property = inverse(
///     |x: i32| x.wrapping_add(10),
///     |x: i32| x.wrapping_sub(10)
/// );
/// let result = check(range(0, 100), property);
/// assert!(result.is_ok());
/// ```
pub fn inverse<T, F, G>(f: F, g: G) -> ClosureProperty<impl Fn(T) -> bool>
where
    F: Fn(T) -> T + Clone + 'static,
    G: Fn(T) -> T + Clone + 'static,
    T: PartialEq + Clone + 'static,
{
    ClosureProperty::new(move |x: T| {
        let fgx = f(g(x.clone()));
        let gfx = g(f(x.clone()));
        fgx == x && gfx == x
    })
}

/// Test that a function is monotonic (always increasing or always decreasing)
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::monotonic_increasing;
/// use protest::{check, range};
///
/// // Test that x^2 is monotonic for positive numbers
/// let property = monotonic_increasing(|x: i32| x * x);
/// let result = check((range(0, 100), range(0, 100)), property);
/// assert!(result.is_ok());
/// ```
pub fn monotonic_increasing<T, R, F>(f: F) -> ClosureProperty<impl Fn((T, T)) -> bool>
where
    F: Fn(T) -> R + Clone + 'static,
    T: PartialOrd + Clone + 'static,
    R: PartialOrd + 'static,
{
    ClosureProperty::new(move |(a, b): (T, T)| {
        if a <= b {
            f(a.clone()) <= f(b.clone())
        } else {
            true // Skip when inputs are not in order
        }
    })
}

/// Test that a function is monotonic decreasing
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::monotonic_decreasing;
/// use protest::{check, range};
///
/// // Test that negation is monotonic decreasing
/// let property = monotonic_decreasing(|x: i32| -x);
/// let result = check((range(-100, 100), range(-100, 100)), property);
/// assert!(result.is_ok());
/// ```
pub fn monotonic_decreasing<T, R, F>(f: F) -> ClosureProperty<impl Fn((T, T)) -> bool>
where
    F: Fn(T) -> R + Clone + 'static,
    T: PartialOrd + Clone + 'static,
    R: PartialOrd + 'static,
{
    ClosureProperty::new(move |(a, b): (T, T)| {
        if a <= b {
            f(a.clone()) >= f(b.clone())
        } else {
            true // Skip when inputs are not in order
        }
    })
}

/// Test that an operation has an identity element: `f(x, identity) == x`
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::has_identity;
/// use protest::{check, range};
///
/// // Test that 0 is the identity for addition
/// let property = has_identity(|a: i32, b: i32| a + b, 0);
/// let result = check(range(-100, 100), property);
/// assert!(result.is_ok());
/// ```
pub fn has_identity<T, F>(op: F, identity: T) -> ClosureProperty<impl Fn(T) -> bool>
where
    F: Fn(T, T) -> T + Clone + 'static,
    T: PartialEq + Clone + 'static,
{
    ClosureProperty::new(move |x: T| {
        let result_left = op(x.clone(), identity.clone());
        let result_right = op(identity.clone(), x.clone());
        result_left == x && result_right == x
    })
}

/// Test the distributive property: `f(a, g(b, c)) == g(f(a, b), f(a, c))`
///
/// # Examples
///
/// ```rust
/// use protest::ergonomic::patterns::distributive;
/// use protest::{check, range};
///
/// // Test that multiplication distributes over addition
/// let property = distributive(
///     |a: i32, b: i32| a.wrapping_mul(b),
///     |a: i32, b: i32| a.wrapping_add(b)
/// );
/// let result = check(
///     (range(-10, 10), range(-10, 10), range(-10, 10)),
///     property
/// );
/// assert!(result.is_ok());
/// ```
pub fn distributive<T, F, G>(f: F, g: G) -> ClosureProperty<impl Fn((T, T, T)) -> bool>
where
    F: Fn(T, T) -> T + Clone + 'static,
    G: Fn(T, T) -> T + Clone + 'static,
    T: PartialEq + Clone + 'static,
{
    ClosureProperty::new(move |(a, b, c): (T, T, T)| {
        let left = f(a.clone(), g(b.clone(), c.clone()));
        let right = g(f(a.clone(), b), f(a, c));
        left == right
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::check;

    use crate::primitives::{IntGenerator, TupleStrategy2, TupleStrategy3};

    #[test]
    fn test_commutative_addition() {
        let property = commutative(|a: i32, b: i32| a.wrapping_add(b));
        let _gen1 = IntGenerator::new(-100, 100);
        let _gen2 = IntGenerator::new(-100, 100);
        // We need a tuple generator - using TupleStrategy2 which implements Generator
        let tuple_gen = TupleStrategy2::<i32, i32>::new();
        let result = check(tuple_gen, property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_associative_addition() {
        let property = associative(|a: i32, b: i32| a.wrapping_add(b));
        let tuple_gen = TupleStrategy3::<i32, i32, i32>::new();
        let result = check(tuple_gen, property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_idempotent_abs() {
        let property = idempotent(|x: i32| x.abs());
        let result = check(IntGenerator::new(-100, 100), property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_round_trip_string_conversion() {
        let property = round_trip(
            |x: i32| x.to_string(),
            |s: String| s.parse::<i32>().unwrap(),
        );
        let result = check(IntGenerator::new(-100, 100), property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_identity_addition() {
        let property = has_identity(|a: i32, b: i32| a.wrapping_add(b), 0);
        let result = check(IntGenerator::new(-100, 100), property);
        assert!(result.is_ok());
    }
}
