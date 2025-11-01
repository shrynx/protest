//! Example demonstrating the #[derive(Operation)] macro
//!
//! This example shows how to use the derive macro to automatically implement
//! the Operation trait for various operation patterns.

use protest_stateful::{
    Operation as OperationDerive,
    dsl::StatefulTest,
    operations::{Operation, OperationSequence},
};

// Example 1: Simple Stack Operations with Derive
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "Vec<i32>")]
enum StackOp {
    #[execute("state.push(*field_0)")]
    #[weight(5)]
    Push(i32),

    #[execute("state.pop()")]
    #[precondition("!state.is_empty()")]
    #[weight(3)]
    Pop,

    #[execute("state.clear()")]
    #[weight(1)]
    #[allow(dead_code)]
    Clear,
}

// Example 2: HashMap Operations with Named Fields
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "std::collections::HashMap<String, i32>")]
enum MapOp {
    #[execute("state.insert(key.clone(), *value)")]
    #[weight(5)]
    #[description("Insert key-value pair")]
    Insert { key: String, value: i32 },

    #[execute("state.remove(key)")]
    #[precondition("state.contains_key(key)")]
    #[weight(3)]
    #[description("Remove key")]
    Remove { key: String },

    #[execute("let _ = state.get(key).copied()")]
    #[precondition("state.contains_key(key)")]
    #[weight(4)]
    Get { key: String },

    #[execute("state.clear()")]
    #[weight(1)]
    #[allow(dead_code)]
    Clear,
}

// Example 3: Counter with Unit and Tuple Variants
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "i32")]
enum CounterOp {
    #[execute("*state += 1")]
    #[weight(5)]
    Increment,

    #[execute("*state -= 1")]
    #[precondition("*state > 0")]
    #[weight(3)]
    Decrement,

    #[execute("*state += *field_0")]
    #[weight(2)]
    Add(i32),

    #[execute("*state = 0")]
    #[weight(1)]
    #[allow(dead_code)]
    Reset,
}

// Example 4: Queue Operations
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "std::collections::VecDeque<String>")]
enum QueueOp {
    #[execute("state.push_back(item.clone())")]
    #[weight(5)]
    Enqueue { item: String },

    #[execute("state.pop_front()")]
    #[precondition("!state.is_empty()")]
    #[weight(4)]
    Dequeue,

    #[execute("let _ = state.front().cloned()")]
    #[precondition("!state.is_empty()")]
    #[weight(3)]
    Peek,

    #[execute("state.clear()")]
    #[weight(1)]
    #[allow(dead_code)]
    Clear,
}

fn main() {
    println!("=== Example 1: Stack Operations ===\n");
    example_stack();

    println!("\n=== Example 2: HashMap Operations ===\n");
    example_hashmap();

    println!("\n=== Example 3: Counter Operations ===\n");
    example_counter();

    println!("\n=== Example 4: Queue Operations ===\n");
    example_queue();
}

fn example_stack() {
    let initial_state: Vec<i32> = vec![];

    let test = StatefulTest::new(initial_state.clone())
        .invariant("size_bounded", |state: &Vec<i32>| state.len() <= 100);

    let mut sequence = OperationSequence::new();
    sequence.push(StackOp::Push(10));
    sequence.push(StackOp::Push(20));
    sequence.push(StackOp::Push(30));
    sequence.push(StackOp::Pop);
    sequence.push(StackOp::Push(40));

    println!("Operations:");
    for (i, op) in sequence.operations().iter().enumerate() {
        println!("  {}: {}", i, op.description());
    }

    match test.run(&sequence) {
        Ok(final_state) => {
            println!("\n✓ Test passed!");
            println!("  Final state: {:?}", final_state);
        }
        Err(err) => {
            println!("\n✗ Test failed: {}", err);
        }
    }
}

fn example_hashmap() {
    use std::collections::HashMap;

    let initial_state: HashMap<String, i32> = HashMap::new();

    let test = StatefulTest::new(initial_state.clone())
        .invariant("size_bounded", |state: &HashMap<String, i32>| {
            state.len() <= 100
        });

    let mut sequence = OperationSequence::new();
    sequence.push(MapOp::Insert {
        key: "foo".to_string(),
        value: 42,
    });
    sequence.push(MapOp::Insert {
        key: "bar".to_string(),
        value: 100,
    });
    sequence.push(MapOp::Get {
        key: "foo".to_string(),
    });
    sequence.push(MapOp::Remove {
        key: "foo".to_string(),
    });

    println!("Operations:");
    for (i, op) in sequence.operations().iter().enumerate() {
        println!("  {}: {}", i, op.description());
    }

    match test.run(&sequence) {
        Ok(final_state) => {
            println!("\n✓ Test passed!");
            println!("  Final state: {:?}", final_state);
        }
        Err(err) => {
            println!("\n✗ Test failed: {}", err);
        }
    }
}

fn example_counter() {
    let initial_state: i32 = 0;

    let test = StatefulTest::new(initial_state)
        .invariant("non_negative", |state: &i32| *state >= 0)
        .invariant("bounded", |state: &i32| *state <= 1000);

    let mut sequence = OperationSequence::new();
    sequence.push(CounterOp::Increment);
    sequence.push(CounterOp::Increment);
    sequence.push(CounterOp::Add(5));
    sequence.push(CounterOp::Decrement);
    sequence.push(CounterOp::Increment);

    println!("Operations:");
    for (i, op) in sequence.operations().iter().enumerate() {
        println!("  {}: {}", i, op.description());
    }

    match test.run(&sequence) {
        Ok(final_state) => {
            println!("\n✓ Test passed!");
            println!("  Final state: {}", final_state);
        }
        Err(err) => {
            println!("\n✗ Test failed: {}", err);
        }
    }
}

fn example_queue() {
    use std::collections::VecDeque;

    let initial_state: VecDeque<String> = VecDeque::new();

    let test = StatefulTest::new(initial_state.clone())
        .invariant("size_bounded", |state: &VecDeque<String>| {
            state.len() <= 100
        });

    let mut sequence = OperationSequence::new();
    sequence.push(QueueOp::Enqueue {
        item: "task1".to_string(),
    });
    sequence.push(QueueOp::Enqueue {
        item: "task2".to_string(),
    });
    sequence.push(QueueOp::Peek);
    sequence.push(QueueOp::Dequeue);
    sequence.push(QueueOp::Enqueue {
        item: "task3".to_string(),
    });

    println!("Operations:");
    for (i, op) in sequence.operations().iter().enumerate() {
        println!("  {}: {}", i, op.description());
    }

    match test.run(&sequence) {
        Ok(final_state) => {
            println!("\n✓ Test passed!");
            println!("  Final state: {:?}", final_state);
        }
        Err(err) => {
            println!("\n✗ Test failed: {}", err);
        }
    }
}
