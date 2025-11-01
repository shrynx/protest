//! # Protest Stateful
//!
//! Stateful property testing for Rust - test state machines, APIs, databases,
//! concurrent systems, and any system that maintains state across operations.
//!
//! ## Features
//!
//! - **State Machine Testing**: Define operations and invariants, automatically test sequences
//! - **Model-Based Testing**: Compare real system behavior against a reference model
//! - **Concurrent Testing**: Test parallel operations with linearizability checking
//! - **Advanced Sequence Shrinking**: Delta debugging and smart shrinking for minimal counterexamples
//! - **Preconditions & Postconditions**: Define valid operation contexts
//! - **Temporal Properties**: Express "eventually" and "always" properties
//!
//! ## Quick Example
//!
//! ```rust
//! use protest_stateful::prelude::*;
//!
//! // Define your state
//! #[derive(Clone, Debug)]
//! struct Stack {
//!     items: Vec<i32>,
//! }
//!
//! // Define your operations
//! #[derive(Debug, Clone)]
//! enum StackOp {
//!     Push(i32),
//!     Pop,
//! }
//!
//! impl Operation for StackOp {
//!     type State = Stack;
//!
//!     fn execute(&self, state: &mut Self::State) {
//!         match self {
//!             StackOp::Push(val) => state.items.push(*val),
//!             StackOp::Pop => { state.items.pop(); }
//!         }
//!     }
//!
//!     fn precondition(&self, state: &Self::State) -> bool {
//!         match self {
//!             StackOp::Pop => !state.items.is_empty(),
//!             _ => true,
//!         }
//!     }
//! }
//!
//! # fn main() {
//! // Create a stateful test
//! let test = StatefulTest::new(Stack { items: vec![] })
//!     .invariant("length_non_negative", |state: &Stack| state.items.len() >= 0);
//!
//! // Create an operation sequence
//! let mut seq = OperationSequence::new();
//! seq.push(StackOp::Push(5));
//! seq.push(StackOp::Push(10));
//! seq.push(StackOp::Pop);
//!
//! // Run the test
//! let result = test.run(&seq);
//! assert!(result.is_ok());
//! # }
//! ```

// Re-export derive macros
pub use protest_stateful_derive::{Operation, stateful_test};

pub mod concurrent;
pub mod dsl;
pub mod invariants;
pub mod model;
pub mod operations;
pub mod temporal;

/// Re-exports for convenient imports
pub mod prelude {
    pub use crate::dsl::*;
    pub use crate::invariants::*;
    pub use crate::model::*;
    pub use crate::operations::*;
}
