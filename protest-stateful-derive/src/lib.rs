//! Derive macros for protest-stateful
//!
//! This crate provides procedural macros for the protest-stateful stateful property testing library.

use proc_macro::TokenStream;

mod operation;
mod stateful_test;

/// Derive macro for automatically implementing the Operation trait
///
/// This macro can be applied to enums to automatically generate implementations
/// of the Operation trait for stateful testing.
///
/// # Basic Usage
///
/// ```
/// use protest_stateful::{Operation, operations::OperationSequence, dsl::StatefulTest};
///
/// #[derive(Debug, Clone, Operation)]
/// #[operation(state = "Vec<i32>")]
/// enum StackOp {
///     #[execute("state.push(*field_0)")]
///     Push(i32),
///     #[execute("state.pop()")]
///     Pop,
///     #[execute("state.clear()")]
///     Clear,
/// }
/// ```
///
/// By default, the state type is inferred from the `#[operation(state = "...")]` attribute,
/// and each variant executes based on its structure.
///
/// # Customization
///
/// The derive macro supports various customization attributes:
///
/// ```
/// use protest_stateful::Operation;
///
/// #[derive(Debug, Clone, Operation)]
/// #[operation(state = "Vec<i32>")]
/// enum StackOp {
///     #[execute("state.push(*field_0)")]
///     Push(i32),
///
///     #[execute("state.pop()")]
///     #[precondition("!state.is_empty()")]
///     Pop,
///
///     #[execute("state.clear()")]
///     Clear,
/// }
/// ```
///
/// # Supported Attributes
///
/// ## Container-level attributes:
/// - `#[operation(state = "Type")]`: Specifies the state type (required)
///
/// ## Variant-level attributes:
/// - `#[execute("expression")]`: Custom execution expression
/// - `#[precondition("expression")]`: Precondition check expression
/// - `#[weight(N)]`: Weight for generation (higher = more frequent)
/// - `#[description("text")]`: Custom description for the operation
///
/// # Weight-based Generation
///
/// Use the `#[weight(N)]` attribute to control operation frequency:
///
/// ```
/// use protest_stateful::Operation;
///
/// #[derive(Debug, Clone, Operation)]
/// #[operation(state = "Vec<i32>")]
/// enum StackOp {
///     #[execute("state.push(*field_0)")]
///     #[weight(10)]  // Generated 10x more often
///     Push(i32),
///
///     #[execute("state.pop()")]
///     #[weight(5)]   // Generated 5x more often
///     Pop,
///
///     #[execute("state.clear()")]
///     #[weight(1)]   // Generated least often
///     Clear,
/// }
/// ```
///
/// # Field Access
///
/// In `#[execute]` and `#[precondition]` expressions:
/// - Named fields: access by name (e.g., `self.value`)
/// - Unnamed fields: access by position (e.g., `self.0`, `self.1`)
/// - Unit variants: no field access needed
///
/// # Example with All Features
///
/// ```
/// use protest_stateful::Operation;
/// use std::collections::HashMap;
///
/// #[derive(Debug, Clone, Operation)]
/// #[operation(state = "HashMap<String, i32>")]
/// enum MapOp {
///     #[execute("state.insert(key.clone(), *value)")]
///     #[weight(5)]
///     #[description("Insert key-value pair")]
///     Insert { key: String, value: i32 },
///
///     #[execute("state.remove(key)")]
///     #[precondition("state.contains_key(key)")]
///     #[weight(3)]
///     Remove { key: String },
///
///     #[execute("state.clear()")]
///     #[weight(1)]
///     Clear,
/// }
/// ```
#[proc_macro_derive(
    Operation,
    attributes(operation, execute, precondition, weight, description)
)]
pub fn derive_operation(input: TokenStream) -> TokenStream {
    operation::derive_operation_impl(input)
}

/// Declarative macro for creating stateful property tests
///
/// This macro provides a convenient DSL for defining stateful property tests
/// with less boilerplate than manually constructing StatefulTest instances.
///
/// # Basic Usage
///
/// ```
/// # /*
/// use protest_stateful::stateful_test;
///
/// stateful_test! {
///     name: counter_test,
///     state: i32 = 0,
///     operations: CounterOp,
///     invariants: {
///         "non_negative" => |state| *state >= 0,
///         "bounded" => |state| *state <= 1000,
///     },
///     config: {
///         iterations: 100,
///         max_sequence_length: 20,
///     }
/// }
/// # */
/// ```
///
/// # Configuration Options
///
/// - `name`: Test function name (required)
/// - `state`: Initial state value and type (required)
/// - `operations`: Operation enum type (required)
/// - `invariants`: Named invariant checks (optional)
/// - `config`: Test configuration (optional)
///   - `iterations`: Number of test iterations (default: 100)
///   - `max_sequence_length`: Maximum operation sequence length (default: 10)
///   - `min_sequence_length`: Minimum operation sequence length (default: 1)
///   - `seed`: Random seed for reproducibility (optional)
///
/// # Complete Example
///
/// ```
/// # /*
/// use protest_stateful::{stateful_test, Operation};
///
/// #[derive(Debug, Clone, Operation)]
/// #[operation(state = "Vec<i32>")]
/// enum StackOp {
///     #[execute("state.push(*field_0)")]
///     Push(i32),
///     #[execute("state.pop()")]
///     Pop,
/// }
///
/// stateful_test! {
///     name: stack_properties,
///     state: Vec<i32> = vec![],
///     operations: StackOp,
///     invariants: {
///         "size_matches" => |state| state.len() <= 100,
///     },
///     config: {
///         iterations: 200,
///         max_sequence_length: 50,
///         seed: 42,
///     }
/// }
/// # */
/// ```
#[proc_macro]
pub fn stateful_test(input: TokenStream) -> TokenStream {
    stateful_test::stateful_test_impl(input)
}
