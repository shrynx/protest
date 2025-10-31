//! Derive macros for the Protest property testing library
//!
//! This crate provides procedural macros for automatically implementing traits
//! in the Protest library.

use proc_macro::TokenStream;

mod derive;
mod property_test;

/// Derive macro for automatically implementing the Generator trait
///
/// This macro can be applied to structs and enums to automatically generate
/// implementations of the Generator trait.
///
/// # Basic Usage
///
/// ```rust
/// use protest::Generator;
///
/// #[derive(Generator)]
/// struct User {
///     id: u32,
///     name: String,
///     active: bool,
/// }
/// ```
///
/// # Customization
///
/// The derive macro supports various customization attributes:
///
/// ```rust
/// use protest::Generator;
///
/// #[derive(Generator)]
/// struct CustomUser {
///     #[generator(range = "1..1000")]
///     id: u32,
///     #[generator(length = "5..20")]
///     name: String,
///     #[generator(custom = "always_true")]
///     active: bool,
/// }
///
/// fn always_true() -> bool {
///     true
/// }
/// ```
///
/// # Supported Attributes
///
/// - `range = "min..max"`: For numeric types, specifies the range of generated values
/// - `length = "min..max"`: For collections and strings, specifies the length range
/// - `custom = "function_name"`: Uses a custom function to generate the field value
///
/// # Generic Types
///
/// The derive macro supports generic types with appropriate bounds:
///
/// ```rust
/// use protest::Generator;
///
/// #[derive(Generator)]
/// struct GenericStruct<T, U> {
///     first: T,
///     second: U,
/// }
/// ```
///
/// # Enums
///
/// The derive macro supports enums with all variant types:
///
/// ```rust
/// use protest::Generator;
///
/// #[derive(Generator)]
/// enum Status {
///     Active,
///     Inactive(String),
///     Pending { reason: String },
/// }
/// ```
#[proc_macro_derive(Generator, attributes(generator))]
pub fn derive_generator(input: TokenStream) -> TokenStream {
    derive::derive_generator_impl(input)
}

/// Attribute macro for creating property-based tests
///
/// This macro transforms a regular test function into a property-based test by automatically
/// generating test data and integrating with the Protest testing framework.
///
/// # Basic Usage
///
/// ```rust
/// use protest::property_test;
///
/// #[property_test]
/// fn test_addition_commutative(a: i32, b: i32) {
///     assert_eq!(a + b, b + a);
/// }
/// ```
///
/// # Configuration Attributes
///
/// The macro supports various configuration attributes:
///
/// ```rust
/// use protest::property_test;
///
/// #[property_test(iterations = 1000, seed = 42)]
/// fn test_with_config(x: u32) {
///     assert!(x >= 0);
/// }
/// ```
///
/// # Supported Attributes
///
/// - `iterations = N`: Number of test iterations (default: 100)
/// - `seed = N`: Random seed for reproducible tests
/// - `max_shrink_iterations = N`: Maximum shrinking attempts (default: 1000)
/// - `shrink_timeout_secs = N`: Shrinking timeout in seconds (default: 10)
///
/// # Async Support
///
/// The macro automatically detects async functions and uses async property testing:
///
/// ```rust
/// use protest::property_test;
///
/// #[property_test]
/// async fn test_async_operation(data: String) {
///     let result = some_async_operation(&data).await;
///     assert!(!result.is_empty());
/// }
/// ```
///
/// # Generator Inference
///
/// The macro automatically infers generators for function parameters based on their types.
/// For custom types, ensure they implement the `Generator` trait or derive it.
#[proc_macro_attribute]
pub fn property_test(args: TokenStream, input: TokenStream) -> TokenStream {
    property_test::property_test_impl(args, input)
}

/// Macro for creating fluent property test builders
///
/// This macro provides shortcuts for common property test patterns and configurations.
///
/// # Basic Usage
///
/// ```rust
/// use protest::test_builder;
///
/// test_builder! {
///     iterations: 1000,
///     seed: 42,
///     test_name: my_property_test,
///     generator: range(1, 100),
///     property: |x: i32| x > 0
/// }
/// ```
///
/// # Supported Options
///
/// - `iterations`: Number of test iterations
/// - `seed`: Random seed for reproducible tests
/// - `max_shrink_iterations`: Maximum shrinking attempts
/// - `shrink_timeout_secs`: Shrinking timeout in seconds
/// - `test_name`: Name of the generated test function
/// - `generator`: Generator expression
/// - `property`: Property closure or function
#[proc_macro]
pub fn test_builder(input: TokenStream) -> TokenStream {
    property_test::test_builder_impl(input)
}
