//! Integration tests for derive macros

use protest_stateful::{
    Operation as OperationDerive,
    dsl::StatefulTest,
    operations::{Operation, OperationSequence},
};

// Test basic enum with unit and unnamed variants
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "i32")]
enum CounterOp {
    #[execute("*state += 1")]
    Increment,

    #[execute("*state -= 1")]
    #[precondition("*state > 0")]
    Decrement,

    #[execute("*state += *field_0")]
    Add(i32),

    #[execute("*state = 0")]
    Reset,
}

#[test]
fn test_derive_counter_operations() {
    let mut state = 5i32;

    // Test increment
    CounterOp::Increment.execute(&mut state);
    assert_eq!(state, 6);

    // Test add
    CounterOp::Add(10).execute(&mut state);
    assert_eq!(state, 16);

    // Test decrement
    assert!(CounterOp::Decrement.precondition(&state));
    CounterOp::Decrement.execute(&mut state);
    assert_eq!(state, 15);

    // Test reset
    CounterOp::Reset.execute(&mut state);
    assert_eq!(state, 0);

    // Test precondition failure
    assert!(!CounterOp::Decrement.precondition(&state));
}

// Test enum with named fields
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "Vec<i32>")]
enum StackOp {
    #[execute("state.push(*field_0)")]
    Push(i32),

    #[execute("state.pop()")]
    #[precondition("!state.is_empty()")]
    Pop,

    #[execute("state.clear()")]
    Clear,
}

#[test]
fn test_derive_stack_operations() {
    let mut state = vec![];

    // Test push
    StackOp::Push(10).execute(&mut state);
    assert_eq!(state, vec![10]);

    StackOp::Push(20).execute(&mut state);
    assert_eq!(state, vec![10, 20]);

    // Test pop precondition
    assert!(StackOp::Pop.precondition(&state));
    StackOp::Pop.execute(&mut state);
    assert_eq!(state, vec![10]);

    // Test clear
    StackOp::Clear.execute(&mut state);
    assert_eq!(state, Vec::<i32>::new());

    // Test pop precondition failure
    assert!(!StackOp::Pop.precondition(&state));
}

// Test with HashMap and named fields
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "std::collections::HashMap<String, i32>")]
enum MapOp {
    #[execute("state.insert(key.clone(), *value)")]
    Insert { key: String, value: i32 },

    #[execute("state.remove(key)")]
    #[precondition("state.contains_key(key)")]
    Remove { key: String },

    #[execute("state.clear()")]
    #[allow(dead_code)]
    Clear,
}

#[test]
fn test_derive_map_operations() {
    use std::collections::HashMap;
    let mut state = HashMap::new();

    // Test insert
    MapOp::Insert {
        key: "foo".to_string(),
        value: 42,
    }
    .execute(&mut state);
    assert_eq!(state.get("foo"), Some(&42));

    // Test remove precondition
    assert!(
        MapOp::Remove {
            key: "foo".to_string()
        }
        .precondition(&state)
    );

    MapOp::Remove {
        key: "foo".to_string(),
    }
    .execute(&mut state);
    assert_eq!(state.get("foo"), None);

    // Test remove precondition failure
    assert!(
        !MapOp::Remove {
            key: "bar".to_string()
        }
        .precondition(&state)
    );
}

// Test integration with StatefulTest
#[test]
fn test_derive_with_stateful_test() {
    let initial_state = 0i32;

    let test = StatefulTest::new(initial_state)
        .invariant("non_negative", |state: &i32| *state >= 0)
        .invariant("bounded", |state: &i32| *state <= 100);

    let mut sequence = OperationSequence::new();
    sequence.push(CounterOp::Increment);
    sequence.push(CounterOp::Increment);
    sequence.push(CounterOp::Add(5));
    sequence.push(CounterOp::Decrement);

    let result = test.run(&sequence);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 6);
}

// Test custom descriptions
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "Vec<String>")]
enum ListOp {
    #[execute("state.push(field_0.clone())")]
    #[description("Add item to list")]
    Add(String),

    #[execute("state.pop()")]
    #[precondition("!state.is_empty()")]
    #[description("Remove last item")]
    Remove,
}

#[test]
fn test_custom_descriptions() {
    let add_op = ListOp::Add("test".to_string());
    // Custom descriptions are used when provided
    assert_eq!(add_op.description(), "Add item to list");

    let remove_op = ListOp::Remove;
    // Custom descriptions are used when provided
    assert_eq!(remove_op.description(), "Remove last item");
}
