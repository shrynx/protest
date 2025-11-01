//! Example: Model-based testing of a key-value store

use protest_stateful::prelude::*;
use std::collections::HashMap;

/// A simple key-value store
#[derive(Debug, Clone)]
struct KeyValueStore {
    data: HashMap<String, String>,
}

impl KeyValueStore {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    #[allow(dead_code)]
    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    fn delete(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    #[allow(dead_code)]
    fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Operations on the key-value store
#[derive(Debug, Clone)]
enum KVOp {
    Set(String, String),
    Get(String),
    Delete(String),
}

impl Operation for KVOp {
    type State = KeyValueStore;

    fn execute(&self, state: &mut Self::State) {
        match self {
            KVOp::Set(k, v) => state.set(k.clone(), v.clone()),
            KVOp::Get(_k) => {
                // Read-only operation
            }
            KVOp::Delete(k) => {
                state.delete(k);
            }
        }
    }

    fn description(&self) -> String {
        match self {
            KVOp::Set(k, v) => format!("Set({}, {})", k, v),
            KVOp::Get(k) => format!("Get({})", k),
            KVOp::Delete(k) => format!("Delete({})", k),
        }
    }
}

/// Reference model (simple HashMap)
#[derive(Debug, Clone)]
struct KVModel {
    data: HashMap<String, String>,
}

impl Model for KVModel {
    type SystemState = KeyValueStore;
    type Operation = KVOp;

    fn execute_model(&mut self, op: &Self::Operation) {
        match op {
            KVOp::Set(k, v) => {
                self.data.insert(k.clone(), v.clone());
            }
            KVOp::Get(_) => {
                // Read-only
            }
            KVOp::Delete(k) => {
                self.data.remove(k);
            }
        }
    }

    fn matches(&self, system_state: &Self::SystemState) -> bool {
        self.data == system_state.data
    }

    fn mismatch_description(&self, system_state: &Self::SystemState) -> Option<String> {
        if self.matches(system_state) {
            None
        } else {
            Some(format!(
                "Model has {} entries, system has {} entries",
                self.data.len(),
                system_state.data.len()
            ))
        }
    }
}

fn main() {
    println!("Model-Based Testing of Key-Value Store\n");

    let model = KVModel {
        data: HashMap::new(),
    };
    let system = KeyValueStore::new();

    let test = ModelBasedTest::new(model, system);

    // Test 1: Basic operations
    println!("Test 1: Basic operations");
    let mut seq1 = OperationSequence::new();
    seq1.push(KVOp::Set("name".to_string(), "Alice".to_string()));
    seq1.push(KVOp::Set("age".to_string(), "30".to_string()));
    seq1.push(KVOp::Get("name".to_string()));
    seq1.push(KVOp::Delete("age".to_string()));

    match test.run(&seq1) {
        Ok(_) => println!("  ✓ Model and system match!\n"),
        Err(e) => println!("  ✗ Mismatch: {}\n", e),
    }

    // Test 2: Multiple sets and deletes
    println!("Test 2: Multiple operations");
    let mut seq2 = OperationSequence::new();
    for i in 0..5 {
        seq2.push(KVOp::Set(format!("key{}", i), format!("value{}", i)));
    }
    seq2.push(KVOp::Delete("key2".to_string()));
    seq2.push(KVOp::Delete("key4".to_string()));
    seq2.push(KVOp::Set("key2".to_string(), "new_value".to_string()));

    match test.run_with_trace(&seq2) {
        Ok(trace) => {
            println!("  ✓ Model and system match through all steps!");
            println!("  Total steps: {}\n", trace.steps().len());
        }
        Err(e) => println!("  ✗ Mismatch: {}\n", e),
    }

    // Test 3: Overwriting values
    println!("Test 3: Overwriting values");
    let mut seq3 = OperationSequence::new();
    seq3.push(KVOp::Set("counter".to_string(), "0".to_string()));
    seq3.push(KVOp::Set("counter".to_string(), "1".to_string()));
    seq3.push(KVOp::Set("counter".to_string(), "2".to_string()));
    seq3.push(KVOp::Get("counter".to_string()));

    match test.run(&seq3) {
        Ok(_) => println!("  ✓ Model and system match!\n"),
        Err(e) => println!("  ✗ Mismatch: {}\n", e),
    }

    println!("All model-based tests completed!");
}
