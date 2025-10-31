//! Shrinking functionality for minimizing failing test cases.

use crate::error::PropertyError;
use std::time::{Duration, Instant};

/// Trait for types that can be shrunk to smaller values
pub trait Shrinkable {
    /// Create an iterator of shrunk values
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>>;
}

/// Result of a shrinking operation
#[derive(Debug, Clone)]
pub struct ShrinkResult<T> {
    /// Original value that failed
    pub original: T,
    /// Minimal value that still fails
    pub minimal: T,
    /// Number of shrinking steps taken
    pub shrink_steps: usize,
    /// Time spent shrinking
    pub shrink_duration: Duration,
    /// Whether shrinking was completed or timed out
    pub completed: bool,
}

impl<T> ShrinkResult<T> {
    /// Create a new shrink result
    pub fn new(
        original: T,
        minimal: T,
        shrink_steps: usize,
        shrink_duration: Duration,
        completed: bool,
    ) -> Self {
        Self {
            original,
            minimal,
            shrink_steps,
            shrink_duration,
            completed,
        }
    }

    /// Create a shrink result for when no shrinking was performed
    pub fn no_shrinking(original: T) -> Self
    where
        T: Clone,
    {
        Self {
            minimal: original.clone(),
            original,
            shrink_steps: 0,
            shrink_duration: Duration::from_secs(0),
            completed: true,
        }
    }
}

/// Configuration for shrinking behavior
#[derive(Debug, Clone)]
pub struct ShrinkConfig {
    /// Maximum number of shrinking iterations
    pub max_iterations: usize,
    /// Timeout for shrinking process
    pub timeout: Duration,
    /// Whether to enable verbose shrinking output
    pub verbose: bool,
}

impl Default for ShrinkConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            timeout: Duration::from_secs(10),
            verbose: false,
        }
    }
}

impl ShrinkConfig {
    /// Create a new shrink configuration
    pub fn new(max_iterations: usize, timeout: Duration, verbose: bool) -> Self {
        Self {
            max_iterations,
            timeout,
            verbose,
        }
    }

    /// Create a shrink configuration with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }

    /// Create a shrink configuration with custom max iterations
    pub fn with_max_iterations(max_iterations: usize) -> Self {
        Self {
            max_iterations,
            ..Default::default()
        }
    }

    /// Enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

/// Shrinking engine that coordinates the shrinking process
pub struct ShrinkEngine {
    config: ShrinkConfig,
}

impl ShrinkEngine {
    /// Create a new shrinking engine with default configuration
    pub fn new() -> Self {
        Self {
            config: ShrinkConfig::default(),
        }
    }

    /// Create a new shrinking engine with custom configuration
    pub fn with_config(config: ShrinkConfig) -> Self {
        Self { config }
    }

    /// Shrink a value using a property test function
    pub fn shrink<T, F>(&self, value: T, property: F) -> ShrinkResult<T>
    where
        T: Shrinkable + Clone,
        F: Fn(&T) -> Result<(), PropertyError>,
    {
        let start_time = Instant::now();
        let mut current = value.clone();
        let mut shrink_steps = 0;
        let mut last_successful_shrink = current.clone();

        // First, verify that the original value actually fails
        if property(&current).is_ok() {
            // Original value doesn't fail, no shrinking needed
            return ShrinkResult::new(value, current, 0, start_time.elapsed(), true);
        }

        while shrink_steps < self.config.max_iterations {
            // Check timeout
            if start_time.elapsed() >= self.config.timeout {
                if self.config.verbose {
                    eprintln!("Shrinking timed out after {} steps", shrink_steps);
                }
                return ShrinkResult::new(
                    value,
                    last_successful_shrink,
                    shrink_steps,
                    start_time.elapsed(),
                    false,
                );
            }

            let mut found_smaller = false;

            // Try all shrunk values from current
            for shrunk in current.shrink() {
                // Test if this shrunk value still fails
                if property(&shrunk).is_err() {
                    // Found a smaller failing value
                    current = shrunk;
                    last_successful_shrink = current.clone();
                    shrink_steps += 1;
                    found_smaller = true;

                    if self.config.verbose {
                        eprintln!("Shrink step {}: found smaller failing value", shrink_steps);
                    }
                    break;
                }
            }

            // If no smaller failing value was found, we're done
            if !found_smaller {
                break;
            }
        }

        if self.config.verbose {
            eprintln!("Shrinking completed after {} steps", shrink_steps);
        }

        ShrinkResult::new(
            value,
            last_successful_shrink,
            shrink_steps,
            start_time.elapsed(),
            shrink_steps < self.config.max_iterations,
        )
    }

    /// Shrink a value with a custom shrinking strategy
    pub fn shrink_with_strategy<T, F, S>(
        &self,
        value: T,
        property: F,
        strategy: S,
    ) -> ShrinkResult<T>
    where
        T: Clone,
        F: Fn(&T) -> Result<(), PropertyError>,
        S: Fn(&T) -> Box<dyn Iterator<Item = T>>,
    {
        let start_time = Instant::now();
        let mut current = value.clone();
        let mut shrink_steps = 0;
        let mut last_successful_shrink = current.clone();

        // First, verify that the original value actually fails
        if property(&current).is_ok() {
            return ShrinkResult::new(value, current, 0, start_time.elapsed(), true);
        }

        while shrink_steps < self.config.max_iterations {
            // Check timeout
            if start_time.elapsed() >= self.config.timeout {
                if self.config.verbose {
                    eprintln!("Shrinking timed out after {} steps", shrink_steps);
                }
                return ShrinkResult::new(
                    value,
                    last_successful_shrink,
                    shrink_steps,
                    start_time.elapsed(),
                    false,
                );
            }

            let mut found_smaller = false;

            // Try all shrunk values using custom strategy
            for shrunk in strategy(&current) {
                // Test if this shrunk value still fails
                if property(&shrunk).is_err() {
                    // Found a smaller failing value
                    current = shrunk;
                    last_successful_shrink = current.clone();
                    shrink_steps += 1;
                    found_smaller = true;

                    if self.config.verbose {
                        eprintln!("Shrink step {}: found smaller failing value", shrink_steps);
                    }
                    break;
                }
            }

            // If no smaller failing value was found, we're done
            if !found_smaller {
                break;
            }
        }

        if self.config.verbose {
            eprintln!("Shrinking completed after {} steps", shrink_steps);
        }

        ShrinkResult::new(
            value,
            last_successful_shrink,
            shrink_steps,
            start_time.elapsed(),
            shrink_steps < self.config.max_iterations,
        )
    }
}

impl Default for ShrinkEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Async shrinking engine that coordinates the shrinking process for async properties
pub struct AsyncShrinkEngine {
    config: ShrinkConfig,
}

impl AsyncShrinkEngine {
    /// Create a new async shrinking engine with default configuration
    pub fn new() -> Self {
        Self {
            config: ShrinkConfig::default(),
        }
    }

    /// Create a new async shrinking engine with custom configuration
    pub fn with_config(config: ShrinkConfig) -> Self {
        Self { config }
    }

    /// Shrink a value using an async property test function
    pub async fn shrink<T, F, Fut>(&self, value: T, property: F) -> ShrinkResult<T>
    where
        T: Shrinkable + Clone + Send + Sync,
        F: Fn(T) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<(), PropertyError>> + Send,
    {
        let start_time = Instant::now();
        let mut current = value.clone();
        let mut shrink_steps = 0;
        let mut last_successful_shrink = current.clone();

        // First, verify that the original value actually fails
        if property(current.clone()).await.is_ok() {
            // Original value doesn't fail, no shrinking needed
            return ShrinkResult::new(value, current, 0, start_time.elapsed(), true);
        }

        while shrink_steps < self.config.max_iterations {
            // Check timeout
            if start_time.elapsed() >= self.config.timeout {
                if self.config.verbose {
                    eprintln!("Async shrinking timed out after {} steps", shrink_steps);
                }
                return ShrinkResult::new(
                    value,
                    last_successful_shrink,
                    shrink_steps,
                    start_time.elapsed(),
                    false,
                );
            }

            let mut found_smaller = false;

            // Try all shrunk values from current, taking the first one that fails
            // The shrinking strategies should provide candidates in order of preference
            for shrunk in current.shrink() {
                // Test if this shrunk value still fails (async)
                if property(shrunk.clone()).await.is_err() {
                    // Found a smaller failing value
                    current = shrunk;
                    last_successful_shrink = current.clone();
                    shrink_steps += 1;
                    found_smaller = true;

                    if self.config.verbose {
                        eprintln!(
                            "Async shrink step {}: found smaller failing value",
                            shrink_steps
                        );
                    }
                    break;
                }
            }

            // If no smaller failing value was found, we're done
            if !found_smaller {
                break;
            }
        }

        if self.config.verbose {
            eprintln!("Async shrinking completed after {} steps", shrink_steps);
        }

        ShrinkResult::new(
            value,
            last_successful_shrink,
            shrink_steps,
            start_time.elapsed(),
            shrink_steps < self.config.max_iterations,
        )
    }

