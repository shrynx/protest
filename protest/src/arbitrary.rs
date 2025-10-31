//! Arbitrary trait and related functionality for automatic test data generation.

use crate::strategy::Strategy;

/// Trait for types that can generate arbitrary instances of themselves
///
/// This is the primary trait that types implement to participate in property-based testing.
/// It provides a default strategy for generating random instances of the type.
pub trait Arbitrary: Sized {
    /// The strategy type used to generate values of this type
    type Strategy: Strategy<Value = Self>;

    /// Create a strategy for generating arbitrary values of this type
    fn arbitrary() -> Self::Strategy;

    /// Create a strategy with custom parameters
    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy;

    /// Parameters that can be used to customize generation
    type Parameters: Default + Clone;
}
