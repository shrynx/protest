//! Invariant checking for stateful properties

use std::fmt::Debug;

/// An invariant that must hold for a state
pub trait Invariant<State> {
    /// Check if the invariant holds for the given state
    fn check(&self, state: &State) -> bool;

    /// Get a description of this invariant
    fn description(&self) -> &str;
}

/// A simple function-based invariant
pub struct FnInvariant<State, F>
where
    F: Fn(&State) -> bool,
{
    name: String,
    check_fn: F,
    _phantom: std::marker::PhantomData<State>,
}

impl<State, F> FnInvariant<State, F>
where
    F: Fn(&State) -> bool,
{
    /// Create a new function-based invariant
    pub fn new(name: impl Into<String>, check_fn: F) -> Self {
        Self {
            name: name.into(),
            check_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State, F> Invariant<State> for FnInvariant<State, F>
where
    F: Fn(&State) -> bool,
{
    fn check(&self, state: &State) -> bool {
        (self.check_fn)(state)
    }

    fn description(&self) -> &str {
        &self.name
    }
}

/// A collection of invariants to check
pub struct InvariantSet<State: 'static> {
    invariants: Vec<Box<dyn Invariant<State>>>,
}

impl<State> InvariantSet<State> {
    /// Create a new empty invariant set
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
        }
    }

    /// Add an invariant
    pub fn add<I: Invariant<State> + 'static>(&mut self, invariant: I) {
        self.invariants.push(Box::new(invariant));
    }

    /// Add a function-based invariant
    pub fn add_fn<F>(&mut self, name: impl Into<String>, check_fn: F)
    where
        F: Fn(&State) -> bool + 'static,
    {
        self.add(FnInvariant::new(name, check_fn));
    }

    /// Check all invariants
    pub fn check_all(&self, state: &State) -> Result<(), InvariantViolation> {
        for inv in &self.invariants {
            if !inv.check(state) {
                return Err(InvariantViolation {
                    description: inv.description().to_string(),
                });
            }
        }
        Ok(())
    }

    /// Get the number of invariants
    pub fn len(&self) -> usize {
        self.invariants.len()
    }

    /// Check if there are no invariants
    pub fn is_empty(&self) -> bool {
        self.invariants.is_empty()
    }
}

impl<State> Default for InvariantSet<State> {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a violation of an invariant
#[derive(Debug, Clone)]
pub struct InvariantViolation {
    pub description: String,
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invariant violated: {}", self.description)
    }
}

impl std::error::Error for InvariantViolation {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counter {
        value: i32,
    }

    #[test]
    fn test_fn_invariant() {
        let inv = FnInvariant::new("non_negative", |state: &Counter| state.value >= 0);

        let state1 = Counter { value: 5 };
        assert!(inv.check(&state1));

        let state2 = Counter { value: -1 };
        assert!(!inv.check(&state2));
    }

    #[test]
    fn test_invariant_set() {
        let mut set = InvariantSet::new();
        set.add_fn("non_negative", |state: &Counter| state.value >= 0);
        set.add_fn("less_than_100", |state: &Counter| state.value < 100);

        let state1 = Counter { value: 50 };
        assert!(set.check_all(&state1).is_ok());

        let state2 = Counter { value: -5 };
        assert!(set.check_all(&state2).is_err());

        let state3 = Counter { value: 150 };
        assert!(set.check_all(&state3).is_err());
    }
}