    /// Shrink a value with a custom async shrinking strategy
    pub async fn shrink_with_strategy<T, F, Fut, S>(
        &self,
        value: T,
        property: F,
        strategy: S,
    ) -> ShrinkResult<T>
    where
        T: Clone + Send + Sync,
        F: Fn(T) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<(), PropertyError>> + Send,
        S: Fn(&T) -> Box<dyn Iterator<Item = T>>,
    {
        let start_time = Instant::now();
        let mut current = value.clone();
        let mut shrink_steps = 0;
        let mut last_successful_shrink = current.clone();

        // First, verify that the original value actually fails
        if property(current.clone()).await.is_ok() {
            return ShrinkResult::new(value, current, 0, start_time.elapsed(), true);
        }

        while shrink_steps < self.config.max_iterations {
            // Check timeout
            if start_time.elapsed() >= self.config.timeout {
                if self.config.verbose {
                    eprintln!(
                        "Async shrinking with strategy timed out after {} steps",
                        shrink_steps
                    );
                }
                return ShrinkResult::new(
                    value,
                    last_successful_shrink,
                    shrink_steps,
                    start_time.elapsed(),
                    false,
                );
            }

            let mut found_smaller = false;

            // Try all shrunk values using custom strategy
            for shrunk in strategy(&current) {
                // Test if this shrunk value still fails (async)
                if property(shrunk.clone()).await.is_err() {
                    // Found a smaller failing value
                    current = shrunk;
                    last_successful_shrink = current.clone();
                    shrink_steps += 1;
                    found_smaller = true;

                    if self.config.verbose {
                        eprintln!(
                            "Async shrink step {}: found smaller failing value",
                            shrink_steps
                        );
                    }
                    break;
                }
            }

            // If no smaller failing value was found, we're done
            if !found_smaller {
                break;
            }
        }

        if self.config.verbose {
            eprintln!(
                "Async shrinking with strategy completed after {} steps",
                shrink_steps
            );
        }

        ShrinkResult::new(
            value,
            last_successful_shrink,
            shrink_steps,
            start_time.elapsed(),
            shrink_steps < self.config.max_iterations,
        )
    }

    /// Shrink a value with timeout-based cancellation support
    pub async fn shrink_with_timeout<T, F, Fut>(
        &self,
        value: T,
        property: F,
        timeout: Duration,
    ) -> ShrinkResult<T>
    where
        T: Shrinkable + Clone + Send + Sync,
        F: Fn(T) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<(), PropertyError>> + Send,
    {
        let start_time = Instant::now();
        let mut current = value.clone();
        let mut shrink_steps = 0;
        let mut last_successful_shrink = current.clone();

        // First, verify that the original value actually fails
        if property(current.clone()).await.is_ok() {
            return ShrinkResult::new(value, current, 0, start_time.elapsed(), true);
        }

        while shrink_steps < self.config.max_iterations {
            // Check timeout (use the more restrictive of the two timeouts)
            let effective_timeout = std::cmp::min(self.config.timeout, timeout);
            if start_time.elapsed() >= effective_timeout {
                if self.config.verbose {
                    eprintln!("Async shrinking timed out after {} steps", shrink_steps);
                }
                return ShrinkResult::new(
                    value,
                    last_successful_shrink,
                    shrink_steps,
                    start_time.elapsed(),
                    false,
                );
            }

            let mut found_smaller = false;

            // Try all shrunk values from current
            for shrunk in current.shrink() {
                // Check timeout before each test
                if start_time.elapsed() >= effective_timeout {
                    return ShrinkResult::new(
                        value,
                        last_successful_shrink,
                        shrink_steps,
                        start_time.elapsed(),
                        false,
                    );
                }

                // Test if this shrunk value still fails (async)
                if property(shrunk.clone()).await.is_err() {
                    // Found a smaller failing value
                    current = shrunk;
                    last_successful_shrink = current.clone();
                    shrink_steps += 1;
                    found_smaller = true;

                    if self.config.verbose {
                        eprintln!(
                            "Async shrink step {}: found smaller failing value",
                            shrink_steps
                        );
                    }
                    break;
                }
            }

            // If no smaller failing value was found, we're done
            if !found_smaller {
                break;
            }
        }

        if self.config.verbose {
            eprintln!("Async shrinking completed after {} steps", shrink_steps);
        }

        ShrinkResult::new(
            value,
            last_successful_shrink,
            shrink_steps,
            start_time.elapsed(),
            shrink_steps < self.config.max_iterations,
        )
    }
}

impl Default for AsyncShrinkEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic shrinking strategies for common patterns
pub mod strategies {
    use super::*;

    /// Binary search shrinking for numeric types
    pub fn binary_search_shrink<T>(value: &T, zero: T) -> Box<dyn Iterator<Item = T> + '_>
    where
        T: Clone + PartialOrd + std::ops::Add<Output = T> + std::ops::Div<Output = T> + From<u8>,
    {
        let mut candidates = Vec::new();

        // Always try zero first if it's smaller
        if zero < *value {
            candidates.push(zero.clone());
        }

        // Binary search approach - try values between zero and current
        let mut current = value.clone();
        let two = T::from(2u8);

        while current > zero {
            let mid = (current.clone() + zero.clone()) / two.clone();
            if mid < current && mid >= zero {
                candidates.push(mid.clone());
                current = mid;
            } else {
                break;
            }
        }

        Box::new(candidates.into_iter())
    }

    /// Linear shrinking - try smaller values by decrementing
    pub fn linear_shrink<T>(value: &T, step: T, min: T) -> Box<dyn Iterator<Item = T> + '_>
    where
        T: Clone + PartialOrd + std::ops::Sub<Output = T>,
    {
        let mut candidates = Vec::new();
        let mut current = value.clone();

        while current > min {
            current = current - step.clone();
            if current >= min {
                candidates.push(current.clone());
            }
        }

        Box::new(candidates.into_iter())
    }

    /// Collection shrinking - try removing elements
    pub fn collection_shrink<T>(collection: &[T]) -> Box<dyn Iterator<Item = Vec<T>> + '_>
    where
        T: Clone,
    {
        let mut candidates = Vec::new();

        // Try empty collection first
        if !collection.is_empty() {
            candidates.push(Vec::new());
        }

        // Try removing each element
        for i in 0..collection.len() {
            let mut shrunk = collection.to_vec();
            shrunk.remove(i);
            candidates.push(shrunk);
        }

        // Try removing half the elements
        if collection.len() > 2 {
            let mid = collection.len() / 2;
            candidates.push(collection[..mid].to_vec());
            candidates.push(collection[mid..].to_vec());
        }

        Box::new(candidates.into_iter())
    }

    /// String shrinking - try shorter strings
    pub fn string_shrink(s: &str) -> Box<dyn Iterator<Item = String> + '_> {
        let mut candidates = Vec::new();

        // Try empty string first
        if !s.is_empty() {
            candidates.push(String::new());
        }

        // Try removing characters from the end
        let chars: Vec<char> = s.chars().collect();
        for i in (1..chars.len()).rev() {
            let shrunk: String = chars[..i].iter().collect();
            candidates.push(shrunk);
        }

        // Try removing characters from the beginning
        for i in 1..chars.len() {
            let shrunk: String = chars[i..].iter().collect();
            candidates.push(shrunk);
        }

        // Try removing characters from the middle
        if chars.len() > 2 {
            let mid = chars.len() / 2;
            let mut shrunk = chars.clone();
            shrunk.remove(mid);
            candidates.push(shrunk.iter().collect());
        }

        Box::new(candidates.into_iter())
    }

