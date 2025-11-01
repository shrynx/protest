use protest_stateful::prelude::*;

// Simple counter for testing
#[derive(Debug, Clone)]
struct Counter {
    value: i32,
}

#[derive(Debug, Clone)]
enum CounterOp {
    Increment,
    Decrement,
    Reset,
}

impl Operation for CounterOp {
    type State = Counter;

    fn execute(&self, state: &mut Self::State) {
        match self {
            CounterOp::Increment => state.value += 1,
            CounterOp::Decrement => state.value -= 1,
            CounterOp::Reset => state.value = 0,
        }
    }

    fn precondition(&self, state: &Self::State) -> bool {
        match self {
            CounterOp::Decrement => state.value > 0,
            _ => true,
        }
    }
}

#[test]
fn test_stateful_basic() {
    let test = StatefulTest::new(Counter { value: 0 })
        .invariant("non_negative", |s: &Counter| s.value >= 0);

    let mut seq = OperationSequence::new();
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Decrement);

    let result = test.run(&seq);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, 1);
}

#[test]
fn test_stateful_invariant_violation() {
    let test = StatefulTest::new(Counter { value: 0 })
        .invariant("value_less_than_2", |s: &Counter| s.value < 2);

    let mut seq = OperationSequence::new();
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment); // Should violate

    let result = test.run(&seq);
    assert!(result.is_err());
}

#[test]
fn test_operation_sequence_shrinking() {
    let mut seq = OperationSequence::new();
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Decrement);
    seq.push(CounterOp::Reset);

    let shrunk = seq.shrink();
    assert!(!shrunk.is_empty());

    for s in shrunk {
        assert!(s.len() < seq.len());
    }
}

#[test]
fn test_execution_trace() {
    let test = StatefulTest::new(Counter { value: 0 });

    let mut seq = OperationSequence::new();
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment);

    let trace = test.run_with_trace(&seq).unwrap();
    assert_eq!(trace.steps().len(), 3);
    assert_eq!(trace.final_state().unwrap().value, 3);
}

// Model-based testing
#[derive(Debug, Clone)]
struct CounterModel {
    value: i32,
}

impl Model for CounterModel {
    type SystemState = Counter;
    type Operation = CounterOp;

    fn execute_model(&mut self, op: &Self::Operation) {
        match op {
            CounterOp::Increment => self.value += 1,
            CounterOp::Decrement => self.value -= 1,
            CounterOp::Reset => self.value = 0,
        }
    }

    fn matches(&self, system: &Self::SystemState) -> bool {
        self.value == system.value
    }
}

#[test]
fn test_model_based() {
    let model = CounterModel { value: 0 };
    let system = Counter { value: 0 };
    let test = ModelBasedTest::new(model, system);

    let mut seq = OperationSequence::new();
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Increment);
    seq.push(CounterOp::Decrement);

    let result = test.run(&seq);
    assert!(result.is_ok());
}

// Temporal properties
use protest_stateful::temporal::*;

#[test]
fn test_temporal_eventually() {
    let states = vec![
        Counter { value: 0 },
        Counter { value: 1 },
        Counter { value: 5 },
        Counter { value: 10 },
    ];

    let prop = Eventually::new("reaches_10", |s: &Counter| s.value == 10);
    assert!(prop.check(&states));

    let prop2 = Eventually::new("reaches_20", |s: &Counter| s.value == 20);
    assert!(!prop2.check(&states));
}

#[test]
fn test_temporal_always() {
    let states = vec![
        Counter { value: 0 },
        Counter { value: 1 },
        Counter { value: 5 },
        Counter { value: 10 },
    ];

    let prop = Always::new("non_negative", |s: &Counter| s.value >= 0);
    assert!(prop.check(&states));

    let states2 = vec![
        Counter { value: 0 },
        Counter { value: -1 },
        Counter { value: 5 },
    ];

    assert!(!prop.check(&states2));
}

#[test]
fn test_temporal_never() {
    let states = vec![
        Counter { value: 0 },
        Counter { value: 1 },
        Counter { value: 5 },
    ];

    let prop = Never::new("never_negative", |s: &Counter| s.value < 0);
    assert!(prop.check(&states));

    let states2 = vec![
        Counter { value: 0 },
        Counter { value: -1 },
        Counter { value: 5 },
    ];

    assert!(!prop.check(&states2));
}

#[test]
fn test_temporal_leads_to() {
    let states = vec![
        Counter { value: 0 },
        Counter { value: 5 },
        Counter { value: 10 },
    ];

    let prop = LeadsTo::new(
        "5_leads_to_10",
        |s: &Counter| s.value == 5,
        |s: &Counter| s.value == 10,
    );

    assert!(prop.check(&states));
}
