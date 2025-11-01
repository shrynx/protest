# Protest Stateful

**Stateful property testing for Rust** - Test state machines, APIs, databases, concurrent systems, and any system that maintains state across operations.

Part of the [Protest](https://github.com/shrynx/protest) property testing ecosystem.

## Features

- 🔄 **State Machine Testing** - Define operations and invariants, automatically test sequences
- 🎯 **Model-Based Testing** - Compare real system behavior against a reference model
- ⚡ **Operation Shrinking** - Minimize failing operation sequences to find root causes
- 🔍 **Preconditions & Postconditions** - Define valid operation contexts
- ⏱️ **Temporal Properties** - Express "eventually", "always", and "leads to" properties
- 🧵 **Concurrent Testing** - Test parallel operations with race condition detection
- 📊 **Execution Traces** - Detailed step-by-step state visualization

## Installation

```toml
[dev-dependencies]
protest-stateful = "0.1"
```

## Quick Start

### Basic Stateful Testing

```rust
use protest_stateful::prelude::*;

// Define your system's state
#[derive(Debug, Clone)]
struct Stack {
    items: Vec<i32>,
}

// Define operations
#[derive(Debug, Clone)]
enum StackOp {
    Push(i32),
    Pop,
}

impl Operation for StackOp {
    type State = Stack;

    fn execute(&self, state: &mut Self::State) {
        match self {
            StackOp::Push(v) => state.items.push(*v),
            StackOp::Pop => { state.items.pop(); }
        }
    }

    fn precondition(&self, state: &Self::State) -> bool {
        match self {
            StackOp::Pop => !state.items.is_empty(),
            _ => true,
        }
    }
}

#[test]
fn test_stack_properties() {
    // Create stateful test with invariants
    let test = StatefulTest::new(Stack { items: vec![] })
        .invariant("length_non_negative", |s| s.items.len() >= 0);

    // Create operation sequence
    let mut seq = OperationSequence::new();
    seq.push(StackOp::Push(10));
    seq.push(StackOp::Push(20));
    seq.push(StackOp::Pop);

    // Run test
    let result = test.run(&seq);
    assert!(result.is_ok());
}
```

## Core Concepts

### 1. Operations

Operations define how your system changes state:

```rust
#[derive(Debug, Clone)]
enum MyOp {
    Action1,
    Action2(i32),
}

impl Operation for MyOp {
    type State = MyState;

    fn execute(&self, state: &mut Self::State) {
        // Modify state
    }

    fn precondition(&self, state: &Self::State) -> bool {
        // Return true if operation is valid in current state
        true
    }

    fn description(&self) -> String {
        format!("{:?}", self)
    }
}
```

### 2. Invariants

Invariants are properties that must hold after every operation:

```rust
let test = StatefulTest::new(initial_state)
    .invariant("name", |state| {
        // Return true if invariant holds
        state.is_valid()
    })
    .invariant("another_property", |state| {
        state.field > 0
    });
```

### 3. Operation Sequences

Test sequences of operations:

```rust
let mut seq = OperationSequence::new();
seq.push(Op1);
seq.push(Op2);
seq.push(Op3);

let result = test.run(&seq);
```

Sequences automatically shrink when failures occur to find minimal failing cases.

## Model-Based Testing

Compare your system against a reference implementation:

```rust
use protest_stateful::prelude::*;
use std::collections::HashMap;

// Your actual system
#[derive(Debug, Clone)]
struct KeyValueStore {
    data: HashMap<String, String>,
}

// Simple reference model
#[derive(Debug, Clone)]
struct KVModel {
    data: HashMap<String, String>,
}

impl Model for KVModel {
    type SystemState = KeyValueStore;
    type Operation = KVOp;

    fn execute_model(&mut self, op: &Self::Operation) {
        // Execute on model
        match op {
            KVOp::Set(k, v) => { self.data.insert(k.clone(), v.clone()); }
            KVOp::Delete(k) => { self.data.remove(k); }
            _ => {}
        }
    }

    fn matches(&self, system: &Self::SystemState) -> bool {
        self.data == system.data
    }
}

#[test]
fn test_kv_store_model() {
    let model = KVModel { data: HashMap::new() };
    let system = KeyValueStore { data: HashMap::new() };
    let test = ModelBasedTest::new(model, system);

    let mut seq = OperationSequence::new();
    seq.push(KVOp::Set("key".into(), "value".into()));
    seq.push(KVOp::Get("key".into()));

    // Automatically checks system matches model after each operation
    assert!(test.run(&seq).is_ok());
}
```

## Temporal Properties

Express properties over execution traces:

```rust
use protest_stateful::temporal::*;

let states = vec![/* execution trace */];

// "Eventually P" - property must hold at some point
let prop1 = Eventually::new("reaches_goal", |s| s.is_goal());
assert!(prop1.check(&states));

// "Always P" - property must hold at every point
let prop2 = Always::new("non_negative", |s| s.value >= 0);
assert!(prop2.check(&states));

// "Never P" - property must never hold
let prop3 = Never::new("never_invalid", |s| s.is_invalid());
assert!(prop3.check(&states));

// "P leads to Q" - if P holds, Q must eventually hold
let prop4 = LeadsTo::new(
    "started_leads_to_finished",
    |s| s.started,
    |s| s.finished
);
assert!(prop4.check(&states));
```

## Concurrent Testing

Test parallel operations on concurrent data structures:

```rust
use protest_stateful::concurrent::*;
use std::sync::{Arc, Mutex};

impl ConcurrentOperation for MyOp {
    fn execute_concurrent(&self, state: &Arc<Mutex<Self::State>>) {
        let mut state = state.lock().unwrap();
        self.execute(&mut state);
    }
}

#[test]
fn test_concurrent_operations() {
    let initial = MyState::new();
    let thread_count = 4;
    let ops_per_thread = 100;

    // Create operations for each thread
    let mut operations = vec![];
    for _ in 0..thread_count {
        operations.push(vec![/* operations */]);
    }

    let config = ConcurrentConfig {
        thread_count,
        operations_per_thread,
        check_linearizability: true,
    };

    let result = run_concurrent(initial, operations, config);
    assert!(result.is_ok());
}
```

## Execution Traces

Get detailed step-by-step execution information:

```rust
let trace = test.run_with_trace(&seq).unwrap();

println!("Initial state: {:?}", trace.initial_state());

for (operation, state) in trace.steps() {
    println!("After {}: {:?}", operation, state);
}

println!("Final state: {:?}", trace.final_state());
```

## Examples

See the [examples/](examples/) directory for complete examples:

- [`stack.rs`](examples/stack.rs) - Testing a stack implementation
- [`key_value_store.rs`](examples/key_value_store.rs) - Model-based testing of a key-value store
- [`concurrent_queue.rs`](examples/concurrent_queue.rs) - Concurrent testing of a queue

Run examples:

```bash
cargo run --example stack
cargo run --example key_value_store
cargo run --example concurrent_queue
```

## Use Cases

### Data Structures
Test stacks, queues, trees, graphs, and custom data structures with complex invariants.

### Databases & Key-Value Stores
Verify CRUD operations, transactions, consistency, and query correctness.

### State Machines & Protocols
Test connection protocols, parsers, and systems with well-defined states and transitions.

### Concurrent Systems
Find race conditions, deadlocks, and verify linearizability of concurrent data structures.

### APIs & Services
Test REST APIs, gRPC services, and distributed systems with stateful interactions.

### File Systems
Verify file operations, directory hierarchies, and consistency properties.

## Advanced Features

### Custom Shrinking

Operation sequences automatically shrink to minimal failing cases:

```rust
let shrunk = sequence.shrink();
// Returns progressively smaller sequences that might still fail
```

### Preconditions

Define when operations are valid:

```rust
fn precondition(&self, state: &Self::State) -> bool {
    match self {
        Op::Withdraw(amount) => state.balance >= *amount,
        _ => true,
    }
}
```

### Multiple Invariants

Add as many invariants as needed:

```rust
let test = StatefulTest::new(state)
    .invariant("positive_balance", |s| s.balance >= 0)
    .invariant("valid_transactions", |s| s.tx_count < 1000)
    .invariant("consistent_state", |s| s.is_consistent());
```

## Best Practices

1. **Start Simple**: Begin with basic invariants and add complexity
2. **Test Edge Cases**: Use preconditions to test boundary conditions
3. **Use Model-Based Testing**: When you have a simple reference implementation
4. **Shrink Sequences**: Let automatic shrinking find minimal failing cases
5. **Add Temporal Properties**: Express "eventually" and "always" requirements
6. **Test Concurrently**: Use concurrent testing for thread-safe data structures

## Comparison with Other Approaches

| Feature | Protest Stateful | Manual Testing | QuickCheck-style |
|---------|------------------|----------------|------------------|
| State Machine Testing | ✅ Built-in | ❌ Manual | 🟡 Possible |
| Model-Based Testing | ✅ Built-in | ❌ Manual | 🟡 Possible |
| Operation Shrinking | ✅ Automatic | ❌ None | ✅ Yes |
| Temporal Properties | ✅ Built-in | ❌ Manual | ❌ No |
| Concurrent Testing | ✅ Built-in | ❌ Manual | ❌ No |
| Execution Traces | ✅ Automatic | ❌ Manual | 🟡 Custom |

## Contributing

Contributions welcome! See [CONTRIBUTING.md](../CONTRIBUTING.md).

## License

MIT License - see [LICENSE](../LICENSE)

## See Also

- [Protest](https://github.com/shrynx/protest) - Core property testing library
- [Protest Extras](../protest-extras/) - Additional generators and shrinking strategies
- [Protest CLI](../protest-cli/) - Command-line tools for test management

---

**Protest Stateful** - Make your stateful systems robust through property-based testing.
