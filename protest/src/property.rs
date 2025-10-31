//! Property definition traits for synchronous and asynchronous testing.

use crate::error::PropertyError;

/// Property definition trait for synchronous testing
pub trait Property<T> {
    type Output;

    /// Test the property with the given input
    fn test(&self, input: T) -> Result<Self::Output, PropertyError>;
}

/// Async property trait for asynchronous testing
pub trait AsyncProperty<T> {
    type Output;

    /// Test the property asynchronously with the given input
    fn test(
        &self,
        input: T,
    ) -> impl std::future::Future<Output = Result<Self::Output, PropertyError>> + Send;
}