    /// Boolean shrinking - only try false if true
    pub fn bool_shrink(value: &bool) -> Box<dyn Iterator<Item = bool> + '_> {
        if *value {
            Box::new(std::iter::once(false))
        } else {
            Box::new(std::iter::empty())
        }
    }

    /// Character shrinking - try simpler characters
    pub fn char_shrink(c: &char) -> Box<dyn Iterator<Item = char> + '_> {
        let mut candidates = Vec::new();

        // Try common simple characters in order of preference
        let simple_chars = ['a', 'A', '0', ' ', '\0'];

        for &simple in &simple_chars {
            if simple < *c {
                candidates.push(simple);
            }
        }

        // Try characters with smaller Unicode values
        let code = *c as u32;
        if code > 0 {
            // Try some intermediate values
            for step in [code / 2, code / 4, code / 8] {
                if step > 0
                    && step < code
                    && let Some(shrunk_char) = char::from_u32(step)
                {
                    candidates.push(shrunk_char);
                }
            }
        }

        Box::new(candidates.into_iter())
    }

    /// Signed integer shrinking with negative value handling
    pub fn signed_int_shrink<T>(value: &T) -> Box<dyn Iterator<Item = T> + '_>
    where
        T: Clone
            + PartialOrd
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + num_traits::Zero
            + num_traits::One
            + num_traits::Signed,
    {
        let mut candidates = Vec::new();
        let zero = T::zero();

        // Always try zero first
        if *value != zero {
            candidates.push(zero.clone());
        }

        if *value > zero {
            // Positive number - use binary search towards zero
            let mut current = value.clone();
            let two = T::one() + T::one();

            while current > zero {
                let mid = current.clone() / two.clone();
                if mid < current && mid >= zero {
                    candidates.push(mid.clone());
                    current = mid;
                } else {
                    break;
                }
            }
        } else if *value < zero {
            // Negative number - try moving towards zero
            let mut current = value.clone();
            let two = T::one() + T::one();

            while current < zero {
                let mid = current.clone() / two.clone();
                if mid > current && mid <= zero {
                    candidates.push(mid.clone());
                    current = mid;
                } else {
                    break;
                }
            }

            // Also try the positive version
            let positive = value.clone().neg();
            if positive > zero {
                candidates.push(positive);
            }
        }

        Box::new(candidates.into_iter())
    }

    /// Unsigned integer shrinking
    pub fn unsigned_int_shrink<T>(value: &T) -> Box<dyn Iterator<Item = T> + '_>
    where
        T: Clone + PartialOrd + std::ops::Div<Output = T> + num_traits::Zero + num_traits::One,
    {
        let mut candidates = Vec::new();
        let zero = T::zero();

        // Always try zero first
        if *value != zero {
            candidates.push(zero.clone());
        }

        // Binary search towards zero
        let mut current = value.clone();
        let two = T::one() + T::one();

        while current > zero {
            let mid = current.clone() / two.clone();
            if mid < current {
                candidates.push(mid.clone());
                current = mid;
            } else {
                break;
            }
        }

        Box::new(candidates.into_iter())
    }

    /// Float shrinking with special handling for NaN and infinity
    pub fn float_shrink<T>(value: &T) -> Box<dyn Iterator<Item = T> + '_>
    where
        T: Clone
            + PartialOrd
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + From<f32>
            + Copy,
        T: std::fmt::Debug,
    {
        let mut candidates = Vec::new();
        let zero = T::from(0.0f32);
        let one = T::from(1.0f32);

        // Handle special float values
        if value.partial_cmp(&zero).is_none() {
            // NaN case - try zero
            candidates.push(zero);
            return Box::new(candidates.into_iter());
        }

        // Try zero first if different
        if *value != zero {
            candidates.push(zero);
        }

        // Try 1.0 and -1.0 for large values
        if *value > one {
            candidates.push(one);
        }
        let neg_one = T::from(-1.0f32);
        if *value < neg_one {
            candidates.push(neg_one);
        }

        // Binary search approach
        if *value > zero {
            let mut current = *value;
            let two = T::from(2.0f32);

            while current > zero {
                let mid = current / two;
                if mid < current {
                    candidates.push(mid);
                    current = mid;
                } else {
                    break;
                }
            }
        } else if *value < zero {
            let mut current = *value;
            let two = T::from(2.0f32);

            while current < zero {
                let mid = current / two;
                if mid > current {
                    candidates.push(mid);
                    current = mid;
                } else {
                    break;
                }
            }

            // Try positive version
            candidates.push(value.neg());
        }

        Box::new(candidates.into_iter())
    }

    /// Recursive shrinking for nested structures
    /// This provides a generic way to shrink complex nested types
    pub fn recursive_shrink<T, F>(value: &T, get_children: F) -> Box<dyn Iterator<Item = T> + '_>
    where
        T: Clone,
        F: Fn(&T) -> Vec<T> + 'static,
    {
        let mut candidates = Vec::new();

        // Get all possible shrunk versions from children
        let children = get_children(value);
        candidates.extend(children);

        Box::new(candidates.into_iter())
    }

    /// Coordinated shrinking for multiple fields
    /// This tries to shrink multiple fields in coordination, not just independently
    pub fn coordinated_field_shrink<T, F1, F2>(
        value: &T,
        field1_shrink: F1,
        field2_shrink: F2,
    ) -> Box<dyn Iterator<Item = T> + '_>
    where
        T: Clone,
        F1: Fn(&T) -> Vec<T> + 'static,
        F2: Fn(&T) -> Vec<T> + 'static,
    {
        let mut candidates = Vec::new();

        // Try shrinking field1 first
        candidates.extend(field1_shrink(value));

        // Try shrinking field2 first
        candidates.extend(field2_shrink(value));

        // Try shrinking both fields together (more aggressive)
        let field1_shrunk = field1_shrink(value);
        for intermediate in field1_shrunk {
            candidates.extend(field2_shrink(&intermediate));
        }

        Box::new(candidates.into_iter())
    }

    /// Map shrinking - try removing entries and shrinking keys/values
    pub fn map_shrink<K, V, M>(map: &M) -> Box<dyn Iterator<Item = M> + '_>
    where
        K: Clone + std::hash::Hash + Eq,
        V: Clone,
        M: Clone + std::iter::IntoIterator<Item = (K, V)> + std::iter::FromIterator<(K, V)>,
    {
        let mut candidates = Vec::new();

        // Try empty map
        candidates.push(M::from_iter(std::iter::empty()));

        // Convert to vector for easier manipulation
        let entries: Vec<(K, V)> = map.clone().into_iter().collect();

        // Try removing each entry
        for i in 0..entries.len() {
            let mut shrunk_entries = entries.clone();
            shrunk_entries.remove(i);
            candidates.push(M::from_iter(shrunk_entries));
        }

        // Try removing half the entries
        if entries.len() > 2 {
            let mid = entries.len() / 2;
            candidates.push(M::from_iter(entries[..mid].iter().cloned()));
            candidates.push(M::from_iter(entries[mid..].iter().cloned()));
        }

        Box::new(candidates.into_iter())
    }

    /// Set shrinking - try removing elements
    pub fn set_shrink<T, S>(set: &S) -> Box<dyn Iterator<Item = S> + '_>
    where
        T: Clone + std::hash::Hash + Eq,
        S: Clone + std::iter::IntoIterator<Item = T> + std::iter::FromIterator<T>,
    {
        let mut candidates = Vec::new();

        // Try empty set
        candidates.push(S::from_iter(std::iter::empty()));

        // Convert to vector for easier manipulation
        let elements: Vec<T> = set.clone().into_iter().collect();

        // Try removing each element
        for i in 0..elements.len() {
            let mut shrunk_elements = elements.clone();
            shrunk_elements.remove(i);
            candidates.push(S::from_iter(shrunk_elements));
        }

        // Try removing half the elements
        if elements.len() > 2 {
            let mid = elements.len() / 2;
            candidates.push(S::from_iter(elements[..mid].iter().cloned()));
            candidates.push(S::from_iter(elements[mid..].iter().cloned()));
        }

        Box::new(candidates.into_iter())
    }

    /// Tuple shrinking with coordination between fields
    pub fn tuple_shrink<A, B>(tuple: &(A, B)) -> Box<dyn Iterator<Item = (A, B)> + '_>
    where
        A: Shrinkable + Clone,
        B: Shrinkable + Clone,
    {
        let mut candidates = Vec::new();

        // Try shrinking first element
        for shrunk_a in tuple.0.shrink() {
            candidates.push((shrunk_a, tuple.1.clone()));
        }

        // Try shrinking second element
        for shrunk_b in tuple.1.shrink() {
            candidates.push((tuple.0.clone(), shrunk_b));
        }

        // Try shrinking both elements together (coordinated shrinking)
        for shrunk_a in tuple.0.shrink() {
            for shrunk_b in tuple.1.shrink() {
                candidates.push((shrunk_a.clone(), shrunk_b));
            }
        }

        Box::new(candidates.into_iter())
    }

    /// Nested collection shrinking - for `Vec<Vec<T>>`, `Vec<HashMap<K,V>>`, etc.
    pub fn nested_collection_shrink<T, C>(collection: &C) -> Box<dyn Iterator<Item = C> + '_>
    where
        T: Shrinkable + Clone,
        C: Clone + std::iter::IntoIterator<Item = T> + std::iter::FromIterator<T>,
    {
        let mut candidates = Vec::new();

        // Try empty collection
        candidates.push(C::from_iter(std::iter::empty()));

        // Convert to vector for manipulation
        let items: Vec<T> = collection.clone().into_iter().collect();

        // Try removing each item
        for i in 0..items.len() {
            let mut shrunk_items = items.clone();
            shrunk_items.remove(i);
            candidates.push(C::from_iter(shrunk_items));
        }

        // Try shrinking each item
        for i in 0..items.len() {
            for shrunk_item in items[i].shrink() {
                let mut shrunk_items = items.clone();
                shrunk_items[i] = shrunk_item;
                candidates.push(C::from_iter(shrunk_items));
            }
        }

        // Try removing half the items
        if items.len() > 2 {
            let mid = items.len() / 2;
            candidates.push(C::from_iter(items[..mid].iter().cloned()));
            candidates.push(C::from_iter(items[mid..].iter().cloned()));
        }

        Box::new(candidates.into_iter())
    }
}

// Implement Shrinkable for all primitive types

impl Shrinkable for bool {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let candidates: Vec<bool> = strategies::bool_shrink(self).collect();
        Box::new(candidates.into_iter())
    }
}

impl Shrinkable for char {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let candidates: Vec<char> = strategies::char_shrink(self).collect();
        Box::new(candidates.into_iter())
    }
}

impl Shrinkable for String {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let candidates: Vec<String> = strategies::string_shrink(self).collect();
        Box::new(candidates.into_iter())
    }
}

