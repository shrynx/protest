use protest_stateful::prelude::*;
use std::collections::HashMap;

// Edge case: State with complex nested structures
#[derive(Debug, Clone, PartialEq)]
struct NestedState {
    data: HashMap<String, Vec<i32>>,
    metadata: Option<String>,
}

#[derive(Debug, Clone)]
enum NestedOp {
    AddKey(String),
    AddValue(String, i32),
    #[allow(dead_code)]
    RemoveKey(String),
    #[allow(dead_code)]
    SetMetadata(String),
    #[allow(dead_code)]
    ClearMetadata,
}

impl Operation for NestedOp {
    type State = NestedState;

    fn execute(&self, state: &mut Self::State) {
        match self {
            NestedOp::AddKey(key) => {
                state.data.insert(key.clone(), Vec::new());
            }
            NestedOp::AddValue(key, value) => {
                if let Some(vec) = state.data.get_mut(key) {
                    vec.push(*value);
                }
            }
            NestedOp::RemoveKey(key) => {
                state.data.remove(key);
            }
            NestedOp::SetMetadata(meta) => {
                state.metadata = Some(meta.clone());
            }
            NestedOp::ClearMetadata => {
                state.metadata = None;
            }
        }
    }

    fn precondition(&self, state: &Self::State) -> bool {
        match self {
            NestedOp::AddValue(key, _) => state.data.contains_key(key),
            NestedOp::RemoveKey(key) => state.data.contains_key(key),
            _ => true,
        }
    }
}

#[test]
fn test_nested_state_operations() {
    let test = StatefulTest::new(NestedState {
        data: HashMap::new(),
        metadata: None,
    })
    .invariant("data_not_too_large", |s: &NestedState| s.data.len() < 100);

    let mut seq = OperationSequence::new();
    seq.push(NestedOp::AddKey("key1".to_string()));
    seq.push(NestedOp::AddValue("key1".to_string(), 42));
    seq.push(NestedOp::SetMetadata("test".to_string()));

    let result = test.run(&seq);
    assert!(result.is_ok());
}

#[test]
fn test_precondition_blocks_invalid_operation() {
    let test = StatefulTest::new(NestedState {
        data: HashMap::new(),
        metadata: None,
    });

    let mut seq = OperationSequence::new();
    // Try to add value to non-existent key
    seq.push(NestedOp::AddValue("missing".to_string(), 42));

    let result = test.run(&seq);
    assert!(result.is_err());
}

// Edge case: Zero-sized state
#[derive(Debug, Clone)]
struct EmptyState;

#[derive(Debug, Clone)]
enum EmptyOp {
    DoNothing,
}

impl Operation for EmptyOp {
    type State = EmptyState;

    fn execute(&self, _state: &mut Self::State) {
        // Intentionally does nothing
    }
}

#[test]
fn test_zero_sized_state() {
    let test = StatefulTest::new(EmptyState);

    let mut seq = OperationSequence::new();
    seq.push(EmptyOp::DoNothing);
    seq.push(EmptyOp::DoNothing);

    let result = test.run(&seq);
    assert!(result.is_ok());
}

// Edge case: State with large values
#[derive(Debug, Clone)]
struct LargeState {
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
enum LargeOp {
    Append(Vec<u8>),
    Clear,
}

impl Operation for LargeOp {
    type State = LargeState;

    fn execute(&self, state: &mut Self::State) {
        match self {
            LargeOp::Append(bytes) => state.data.extend(bytes),
            LargeOp::Clear => state.data.clear(),
        }
    }
}

#[test]
fn test_large_state_handling() {
    let test = StatefulTest::new(LargeState { data: Vec::new() })
        .invariant("size_reasonable", |s: &LargeState| s.data.len() < 10000);

    let mut seq = OperationSequence::new();
    seq.push(LargeOp::Append(vec![1, 2, 3]));
    seq.push(LargeOp::Append(vec![4, 5, 6]));
    seq.push(LargeOp::Clear);
    seq.push(LargeOp::Append(vec![7, 8, 9]));

    let result = test.run(&seq);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, vec![7, 8, 9]);
}

// Edge case: Cyclic state dependencies
#[derive(Debug, Clone)]
struct CyclicState {
    a: i32,
    b: i32,
}

#[derive(Debug, Clone)]
enum CyclicOp {
    SwapAB,
    IncrementA,
    DecrementB,
}

impl Operation for CyclicOp {
    type State = CyclicState;

    fn execute(&self, state: &mut Self::State) {
        match self {
            CyclicOp::SwapAB => {
                std::mem::swap(&mut state.a, &mut state.b);
            }
            CyclicOp::IncrementA => state.a += 1,
            CyclicOp::DecrementB => state.b -= 1,
        }
    }
}

#[test]
fn test_cyclic_dependencies() {
    let test = StatefulTest::new(CyclicState { a: 10, b: 20 })
        .invariant("both_positive", |s: &CyclicState| s.a > 0 && s.b > 0);

    let mut seq = OperationSequence::new();
    seq.push(CyclicOp::SwapAB); // a=20, b=10
    seq.push(CyclicOp::IncrementA); // a=21, b=10
    seq.push(CyclicOp::DecrementB); // a=21, b=9

    let result = test.run(&seq);
    assert!(result.is_ok());
    let final_state = result.unwrap();
    assert_eq!(final_state.a, 21);
    assert_eq!(final_state.b, 9);
}

