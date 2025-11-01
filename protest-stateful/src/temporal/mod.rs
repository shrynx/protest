//! Temporal properties for stateful testing
//!
//! Express properties like "eventually" and "always" over execution traces

/// A temporal property that can be checked over an execution trace
pub trait TemporalProperty<State> {
    /// Check if the property holds for the given trace
    fn check(&self, trace: &[State]) -> bool;

    /// Description of the property
    fn description(&self) -> &str;
}

/// "Eventually P" - P must hold at some point in the trace
pub struct Eventually<State, F>
where
    F: Fn(&State) -> bool,
{
    name: String,
    predicate: F,
    _phantom: std::marker::PhantomData<State>,
}

impl<State, F> Eventually<State, F>
where
    F: Fn(&State) -> bool,
{
    /// Create a new "eventually" property
    pub fn new(name: impl Into<String>, predicate: F) -> Self {
        Self {
            name: name.into(),
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State, F> TemporalProperty<State> for Eventually<State, F>
where
    F: Fn(&State) -> bool,
{
    fn check(&self, trace: &[State]) -> bool {
        trace.iter().any(|state| (self.predicate)(state))
    }

    fn description(&self) -> &str {
        &self.name
    }
}

/// "Always P" - P must hold at every point in the trace
pub struct Always<State, F>
where
    F: Fn(&State) -> bool,
{
    name: String,
    predicate: F,
    _phantom: std::marker::PhantomData<State>,
}

impl<State, F> Always<State, F>
where
    F: Fn(&State) -> bool,
{
    /// Create a new "always" property
    pub fn new(name: impl Into<String>, predicate: F) -> Self {
        Self {
            name: name.into(),
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State, F> TemporalProperty<State> for Always<State, F>
where
    F: Fn(&State) -> bool,
{
    fn check(&self, trace: &[State]) -> bool {
        trace.iter().all(|state| (self.predicate)(state))
    }

    fn description(&self) -> &str {
        &self.name
    }
}

/// "Never P" - P must not hold at any point in the trace
pub struct Never<State, F>
where
    F: Fn(&State) -> bool,
{
    name: String,
    predicate: F,
    _phantom: std::marker::PhantomData<State>,
}

impl<State, F> Never<State, F>
where
    F: Fn(&State) -> bool,
{
    /// Create a new "never" property
    pub fn new(name: impl Into<String>, predicate: F) -> Self {
        Self {
            name: name.into(),
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State, F> TemporalProperty<State> for Never<State, F>
where
    F: Fn(&State) -> bool,
{
    fn check(&self, trace: &[State]) -> bool {
        !trace.iter().any(|state| (self.predicate)(state))
    }

    fn description(&self) -> &str {
        &self.name
    }
}

/// "P leads to Q" - If P holds, then Q must eventually hold later
pub struct LeadsTo<State, F1, F2>
where
    F1: Fn(&State) -> bool,
    F2: Fn(&State) -> bool,
{
    name: String,
    p_predicate: F1,
    q_predicate: F2,
    _phantom: std::marker::PhantomData<State>,
}

impl<State, F1, F2> LeadsTo<State, F1, F2>
where
    F1: Fn(&State) -> bool,
    F2: Fn(&State) -> bool,
{
    /// Create a new "leads to" property
    pub fn new(name: impl Into<String>, p_predicate: F1, q_predicate: F2) -> Self {
        Self {
            name: name.into(),
            p_predicate,
            q_predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State, F1, F2> TemporalProperty<State> for LeadsTo<State, F1, F2>
where
    F1: Fn(&State) -> bool,
    F2: Fn(&State) -> bool,
{
    fn check(&self, trace: &[State]) -> bool {
        for (i, state) in trace.iter().enumerate() {
            if (self.p_predicate)(state) {
                // P holds at position i, check if Q holds at some point after i
                if !trace[i..].iter().any(|s| (self.q_predicate)(s)) {
                    return false; // Q never holds after P
                }
            }
        }
        true
    }

    fn description(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct State {
        value: i32,
        flag: bool,
    }

    #[test]
    fn test_eventually() {
        let prop = Eventually::new("value reaches 10", |s: &State| s.value == 10);

        let trace = vec![
            State {
                value: 0,
                flag: false,
            },
            State {
                value: 5,
                flag: false,
            },
            State {
                value: 10,
                flag: true,
            },
        ];

        assert!(prop.check(&trace));

        let trace2 = vec![
            State {
                value: 0,
                flag: false,
            },
            State {
                value: 5,
                flag: false,
            },
        ];

        assert!(!prop.check(&trace2));
    }

    #[test]
    fn test_always() {
        let prop = Always::new("value non-negative", |s: &State| s.value >= 0);

        let trace = vec![
            State {
                value: 0,
                flag: false,
            },
            State {
                value: 5,
                flag: false,
            },
            State {
                value: 10,
                flag: true,
            },
        ];

        assert!(prop.check(&trace));

        let trace2 = vec![
            State {
                value: 0,
                flag: false,
            },
            State {
                value: -1,
                flag: false,
            },
        ];

        assert!(!prop.check(&trace2));
    }

    #[test]
    fn test_leads_to() {
        let prop = LeadsTo::new(
            "flag true leads to value 10",
            |s: &State| s.flag,
            |s: &State| s.value == 10,
        );

        let trace = vec![
            State {
                value: 0,
                flag: false,
            },
            State {
                value: 5,
                flag: true,
            },
            State {
                value: 10,
                flag: true,
            },
        ];

        assert!(prop.check(&trace));

        let trace2 = vec![
            State {
                value: 0,
                flag: false,
            },
            State {
                value: 5,
                flag: true,
            },
            State {
                value: 5,
                flag: false,
            },
        ];

        assert!(!prop.check(&trace2));
    }
}