// Macro to implement Shrinkable for signed integer types
macro_rules! impl_shrinkable_signed_int {
    ($($t:ty),*) => {
        $(
            impl Shrinkable for $t {
                fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                    let candidates: Vec<$t> = strategies::signed_int_shrink(self).collect();
                    Box::new(candidates.into_iter())
                }
            }
        )*
    };
}

// Macro to implement Shrinkable for unsigned integer types
macro_rules! impl_shrinkable_unsigned_int {
    ($($t:ty),*) => {
        $(
            impl Shrinkable for $t {
                fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                    let candidates: Vec<$t> = strategies::unsigned_int_shrink(self).collect();
                    Box::new(candidates.into_iter())
                }
            }
        )*
    };
}

// Macro to implement Shrinkable for float types
macro_rules! impl_shrinkable_float {
    ($($t:ty),*) => {
        $(
            impl Shrinkable for $t {
                fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                    let candidates: Vec<$t> = strategies::float_shrink(self).collect();
                    Box::new(candidates.into_iter())
                }
            }
        )*
    };
}

// Apply the macros to implement Shrinkable for all primitive numeric types
impl_shrinkable_signed_int!(i8, i16, i32, i64, i128, isize);
impl_shrinkable_unsigned_int!(u8, u16, u32, u64, u128, usize);
impl_shrinkable_float!(f32, f64);

// Implement Shrinkable for Vec<T> where T: Shrinkable to handle element shrinking
impl<T: Shrinkable + Clone + 'static> Shrinkable for Vec<T> {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates: Vec<Vec<T>> = strategies::collection_shrink(self).collect();

        // Also try shrinking individual elements
        for i in 0..self.len() {
            for shrunk_element in self[i].shrink() {
                let mut shrunk_vec = self.clone();
                shrunk_vec[i] = shrunk_element;
                candidates.push(shrunk_vec);
            }
        }

        Box::new(candidates.into_iter())
    }
}

// Note: We don't implement Shrinkable for &[T] because slices are borrowed
// and we can't create new borrowed slices with different lifetimes

// Implement Shrinkable for tuples
impl<A: Shrinkable + Clone + 'static, B: Shrinkable + Clone + 'static> Shrinkable for (A, B) {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try shrinking the first element
        for shrunk_a in self.0.shrink() {
            candidates.push((shrunk_a, self.1.clone()));
        }

        // Try shrinking the second element
        for shrunk_b in self.1.shrink() {
            candidates.push((self.0.clone(), shrunk_b));
        }

        Box::new(candidates.into_iter())
    }
}

impl<
    A: Shrinkable + Clone + 'static,
    B: Shrinkable + Clone + 'static,
    C: Shrinkable + Clone + 'static,
> Shrinkable for (A, B, C)
{
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try shrinking each element
        for shrunk_a in self.0.shrink() {
            candidates.push((shrunk_a, self.1.clone(), self.2.clone()));
        }

        for shrunk_b in self.1.shrink() {
            candidates.push((self.0.clone(), shrunk_b, self.2.clone()));
        }

        for shrunk_c in self.2.shrink() {
            candidates.push((self.0.clone(), self.1.clone(), shrunk_c));
        }

        Box::new(candidates.into_iter())
    }
}

// Implement Shrinkable for Option
impl<T: Shrinkable + Clone + 'static> Shrinkable for Option<T> {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            None => Box::new(std::iter::empty()),
            Some(value) => {
                let mut candidates = vec![None]; // Always try None first

                // Try shrinking the contained value
                for shrunk in value.shrink() {
                    candidates.push(Some(shrunk));
                }

                Box::new(candidates.into_iter())
            }
        }
    }
}

// Implement Shrinkable for Result
impl<T: Shrinkable + Clone + 'static, E: Shrinkable + Clone + 'static> Shrinkable for Result<T, E> {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        match self {
            Ok(value) => {
                // Try shrinking the Ok value
                for shrunk in value.shrink() {
                    candidates.push(Ok(shrunk));
                }
            }
            Err(error) => {
                // Try shrinking the Err value
                for shrunk in error.shrink() {
                    candidates.push(Err(shrunk));
                }
            }
        }

        Box::new(candidates.into_iter())
    }
}

// Implement Shrinkable for larger tuples
impl<A, B, C, D> Shrinkable for (A, B, C, D)
where
    A: Shrinkable + Clone + 'static,
    B: Shrinkable + Clone + 'static,
    C: Shrinkable + Clone + 'static,
    D: Shrinkable + Clone + 'static,
{
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try shrinking each element independently
        for shrunk_a in self.0.shrink() {
            candidates.push((shrunk_a, self.1.clone(), self.2.clone(), self.3.clone()));
        }

        for shrunk_b in self.1.shrink() {
            candidates.push((self.0.clone(), shrunk_b, self.2.clone(), self.3.clone()));
        }

        for shrunk_c in self.2.shrink() {
            candidates.push((self.0.clone(), self.1.clone(), shrunk_c, self.3.clone()));
        }

        for shrunk_d in self.3.shrink() {
            candidates.push((self.0.clone(), self.1.clone(), self.2.clone(), shrunk_d));
        }

        Box::new(candidates.into_iter())
    }
}

impl<A, B, C, D, E> Shrinkable for (A, B, C, D, E)
where
    A: Shrinkable + Clone + 'static,
    B: Shrinkable + Clone + 'static,
    C: Shrinkable + Clone + 'static,
    D: Shrinkable + Clone + 'static,
    E: Shrinkable + Clone + 'static,
{
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try shrinking each element independently
        for shrunk_a in self.0.shrink() {
            candidates.push((
                shrunk_a,
                self.1.clone(),
                self.2.clone(),
                self.3.clone(),
                self.4.clone(),
            ));
        }

        for shrunk_b in self.1.shrink() {
            candidates.push((
                self.0.clone(),
                shrunk_b,
                self.2.clone(),
                self.3.clone(),
                self.4.clone(),
            ));
        }

        for shrunk_c in self.2.shrink() {
            candidates.push((
                self.0.clone(),
                self.1.clone(),
                shrunk_c,
                self.3.clone(),
                self.4.clone(),
            ));
        }

        for shrunk_d in self.3.shrink() {
            candidates.push((
                self.0.clone(),
                self.1.clone(),
                self.2.clone(),
                shrunk_d,
                self.4.clone(),
            ));
        }

        for shrunk_e in self.4.shrink() {
            candidates.push((
                self.0.clone(),
                self.1.clone(),
                self.2.clone(),
                self.3.clone(),
                shrunk_e,
            ));
        }

        Box::new(candidates.into_iter())
    }
}

// Implement Shrinkable for HashMap
impl<K, V> Shrinkable for std::collections::HashMap<K, V>
where
    K: Shrinkable + Clone + std::hash::Hash + Eq + 'static,
    V: Shrinkable + Clone + 'static,
{
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try empty map first
        if !self.is_empty() {
            candidates.push(std::collections::HashMap::new());
        }

        // Try removing each key-value pair
        for key in self.keys() {
            let mut shrunk = self.clone();
            shrunk.remove(key);
            candidates.push(shrunk);
        }

        // Try shrinking individual keys and values
        for (key, value) in self.iter() {
            // Try shrinking the key
            for shrunk_key in key.shrink() {
                if !self.contains_key(&shrunk_key) {
                    let mut shrunk_map = self.clone();
                    shrunk_map.remove(key);
                    shrunk_map.insert(shrunk_key, value.clone());
                    candidates.push(shrunk_map);
                }
            }

            // Try shrinking the value
            for shrunk_value in value.shrink() {
                let mut shrunk_map = self.clone();
                shrunk_map.insert(key.clone(), shrunk_value);
                candidates.push(shrunk_map);
            }
        }

        // Try removing half the entries
        if self.len() > 2 {
            let keys: Vec<_> = self.keys().cloned().collect();
            let mid = keys.len() / 2;

            // First half
            let mut first_half = std::collections::HashMap::new();
            for key in &keys[..mid] {
                if let Some(value) = self.get(key) {
                    first_half.insert(key.clone(), value.clone());
                }
            }
            candidates.push(first_half);

            // Second half
            let mut second_half = std::collections::HashMap::new();
            for key in &keys[mid..] {
                if let Some(value) = self.get(key) {
                    second_half.insert(key.clone(), value.clone());
                }
            }
            candidates.push(second_half);
        }

        Box::new(candidates.into_iter())
    }
}