// Edge case: Operations that can fail
#[derive(Debug, Clone)]
struct FallibleState {
    value: i32,
    error_count: usize,
}

#[derive(Debug, Clone)]
enum FallibleOp {
    TryDivide(i32),
    #[allow(dead_code)]
    Reset,
}

impl Operation for FallibleOp {
    type State = FallibleState;

    fn execute(&self, state: &mut Self::State) {
        match self {
            FallibleOp::TryDivide(divisor) => {
                if *divisor == 0 {
                    state.error_count += 1;
                } else {
                    state.value /= divisor;
                }
            }
            FallibleOp::Reset => {
                state.value = 0;
                state.error_count = 0;
            }
        }
    }

    fn precondition(&self, _state: &Self::State) -> bool {
        // Always allowed, but may increment error_count
        true
    }
}

#[test]
fn test_fallible_operations() {
    let test = StatefulTest::new(FallibleState {
        value: 100,
        error_count: 0,
    })
    .invariant("errors_tracked", |s: &FallibleState| s.error_count < 10);

    let mut seq = OperationSequence::new();
    seq.push(FallibleOp::TryDivide(2)); // value = 50
    seq.push(FallibleOp::TryDivide(0)); // error, value = 50
    seq.push(FallibleOp::TryDivide(5)); // value = 10

    let result = test.run(&seq);
    assert!(result.is_ok());
    let final_state = result.unwrap();
    assert_eq!(final_state.value, 10);
    assert_eq!(final_state.error_count, 1);
}

// Edge case: Extremely long sequences
#[test]
fn test_very_long_sequence() {
    #[derive(Debug, Clone)]
    struct Counter {
        count: i32,
    }

    #[derive(Debug, Clone)]
    enum CountOp {
        Inc,
    }

    impl Operation for CountOp {
        type State = Counter;
        fn execute(&self, state: &mut Self::State) {
            state.count += 1;
        }
    }

    let test = StatefulTest::new(Counter { count: 0 });

    let mut seq = OperationSequence::new();
    for _ in 0..1000 {
        seq.push(CountOp::Inc);
    }

    let result = test.run(&seq);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().count, 1000);
}

// Edge case: Shrinking maintains validity
#[test]
fn test_shrinking_preserves_structure() {
    let mut seq = OperationSequence::new();
    seq.push(CyclicOp::IncrementA);
    seq.push(CyclicOp::DecrementB);
    seq.push(CyclicOp::SwapAB);
    seq.push(CyclicOp::IncrementA);
    seq.push(CyclicOp::DecrementB);

    let shrunk = seq.shrink();

    // All shrunk sequences should be valid
    for s in shrunk {
        assert!(!s.is_empty());
        assert!(s.len() < seq.len());
        // Could run them through the test to verify they're still valid
    }
}

// Edge case: Model with different internal representation
#[derive(Debug, Clone)]
struct CompressedModel {
    compressed: String, // Simulates compression
}

#[derive(Debug, Clone)]
struct UncompressedSystem {
    data: Vec<String>,
}

#[derive(Debug, Clone)]
enum DataOp {
    Add(String),
    #[allow(dead_code)]
    Clear,
}

impl Operation for DataOp {
    type State = UncompressedSystem;

    fn execute(&self, state: &mut Self::State) {
        match self {
            DataOp::Add(s) => state.data.push(s.clone()),
            DataOp::Clear => state.data.clear(),
        }
    }
}

impl Model for CompressedModel {
    type SystemState = UncompressedSystem;
    type Operation = DataOp;

    fn execute_model(&mut self, op: &Self::Operation) {
        match op {
            DataOp::Add(s) => {
                if !self.compressed.is_empty() {
                    self.compressed.push(',');
                }
                self.compressed.push_str(s);
            }
            DataOp::Clear => self.compressed.clear(),
        }
    }

    fn matches(&self, system: &Self::SystemState) -> bool {
        let system_compressed = system.data.join(",");
        self.compressed == system_compressed
    }
}

#[test]
fn test_model_with_different_representation() {
    let model = CompressedModel {
        compressed: String::new(),
    };
    let system = UncompressedSystem { data: Vec::new() };

    let test = ModelBasedTest::new(model, system);

    let mut seq = OperationSequence::new();
    seq.push(DataOp::Add("a".to_string()));
    seq.push(DataOp::Add("b".to_string()));
    seq.push(DataOp::Add("c".to_string()));

    let result = test.run(&seq);
    assert!(result.is_ok());
}

#[test]
fn test_model_trace_inspection() {
    let model = CompressedModel {
        compressed: String::new(),
    };
    let system = UncompressedSystem { data: Vec::new() };

    let test = ModelBasedTest::new(model, system);

    let mut seq = OperationSequence::new();
    seq.push(DataOp::Add("x".to_string()));
    seq.push(DataOp::Add("y".to_string()));

    let trace = test.run_with_trace(&seq).unwrap();
    assert_eq!(trace.steps().len(), 2);
}
