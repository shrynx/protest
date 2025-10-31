//! Extra generators for common data types and patterns
//!
//! This module provides generators for:
//! - Network types (IP addresses, URLs, emails)
//! - DateTime types (timestamps, durations, system time)
//! - Text types (alphabetic, identifiers, sentences)
//! - Collection types (non-empty, sorted, unique)
//! - Numeric types (positive, even, prime, percentages)
//! - Domain types (UUIDs, base64, hex, paths)

pub mod collections;
pub mod datetime;
pub mod domain;
pub mod network;
pub mod numeric;
pub mod text;