// Implement Shrinkable for BTreeMap
impl<K, V> Shrinkable for std::collections::BTreeMap<K, V>
where
    K: Shrinkable + Clone + Ord + 'static,
    V: Shrinkable + Clone + 'static,
{
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try empty map first
        if !self.is_empty() {
            candidates.push(std::collections::BTreeMap::new());
        }

        // Try removing each key-value pair
        for key in self.keys() {
            let mut shrunk = self.clone();
            shrunk.remove(key);
            candidates.push(shrunk);
        }

        // Try shrinking individual keys and values
        for (key, value) in self.iter() {
            // Try shrinking the key
            for shrunk_key in key.shrink() {
                if !self.contains_key(&shrunk_key) {
                    let mut shrunk_map = self.clone();
                    shrunk_map.remove(key);
                    shrunk_map.insert(shrunk_key, value.clone());
                    candidates.push(shrunk_map);
                }
            }

            // Try shrinking the value
            for shrunk_value in value.shrink() {
                let mut shrunk_map = self.clone();
                shrunk_map.insert(key.clone(), shrunk_value);
                candidates.push(shrunk_map);
            }
        }

        // Try removing half the entries
        if self.len() > 2 {
            let keys: Vec<_> = self.keys().cloned().collect();
            let mid = keys.len() / 2;

            // First half
            let mut first_half = std::collections::BTreeMap::new();
            for key in &keys[..mid] {
                if let Some(value) = self.get(key) {
                    first_half.insert(key.clone(), value.clone());
                }
            }
            candidates.push(first_half);

            // Second half
            let mut second_half = std::collections::BTreeMap::new();
            for key in &keys[mid..] {
                if let Some(value) = self.get(key) {
                    second_half.insert(key.clone(), value.clone());
                }
            }
            candidates.push(second_half);
        }

        Box::new(candidates.into_iter())
    }
}

// Implement Shrinkable for HashSet
impl<T> Shrinkable for std::collections::HashSet<T>
where
    T: Shrinkable + Clone + std::hash::Hash + Eq + 'static,
{
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try empty set first
        if !self.is_empty() {
            candidates.push(std::collections::HashSet::new());
        }

        // Try removing each element
        for item in self.iter() {
            let mut shrunk = self.clone();
            shrunk.remove(item);
            candidates.push(shrunk);
        }

        // Try shrinking individual elements
        for item in self.iter() {
            for shrunk_item in item.shrink() {
                if !self.contains(&shrunk_item) {
                    let mut shrunk_set = self.clone();
                    shrunk_set.remove(item);
                    shrunk_set.insert(shrunk_item);
                    candidates.push(shrunk_set);
                }
            }
        }

        // Try removing half the elements
        if self.len() > 2 {
            let items: Vec<_> = self.iter().cloned().collect();
            let mid = items.len() / 2;

            // First half
            let first_half: std::collections::HashSet<_> = items[..mid].iter().cloned().collect();
            candidates.push(first_half);

            // Second half
            let second_half: std::collections::HashSet<_> = items[mid..].iter().cloned().collect();
            candidates.push(second_half);
        }

        Box::new(candidates.into_iter())
    }
}

