//! Ergonomic APIs for property-based testing
//!
//! This module provides convenient, low-boilerplate APIs for writing property tests.
//! It includes:
//! - Closure-based property definitions
//! - Automatic generator inference
//! - Fluent builder APIs
//! - Common property patterns
//!
//! # Examples
//!
//! ```rust
//! use protest::ergonomic::*;
//!
//! // Using closures directly as properties
//! let result = check_with_closure(
//!     protest::range(1, 100),
//!     |x: i32| x > 0
//! );
//! ```

pub mod auto_gen;
pub mod builder;
pub mod closure_property;
pub mod macros;
pub mod patterns;

// Re-export main types
pub use auto_gen::{AutoGen, InferredGenerator};
pub use builder::{ErgonomicPropertyTest, ErgonomicPropertyTestWithClosure, property};
pub use closure_property::{
    ClosureProperty, PropertyClosure, check_with_closure, check_with_closure_config,
};
pub use patterns::*;

// Macros are exported at crate root via #[macro_export]