// Implement Shrinkable for BTreeSet
impl<T> Shrinkable for std::collections::BTreeSet<T>
where
    T: Shrinkable + Clone + Ord + 'static,
{
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut candidates = Vec::new();

        // Try empty set first
        if !self.is_empty() {
            candidates.push(std::collections::BTreeSet::new());
        }

        // Try removing each element
        for item in self.iter() {
            let mut shrunk = self.clone();
            shrunk.remove(item);
            candidates.push(shrunk);
        }

        // Try shrinking individual elements
        for item in self.iter() {
            for shrunk_item in item.shrink() {
                if !self.contains(&shrunk_item) {
                    let mut shrunk_set = self.clone();
                    shrunk_set.remove(item);
                    shrunk_set.insert(shrunk_item);
                    candidates.push(shrunk_set);
                }
            }
        }

        // Try removing half the elements
        if self.len() > 2 {
            let items: Vec<_> = self.iter().cloned().collect();
            let mid = items.len() / 2;

            // First half
            let first_half: std::collections::BTreeSet<_> = items[..mid].iter().cloned().collect();
            candidates.push(first_half);

            // Second half
            let second_half: std::collections::BTreeSet<_> = items[mid..].iter().cloned().collect();
            candidates.push(second_half);
        }

        Box::new(candidates.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_shrink_result_creation() {
        let result = ShrinkResult::new(100, 5, 10, Duration::from_millis(500), true);

        assert_eq!(result.original, 100);
        assert_eq!(result.minimal, 5);
        assert_eq!(result.shrink_steps, 10);
        assert_eq!(result.shrink_duration, Duration::from_millis(500));
        assert!(result.completed);
    }

    #[test]
    fn test_shrink_result_no_shrinking() {
        let result = ShrinkResult::no_shrinking(42);

        assert_eq!(result.original, 42);
        assert_eq!(result.minimal, 42);
        assert_eq!(result.shrink_steps, 0);
        assert_eq!(result.shrink_duration, Duration::from_secs(0));
        assert!(result.completed);
    }

    #[test]
    fn test_shrink_config_defaults() {
        let config = ShrinkConfig::default();

        assert_eq!(config.max_iterations, 1000);
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert!(!config.verbose);
    }

    #[test]
    fn test_shrink_config_builders() {
        let config = ShrinkConfig::with_timeout(Duration::from_secs(5));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.max_iterations, 1000); // Default

        let config = ShrinkConfig::with_max_iterations(500);
        assert_eq!(config.max_iterations, 500);
        assert_eq!(config.timeout, Duration::from_secs(10)); // Default

        let config = ShrinkConfig::default().verbose();
        assert!(config.verbose);
    }

    #[test]
    fn test_shrink_engine_creation() {
        let engine = ShrinkEngine::new();
        assert_eq!(engine.config.max_iterations, 1000);

        let custom_config = ShrinkConfig::with_max_iterations(500);
        let engine = ShrinkEngine::with_config(custom_config);
        assert_eq!(engine.config.max_iterations, 500);
    }

    #[test]
    fn test_shrink_engine_basic_shrinking() {
        let engine = ShrinkEngine::new();

        // Property that fails for values > 10
        let property = |x: &i32| {
            if *x > 10 {
                Err(PropertyError::PropertyFailed {
                    message: format!("Value {} is too large", x),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let result = engine.shrink(100, property);

        assert_eq!(result.original, 100);
        // The binary search should find a value that still fails but is close to the boundary
        // Since our property fails for values > 10, the minimal failing value should be > 10
        assert!(result.minimal > 10);
        assert!(result.shrink_steps > 0);
        assert!(result.completed);
    }

    #[test]
    fn test_shrink_engine_no_shrinking_needed() {
        let engine = ShrinkEngine::new();

        // Property that never fails
        let property = |_x: &i32| Ok(());

        let result = engine.shrink(100, property);

        assert_eq!(result.original, 100);
        assert_eq!(result.minimal, 100);
        assert_eq!(result.shrink_steps, 0);
        assert!(result.completed);
    }

    #[test]
    fn test_shrink_engine_timeout() {
        let config = ShrinkConfig::with_timeout(Duration::from_millis(1));
        let engine = ShrinkEngine::with_config(config);

        // Property that always fails (will cause timeout)
        let property = |_x: &i32| {
            // Add a small delay to ensure timeout
            std::thread::sleep(Duration::from_millis(2));
            Err(PropertyError::PropertyFailed {
                message: "Always fails".to_string(),
                context: None,
                iteration: None,
            })
        };

        let result = engine.shrink(100, property);

        assert_eq!(result.original, 100);
        assert!(!result.completed); // Should timeout
    }

    #[test]
    fn test_shrink_engine_max_iterations() {
        let config = ShrinkConfig::with_max_iterations(5);
        let engine = ShrinkEngine::with_config(config);

        // Property that always fails
        let property = |_x: &i32| {
            Err(PropertyError::PropertyFailed {
                message: "Always fails".to_string(),
                context: None,
                iteration: None,
            })
        };

        let result = engine.shrink(100, property);

        assert_eq!(result.original, 100);
        // Should hit max iterations or complete earlier if no more shrinking is possible
        assert!(result.shrink_steps <= 5);
        // May complete if binary search finds no more candidates quickly
        if result.shrink_steps == 5 {
            assert!(!result.completed);
        }
    }

    #[test]
    fn test_shrink_engine_with_custom_strategy() {
        let engine = ShrinkEngine::new();

        // Custom strategy that just tries half the value
        let strategy = |x: &i32| {
            if *x > 0 {
                Box::new(std::iter::once(*x / 2)) as Box<dyn Iterator<Item = i32>>
            } else {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = i32>>
            }
        };

        // Property that fails for values > 5
        let property = |x: &i32| {
            if *x > 5 {
                Err(PropertyError::PropertyFailed {
                    message: format!("Value {} is too large", x),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let result = engine.shrink_with_strategy(20, property, strategy);

        assert_eq!(result.original, 20);
        // The minimal failing value should be > 5 (since that's what fails)
        // With halving strategy: 20 -> 10 -> 5, but 5 passes, so minimal should be 10
        assert!(result.minimal > 5);
        assert!(result.shrink_steps > 0);
        assert!(result.completed);
    }

    #[test]
    fn test_binary_search_shrink_strategy() {
        let shrunk: Vec<i32> = strategies::binary_search_shrink(&100, 0).collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0)); // Should always try zero first

        // All shrunk values should be smaller than original
        for value in &shrunk {
            assert!(*value < 100);
            assert!(*value >= 0);
        }
    }

    #[test]
    fn test_linear_shrink_strategy() {
        let shrunk: Vec<i32> = strategies::linear_shrink(&10, 2, 0).collect();

        assert!(!shrunk.is_empty());

        // Should contain values like 8, 6, 4, 2, 0
        let expected = vec![8, 6, 4, 2, 0];
        assert_eq!(shrunk, expected);
    }

    #[test]
    fn test_collection_shrink_strategy() {
        let original = vec![1, 2, 3, 4];
        let shrunk: Vec<Vec<i32>> = strategies::collection_shrink(&original).collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&vec![])); // Should try empty collection

        // Should try removing each element
        assert!(shrunk.contains(&vec![2, 3, 4])); // Remove first
        assert!(shrunk.contains(&vec![1, 3, 4])); // Remove second
        assert!(shrunk.contains(&vec![1, 2, 4])); // Remove third
        assert!(shrunk.contains(&vec![1, 2, 3])); // Remove fourth

        // Should try halves
        assert!(shrunk.contains(&vec![1, 2])); // First half
        assert!(shrunk.contains(&vec![3, 4])); // Second half
    }

    #[test]
    fn test_collection_shrink_empty() {
        let original: Vec<i32> = vec![];
        let shrunk: Vec<Vec<i32>> = strategies::collection_shrink(&original).collect();

        assert!(shrunk.is_empty()); // No shrinking possible for empty collection
    }

    #[test]
    fn test_collection_shrink_single_element() {
        let original = vec![42];
        let shrunk: Vec<Vec<i32>> = strategies::collection_shrink(&original).collect();

        // Should try empty collection and removing the single element (which results in empty)
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&vec![])); // Should try empty collection
    }

    #[test]
    fn test_shrinkable_trait_for_integers() {
        let value = 50;
        let shrunk: Vec<i32> = value.shrink().collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0));

        // All values should be smaller
        for shrunk_value in shrunk {
            assert!(shrunk_value < value);
        }
    }

    #[test]
    fn test_shrinkable_trait_for_vectors() {
        let value = vec![1, 2, 3];
        let shrunk: Vec<Vec<i32>> = value.shrink().collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&vec![]));

        // All shrunk collections should be smaller or equal in size
        for shrunk_vec in shrunk {
            assert!(shrunk_vec.len() <= value.len());
        }
    }

    #[test]
    fn test_shrink_engine_integration_with_vectors() {
        let engine = ShrinkEngine::new();

        // Property that fails for vectors with length > 2
        let property = |v: &Vec<i32>| {
            if v.len() > 2 {
                Err(PropertyError::PropertyFailed {
                    message: format!("Vector too long: {}", v.len()),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let original = vec![1, 2, 3, 4, 5];
        let result = engine.shrink(original.clone(), property);

        assert_eq!(result.original, original);
        // The minimal failing vector should have length > 2 (since that's what fails)
        assert!(result.minimal.len() > 2);
        // But it should be smaller than the original
        assert!(result.minimal.len() < original.len());
        assert!(result.shrink_steps > 0);
        assert!(result.completed);
    }

    #[test]
    fn test_shrink_result_clone() {
        let result = ShrinkResult::new(100, 5, 10, Duration::from_millis(500), true);

        let cloned = result.clone();
        assert_eq!(cloned.original, result.original);
        assert_eq!(cloned.minimal, result.minimal);
        assert_eq!(cloned.shrink_steps, result.shrink_steps);
        assert_eq!(cloned.shrink_duration, result.shrink_duration);
        assert_eq!(cloned.completed, result.completed);
    }

    // Tests for primitive type shrinking

    #[test]
    fn test_bool_shrinking() {
        // true should shrink to false
        let shrunk: Vec<bool> = true.shrink().collect();
        assert_eq!(shrunk, vec![false]);

        // false should not shrink
        let shrunk: Vec<bool> = false.shrink().collect();
        assert!(shrunk.is_empty());
    }

    #[test]
    fn test_char_shrinking() {
        // 'z' should shrink to simpler characters
        let shrunk: Vec<char> = 'z'.shrink().collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&'a'));

        // All shrunk characters should be "simpler" (smaller Unicode values)
        for &c in &shrunk {
            assert!(c < 'z');
        }

        // 'a' should shrink to even simpler characters
        let shrunk: Vec<char> = 'a'.shrink().collect();
        for &c in &shrunk {
            assert!(c < 'a');
        }
    }

    #[test]
    fn test_string_shrinking() {
        let original = "hello".to_string();
        let shrunk: Vec<String> = original.shrink().collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&String::new())); // Should try empty string

        // All shrunk strings should be shorter or equal
        for s in &shrunk {
            assert!(s.len() <= original.len());
        }

        // Should contain some prefixes and suffixes
        assert!(shrunk.iter().any(|s| s == "hell"));
        assert!(shrunk.iter().any(|s| s == "ello"));
    }

    #[test]
    fn test_signed_integer_shrinking() {
        // Positive number
        let shrunk: Vec<i32> = 100i32.shrink().collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0)); // Should try zero

        // All shrunk values should be smaller in absolute value
        for &val in &shrunk {
            assert!(val.abs() < 100);
        }

        // Negative number
        let shrunk: Vec<i32> = (-50i32).shrink().collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0)); // Should try zero

        // Should try positive version
        assert!(shrunk.contains(&50));

        // All shrunk values should be closer to zero
        for &val in &shrunk {
            assert!(val.abs() <= 50);
        }
    }

    #[test]
    fn test_unsigned_integer_shrinking() {
        let shrunk: Vec<u32> = 100u32.shrink().collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0)); // Should try zero

        // All shrunk values should be smaller
        for &val in &shrunk {
            assert!(val < 100);
        }
    }

    #[test]
    fn test_float_shrinking() {
        // Positive float
        let shrunk: Vec<f64> = 100.5f64.shrink().collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0.0)); // Should try zero

        // All shrunk values should be smaller in absolute value
        for &val in &shrunk {
            assert!(val.abs() < 100.5);
        }

        // Negative float
        let shrunk: Vec<f64> = (-50.5f64).shrink().collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0.0)); // Should try zero

        // Should try positive version
        assert!(shrunk.contains(&50.5));
    }

    #[test]
    fn test_float_shrinking_special_values() {
        // Test NaN
        let shrunk: Vec<f64> = f64::NAN.shrink().collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0.0)); // Should try zero for NaN
    }

    #[test]
    fn test_option_shrinking() {
        // Some value should shrink to None and shrunk inner values
        let original = Some(100i32);
        let shrunk: Vec<Option<i32>> = original.shrink().collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&None)); // Should try None first

        // Should contain shrunk versions of the inner value
        assert!(shrunk.iter().any(|opt| opt == &Some(0)));

        // None should not shrink
        let shrunk: Vec<Option<i32>> = None.shrink().collect();
        assert!(shrunk.is_empty());
    }

    #[test]
    fn test_tuple_shrinking() {
        let original = (100i32, true);
        let shrunk: Vec<(i32, bool)> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try shrinking first element
        assert!(shrunk.iter().any(|(a, b)| *a == 0 && *b));

        // Should try shrinking second element
        assert!(shrunk.iter().any(|(a, b)| *a == 100 && !(*b)));
    }

    #[test]
    fn test_triple_tuple_shrinking() {
        let original = (10i32, true, "hi".to_string());
        let shrunk: Vec<(i32, bool, String)> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try shrinking each element independently
        assert!(shrunk.iter().any(|(a, b, c)| *a == 0 && *b && c == "hi"));
        assert!(
            shrunk
                .iter()
                .any(|(a, b, c)| *a == 10 && !(*b) && c == "hi")
        );
        assert!(
            shrunk
                .iter()
                .any(|(a, b, c)| *a == 10 && *b && c.is_empty())
        );
    }

    #[test]
    fn test_string_shrink_strategy() {
        let shrunk: Vec<String> = strategies::string_shrink("hello").collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&String::new())); // Empty string

        // Should contain prefixes
        assert!(shrunk.contains(&"hell".to_string()));
        assert!(shrunk.contains(&"hel".to_string()));

        // Should contain suffixes
        assert!(shrunk.contains(&"ello".to_string()));
        assert!(shrunk.contains(&"llo".to_string()));
    }

    #[test]
    fn test_bool_shrink_strategy() {
        let shrunk: Vec<bool> = strategies::bool_shrink(&true).collect();
        assert_eq!(shrunk, vec![false]);

        let shrunk: Vec<bool> = strategies::bool_shrink(&false).collect();
        assert!(shrunk.is_empty());
    }

    #[test]
    fn test_char_shrink_strategy() {
        let shrunk: Vec<char> = strategies::char_shrink(&'z').collect();
        assert!(!shrunk.is_empty());

        // Should contain some simple characters
        assert!(shrunk.contains(&'a'));

        // All should be smaller than 'z'
        for &c in &shrunk {
            assert!(c < 'z');
        }
    }

    #[test]
    fn test_signed_int_shrink_strategy() {
        // Positive number
        let shrunk: Vec<i32> = strategies::signed_int_shrink(&100).collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0));

        // Negative number
        let shrunk: Vec<i32> = strategies::signed_int_shrink(&-50).collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0));
        assert!(shrunk.contains(&50)); // Positive version
    }

    #[test]
    fn test_unsigned_int_shrink_strategy() {
        let shrunk: Vec<u32> = strategies::unsigned_int_shrink(&100).collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0));

        // All should be smaller
        for &val in &shrunk {
            assert!(val < 100);
        }
    }

    #[test]
    fn test_float_shrink_strategy() {
        let shrunk: Vec<f64> = strategies::float_shrink(&100.5).collect();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0.0));

        // Should contain 1.0 for large values
        assert!(shrunk.contains(&1.0));
    }

    #[test]
    fn test_comprehensive_shrinking_integration() {
        let engine = ShrinkEngine::new();

        // Test with a complex property that uses multiple types
        let property = |data: &(i32, String, bool)| {
            let (num, text, flag) = data;
            if *num > 50 || text.len() > 3 || *flag {
                Err(PropertyError::PropertyFailed {
                    message: "Complex condition failed".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let original = (100, "hello".to_string(), true);
        let result = engine.shrink(original.clone(), property);

        assert_eq!(result.original, original);
        assert!(result.shrink_steps > 0);
        assert!(result.completed);

        // The minimal case should still fail the property
        let (min_num, min_text, min_flag) = &result.minimal;
        assert!(*min_num > 50 || min_text.len() > 3 || *min_flag);
    }

    // Tests for complex type shrinking

    #[test]
    fn test_result_shrinking() {
        // Ok variant should shrink the inner value
        let ok_result: Result<i32, String> = Ok(100);
        let shrunk: Vec<Result<i32, String>> = ok_result.shrink().collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&Ok(0))); // Should shrink inner value

        // Err variant should shrink the error value
        let err_result: Result<i32, String> = Err("error message".to_string());
        let shrunk: Vec<Result<i32, String>> = err_result.shrink().collect();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&Err(String::new()))); // Should shrink error string
    }

    #[test]
    fn test_four_tuple_shrinking() {
        let original = (10i32, true, "hi".to_string(), 5u32);
        let shrunk: Vec<(i32, bool, String, u32)> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try shrinking each element independently
        assert!(
            shrunk
                .iter()
                .any(|(a, b, c, d)| *a == 0 && *b && c == "hi" && *d == 5)
        );
        assert!(
            shrunk
                .iter()
                .any(|(a, b, c, d)| *a == 10 && !(*b) && c == "hi" && *d == 5)
        );
        assert!(
            shrunk
                .iter()
                .any(|(a, b, c, d)| *a == 10 && *b && c.is_empty() && *d == 5)
        );
        assert!(
            shrunk
                .iter()
                .any(|(a, b, c, d)| *a == 10 && *b && c == "hi" && *d == 0)
        );
    }

    #[test]
    fn test_five_tuple_shrinking() {
        let original = (1i32, 2i32, 3i32, 4i32, 5i32);
        let shrunk: Vec<(i32, i32, i32, i32, i32)> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try shrinking each element independently
        assert!(shrunk.iter().any(|(a, b, c, d, e)| *a == 0
            && *b == 2
            && *c == 3
            && *d == 4
            && *e == 5));
        assert!(shrunk.iter().any(|(a, b, c, d, e)| *a == 1
            && *b == 0
            && *c == 3
            && *d == 4
            && *e == 5));
        assert!(shrunk.iter().any(|(a, b, c, d, e)| *a == 1
            && *b == 2
            && *c == 0
            && *d == 4
            && *e == 5));
        assert!(shrunk.iter().any(|(a, b, c, d, e)| *a == 1
            && *b == 2
            && *c == 3
            && *d == 0
            && *e == 5));
        assert!(shrunk.iter().any(|(a, b, c, d, e)| *a == 1
            && *b == 2
            && *c == 3
            && *d == 4
            && *e == 0));
    }

    #[test]
    fn test_hashmap_shrinking() {
        use std::collections::HashMap;

        let mut original = HashMap::new();
        original.insert("key1".to_string(), 10i32);
        original.insert("key2".to_string(), 20i32);
        original.insert("key3".to_string(), 30i32);

        let shrunk: Vec<HashMap<String, i32>> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try empty map
        assert!(shrunk.iter().any(|m| m.is_empty()));

        // Should try removing each key
        assert!(
            shrunk
                .iter()
                .any(|m| m.len() == 2 && !m.contains_key("key1"))
        );
        assert!(
            shrunk
                .iter()
                .any(|m| m.len() == 2 && !m.contains_key("key2"))
        );
        assert!(
            shrunk
                .iter()
                .any(|m| m.len() == 2 && !m.contains_key("key3"))
        );

        // Should try shrinking values
        assert!(shrunk.iter().any(|m| m.get("key1") == Some(&0)));
        assert!(shrunk.iter().any(|m| m.get("key2") == Some(&0)));
        assert!(shrunk.iter().any(|m| m.get("key3") == Some(&0)));
    }

    #[test]
    fn test_btreemap_shrinking() {
        use std::collections::BTreeMap;

        let mut original = BTreeMap::new();
        original.insert(1i32, "value1".to_string());
        original.insert(2i32, "value2".to_string());
        original.insert(3i32, "value3".to_string());

        let shrunk: Vec<BTreeMap<i32, String>> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try empty map
        assert!(shrunk.iter().any(|m| m.is_empty()));

        // Should try removing each key
        assert!(shrunk.iter().any(|m| m.len() == 2 && !m.contains_key(&1)));
        assert!(shrunk.iter().any(|m| m.len() == 2 && !m.contains_key(&2)));
        assert!(shrunk.iter().any(|m| m.len() == 2 && !m.contains_key(&3)));

        // Should try shrinking values
        assert!(shrunk.iter().any(|m| m.get(&1) == Some(&String::new())));
        assert!(shrunk.iter().any(|m| m.get(&2) == Some(&String::new())));
        assert!(shrunk.iter().any(|m| m.get(&3) == Some(&String::new())));
    }

    #[test]
    fn test_hashset_shrinking() {
        use std::collections::HashSet;

        let mut original = HashSet::new();
        original.insert("item1".to_string());
        original.insert("item2".to_string());
        original.insert("item3".to_string());

        let shrunk: Vec<HashSet<String>> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try empty set
        assert!(shrunk.iter().any(|s| s.is_empty()));

        // Should try removing each item
        assert!(shrunk.iter().any(|s| s.len() == 2 && !s.contains("item1")));
        assert!(shrunk.iter().any(|s| s.len() == 2 && !s.contains("item2")));
        assert!(shrunk.iter().any(|s| s.len() == 2 && !s.contains("item3")));

        // Should try shrinking items
        assert!(shrunk.iter().any(|s| s.contains("")));
    }

    #[test]
    fn test_btreeset_shrinking() {
        use std::collections::BTreeSet;

        let mut original = BTreeSet::new();
        original.insert(10i32);
        original.insert(20i32);
        original.insert(30i32);

        let shrunk: Vec<BTreeSet<i32>> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try empty set
        assert!(shrunk.iter().any(|s| s.is_empty()));

        // Should try removing each item
        assert!(shrunk.iter().any(|s| s.len() == 2 && !s.contains(&10)));
        assert!(shrunk.iter().any(|s| s.len() == 2 && !s.contains(&20)));
        assert!(shrunk.iter().any(|s| s.len() == 2 && !s.contains(&30)));

        // Should try shrinking items
        assert!(shrunk.iter().any(|s| s.contains(&0)));
    }

    #[test]
    fn test_nested_vec_shrinking() {
        let original = vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]];

        let shrunk: Vec<Vec<Vec<i32>>> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try empty outer vector
        assert!(shrunk.iter().any(|v| v.is_empty()));

        // Should try removing each inner vector
        assert!(shrunk.iter().any(|v| v.len() == 2 && v[0] == vec![4, 5]));
        assert!(shrunk.iter().any(|v| v.len() == 2 && v[0] == vec![1, 2, 3]));

        // Should try shrinking inner vectors
        assert!(shrunk.iter().any(|v| v.contains(&vec![])));
        assert!(
            shrunk
                .iter()
                .any(|v| v.iter().any(|inner| inner.contains(&0)))
        );
    }

    #[test]
    fn test_complex_nested_structure_shrinking() {
        use std::collections::HashMap;

        // Create a complex nested structure
        let mut inner_map1 = HashMap::new();
        inner_map1.insert("a".to_string(), vec![1, 2, 3]);
        inner_map1.insert("b".to_string(), vec![4, 5]);

        let mut inner_map2 = HashMap::new();
        inner_map2.insert("c".to_string(), vec![6, 7, 8]);

        let original = vec![inner_map1, inner_map2];

        let shrunk: Vec<Vec<HashMap<String, Vec<i32>>>> = original.shrink().collect();

        assert!(!shrunk.is_empty());

        // Should try empty outer vector
        assert!(shrunk.iter().any(|v| v.is_empty()));

        // Should try removing each inner map
        assert!(shrunk.iter().any(|v| v.len() == 1));

        // Should try shrinking inner maps (empty maps)
        assert!(shrunk.iter().any(|v| v.iter().any(|m| m.is_empty())));

        // Should try shrinking inner vectors
        assert!(
            shrunk
                .iter()
                .any(|v| { v.iter().any(|m| { m.values().any(|vec| vec.is_empty()) }) })
        );
    }

    #[test]
    fn test_shrink_engine_with_complex_types() {
        let engine = ShrinkEngine::new();

        // Property that fails for maps with more than 2 entries
        let property = |map: &std::collections::HashMap<String, i32>| {
            if map.len() > 2 {
                Err(PropertyError::PropertyFailed {
                    message: format!("Map too large: {}", map.len()),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let mut original = std::collections::HashMap::new();
        original.insert("key1".to_string(), 10);
        original.insert("key2".to_string(), 20);
        original.insert("key3".to_string(), 30);
        original.insert("key4".to_string(), 40);

        let result = engine.shrink(original.clone(), property);

        assert_eq!(result.original, original);
        // The minimal failing map should have length > 2 (since that's what fails)
        assert!(result.minimal.len() > 2);
        // But it should be smaller than the original
        assert!(result.minimal.len() < original.len());
        assert!(result.shrink_steps > 0);
        assert!(result.completed);
    }

    #[test]
    fn test_coordinated_shrinking_strategies() {
        // Test the coordinated field shrinking strategy
        let original = (100i32, "hello".to_string());

        let field1_shrink = |tuple: &(i32, String)| {
            let mut candidates = Vec::new();
            for shrunk in tuple.0.shrink() {
                candidates.push((shrunk, tuple.1.clone()));
            }
            candidates
        };

        let field2_shrink = |tuple: &(i32, String)| {
            let mut candidates = Vec::new();
            for shrunk in tuple.1.shrink() {
                candidates.push((tuple.0, shrunk));
            }
            candidates
        };

        let shrunk: Vec<(i32, String)> =
            strategies::coordinated_field_shrink(&original, field1_shrink, field2_shrink).collect();

        assert!(!shrunk.is_empty());

        // Should contain individual field shrinking
        assert!(shrunk.contains(&(0, "hello".to_string())));
        assert!(shrunk.contains(&(100, String::new())));

        // Should contain coordinated shrinking (both fields shrunk)
        assert!(shrunk.contains(&(0, String::new())));
    }

    #[test]
    fn test_recursive_shrinking_strategy() {
        // Test recursive shrinking with a simple tree-like structure
        #[derive(Clone, Debug, PartialEq)]
        struct Node {
            value: i32,
            children: Vec<Node>,
        }

        impl Shrinkable for Node {
            fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                let mut candidates = Vec::new();

                // Try shrinking the value
                for shrunk_value in self.value.shrink() {
                    candidates.push(Node {
                        value: shrunk_value,
                        children: self.children.clone(),
                    });
                }

                // Try shrinking the children vector
                for shrunk_children in self.children.shrink() {
                    candidates.push(Node {
                        value: self.value,
                        children: shrunk_children,
                    });
                }

                Box::new(candidates.into_iter())
            }
        }

        let original = Node {
            value: 10,
            children: vec![
                Node {
                    value: 5,
                    children: vec![],
                },
                Node {
                    value: 15,
                    children: vec![],
                },
            ],
        };

        let get_children = |node: &Node| {
            let mut candidates = Vec::new();

            // Try shrinking the value
            for shrunk_value in node.value.shrink() {
                candidates.push(Node {
                    value: shrunk_value,
                    children: node.children.clone(),
                });
            }

            // Try shrinking the children vector
            for shrunk_children in node.children.shrink() {
                candidates.push(Node {
                    value: node.value,
                    children: shrunk_children,
                });
            }

            candidates
        };

        let shrunk: Vec<Node> = strategies::recursive_shrink(&original, get_children).collect();

        assert!(!shrunk.is_empty());

        // Should try shrinking the root value
        assert!(shrunk.iter().any(|n| n.value == 0 && n.children.len() == 2));

        // Should try shrinking the children
        assert!(
            shrunk
                .iter()
                .any(|n| n.value == 10 && n.children.is_empty())
        );
    }

    #[test]
    fn test_tuple_coordinated_shrinking_strategy() {
        let original = (50i32, true);
        let shrunk: Vec<(i32, bool)> = strategies::tuple_shrink(&original).collect();

        assert!(!shrunk.is_empty());

        // Should contain individual shrinking
        assert!(shrunk.contains(&(0, true)));
        assert!(shrunk.contains(&(50, false)));

        // Should contain coordinated shrinking
        assert!(shrunk.contains(&(0, false)));
    }

    #[test]
    fn test_nested_collection_shrinking_strategy() {
        let original = vec![vec![1, 2], vec![3, 4, 5]];
        let shrunk: Vec<Vec<Vec<i32>>> = strategies::nested_collection_shrink(&original).collect();

        assert!(!shrunk.is_empty());

        // Should try empty collection
        assert!(shrunk.contains(&vec![]));

        // Should try removing inner collections
        assert!(shrunk.contains(&vec![vec![3, 4, 5]]));
        assert!(shrunk.contains(&vec![vec![1, 2]]));

        // Should try shrinking inner collections
        assert!(shrunk.iter().any(|v| v.contains(&vec![])));
        assert!(
            shrunk
                .iter()
                .any(|v| v.iter().any(|inner| inner.contains(&0)))
        );
    }

    // Async shrinkage tests
    #[tokio::test]
    async fn test_async_shrink_engine_basic() {
        let engine = AsyncShrinkEngine::new();
        let original = 100;

        let property = |value: i32| async move {
            // Simulate some async work
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            if value > 10 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let result = engine.shrink(original, property).await;

        // Should shrink to a value > 10 (since that's what fails) but smaller than original
        assert!(
            result.minimal > 10,
            "Expected minimal > 10 (since that's what fails), got {}",
            result.minimal
        );
        assert!(
            result.minimal < original,
            "Expected minimal < original, got {} vs {}",
            result.minimal,
            original
        );
        assert!(result.shrink_steps > 0);
        assert_eq!(result.original, original);
        assert!(result.completed);
    }

    #[tokio::test]
    async fn test_async_shrink_engine_no_shrinking_needed() {
        let engine = AsyncShrinkEngine::new();
        let original = 5;

        let property = |value: i32| async move {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            if value > 10 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let result = engine.shrink(original, property).await;

        // No shrinking should be needed since original value passes
        assert_eq!(result.minimal, original);
        assert_eq!(result.shrink_steps, 0);
        assert!(result.completed);
    }

    #[tokio::test]
    async fn test_async_shrink_engine_with_timeout() {
        let config = ShrinkConfig {
            max_iterations: 1000,
            timeout: Duration::from_millis(10), // Very short timeout
            verbose: false,
        };
        let engine = AsyncShrinkEngine::with_config(config);
        let original = 1000;

        let property = |value: i32| async move {
            // Simulate slow async work
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            if value > 0 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let result = engine.shrink(original, property).await;

        // Should timeout before completing all shrinking
        assert_eq!(result.original, original);
        // May or may not have completed due to timeout
        assert!(result.shrink_duration.as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_async_shrink_engine_with_custom_timeout() {
        let engine = AsyncShrinkEngine::new();
        let original = 50;
        let custom_timeout = Duration::from_millis(20);

        let property = |value: i32| async move {
            // Simulate some async work
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
            if value > 10 {
                Err(PropertyError::PropertyFailed {
                    message: "Value too large".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        let result = engine
            .shrink_with_timeout(original, property, custom_timeout)
            .await;

        // Should respect the custom timeout
        // Use more lenient tolerance for CI environments (especially macOS nightly)
        // which may have higher scheduling overhead
        assert_eq!(result.original, original);
        assert!(
            result.shrink_duration <= custom_timeout + Duration::from_millis(50),
            "Shrink duration {:?} exceeded timeout {:?} + 50ms tolerance",
            result.shrink_duration,
            custom_timeout
        );
    }

    #[tokio::test]
    async fn test_async_shrink_engine_with_strategy() {
        let engine = AsyncShrinkEngine::new();
        let original = vec![1, 2, 3, 4, 5];

        let property = |v: Vec<i32>| async move {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            if v.len() > 2 {
                Err(PropertyError::PropertyFailed {
                    message: "Vector too long".to_string(),
                    context: None,
                    iteration: None,
                })
            } else {
                Ok(())
            }
        };

        // Custom strategy that removes elements from the end
        let strategy = |v: &Vec<i32>| -> Box<dyn Iterator<Item = Vec<i32>>> {
            if v.is_empty() {
                Box::new(std::iter::empty())
            } else {
                let mut smaller = v.clone();
                smaller.pop();
                Box::new(std::iter::once(smaller))
            }
        };

        let result = engine
            .shrink_with_strategy(original.clone(), property, strategy)
            .await;

        // Should shrink to a vector of length > 2 (since that's what fails) but smaller than original
        assert!(
            result.minimal.len() > 2,
            "Expected minimal length > 2 (since that's what fails), got {}",
            result.minimal.len()
        );
        assert!(
            result.minimal.len() < original.len(),
            "Expected minimal length < original length, got {} vs {}",
            result.minimal.len(),
            original.len()
        );
        assert!(result.shrink_steps > 0);
        assert_eq!(result.original, original);
    }
}
