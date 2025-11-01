# Protest Stateful

**Stateful property testing for Rust** - Test state machines, APIs, databases, concurrent systems, and any system that maintains state across operations.

Part of the [Protest](https://github.com/shrynx/protest) property testing ecosystem.

## Features

- 🔄 **State Machine Testing** - Define operations and invariants, automatically test sequences
- 🎯 **Model-Based Testing** - Compare real system behavior against a reference model
- ⚡ **Advanced Sequence Shrinking** - Delta debugging and smart shrinking for minimal counterexamples
- 🔍 **Preconditions & Postconditions** - Define valid operation contexts
- ⏱️ **Temporal Properties** - Express "eventually", "always", and "leads to" properties
- 🧵 **Linearizability Verification** - Verify concurrent operations are linearizable
- 🎨 **History Visualization** - Visual timeline and conflict analysis for concurrent executions
- 📊 **Execution Traces** - Detailed step-by-step state visualization
- 🔧 **Derive Macros** - Automatically implement Operation trait with `#[derive(Operation)]`

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

### Using Derive Macros (Simpler Approach)

The `#[derive(Operation)]` macro automatically implements the Operation trait:

```rust
use protest_stateful::{Operation, prelude::*};

// Automatically implement Operation trait
#[derive(Debug, Clone, Operation)]
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
    Clear,
}

#[test]
fn test_with_derive() {
    let test = StatefulTest::new(vec![])
        .invariant("bounded", |s: &Vec<i32>| s.len() <= 100);

    let mut seq = OperationSequence::new();
    seq.push(StackOp::Push(42));
    seq.push(StackOp::Pop);

    assert!(test.run(&seq).is_ok());
}
```

**Derive Macro Features:**
- `#[operation(state = "Type")]` - Specify the state type
- `#[execute("expression")]` - Define execution logic
- `#[precondition("expression")]` - Add precondition checks
- `#[weight(N)]` - Control operation frequency (higher = more frequent)
- `#[description("text")]` - Custom operation descriptions

For unnamed fields (tuple variants), use `field_0`, `field_1`, etc. in expressions.
For named fields, use the field names directly.

### Weight-Based Operation Generation

Generate operations according to their weights to create realistic test scenarios:

```rust
use protest_stateful::{Operation, operations::WeightedGenerator};
use rand::thread_rng;

#[derive(Debug, Clone, Operation)]
#[operation(state = "BankAccount")]
enum BankOp {
    #[execute("state.deposit(*field_0)")]
    #[weight(10)]  // Common: deposits happen frequently
    Deposit(u32),

    #[execute("state.withdraw(*field_0)")]
    #[precondition("state.balance >= *field_0")]
    #[weight(7)]   // Fairly common
    Withdraw(u32),

    #[execute("let _ = state.balance")]
    #[weight(15)]  // Very common: balance checks
    CheckBalance,

    #[execute("state.close()")]
    #[weight(1)]   // Rare: account closures
    Close,
}

// Create a weighted generator
let variants = vec![
    BankOp::Deposit(10),
    BankOp::Withdraw(5),
    BankOp::CheckBalance,
    BankOp::Close,
];
let mut generator = WeightedGenerator::new(variants, thread_rng());

// Generate 100 operations with realistic frequencies
let operations = generator.generate(100);
// CheckBalance appears ~44% of the time (weight 15/34)
// Deposit appears ~29% of the time (weight 10/34)
// Withdraw appears ~20% of the time (weight 7/34)
// Close appears ~3% of the time (weight 1/34)
```

**Benefits of weighted generation:**
- **Realistic workloads**: Mirror real-world usage patterns
- **Common paths tested more**: High-frequency operations get more coverage
- **Rare edge cases still tested**: Low-weight operations still appear occasionally
- **Performance testing**: Simulate production-like operation distributions

See the [weighted_generation.rs](examples/weighted_generation.rs) example for complete demonstrations.

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

## Concurrent Testing & Linearizability Verification

### Linearizability Checking

Verify that concurrent operations are linearizable - ensuring they appear to execute atomically at some point between invocation and response.

```rust
use protest_stateful::concurrent::linearizability::*;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

// Define a sequential specification
#[derive(Debug)]
struct QueueModel {
    queue: VecDeque<i32>,
}

impl SequentialSpec for QueueModel {
    fn apply(&mut self, operation: &str) -> String {
        if let Some(val) = operation.strip_prefix("enqueue(") {
            let v: i32 = val.trim_end_matches(')').parse().unwrap();
            self.queue.push_back(v);
            "ok".to_string()
        } else if operation == "dequeue()" {
            self.queue.pop_front()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "empty".to_string())
        } else {
            "unknown".to_string()
        }
    }

    fn reset(&mut self) {
        self.queue.clear();
    }
}

#[test]
fn test_queue_linearizability() {
    let mut history = History::new();
    let start = Instant::now();

    // Record concurrent operations
    let op1 = history.record_invocation(0, "enqueue(1)".to_string(), start);
    history.record_response(op1, "ok".to_string(), start + Duration::from_millis(10));

    let op2 = history.record_invocation(1, "enqueue(2)".to_string(),
        start + Duration::from_millis(5));
    history.record_response(op2, "ok".to_string(), start + Duration::from_millis(15));

    let op3 = history.record_invocation(2, "dequeue()".to_string(),
        start + Duration::from_millis(20));
    history.record_response(op3, "1".to_string(), start + Duration::from_millis(30));

    // Check linearizability
    let model = QueueModel { queue: VecDeque::new() };
    let mut checker = LinearizabilityChecker::new(model);

    let result = checker.check(&history);

    match result {
        LinearizabilityResult::Linearizable { order } => {
            println!("✓ Operations are linearizable!");
            println!("Valid order: {:?}", order);
        }
        LinearizabilityResult::NotLinearizable { reason, .. } => {
            panic!("Not linearizable: {}", reason);
        }
    }
}
```

### Visualization

Visualize concurrent histories and linearizability results:

```rust
// Visualize the execution timeline
println!("{}", history.visualize());

// Get detailed linearizability analysis
println!("{}", result.visualize(&history));
```

### Basic Concurrent Testing

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
        check_linearizability: false,  // Set to true to enable checking
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
- [`sequence_shrinking.rs`](examples/sequence_shrinking.rs) - Advanced shrinking strategies demonstration
- [`linearizability_verification.rs`](examples/linearizability_verification.rs) - Linearizability checking for concurrent operations
- [`derive_macro.rs`](examples/derive_macro.rs) - Using #[derive(Operation)] for automatic trait implementation

Run examples:

```bash
cargo run --example stack
cargo run --example key_value_store
cargo run --example concurrent_queue
cargo run --example sequence_shrinking
cargo run --example linearizability_verification
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

### Advanced Sequence Shrinking

Protest-stateful includes sophisticated shrinking algorithms to find minimal failing sequences:

#### Delta Debugging

Uses binary search to find minimal failing subsequences in O(n log n) tests:

```rust
use protest_stateful::operations::shrinking::*;

let shrinker = DeltaDebugSequenceShrinker::new(failing_sequence);
let test = StatefulTest::new(initial_state)
    .invariant("property", |s| s.is_valid());

// Find minimal sequence that still fails
let (minimal, test_count) = shrinker.minimize_with_stats(|seq| {
    test.run(seq).is_err()
});

println!("Reduced from {} to {} operations in {} tests",
    failing_sequence.len(), minimal.len(), test_count);
```

#### Smart Shrinking with Constraints

Shrink while preserving invariants and preconditions:

```rust
let config = SmartSequenceShrinking::new()
    .preserve_invariants(true)
    .preserve_preconditions(true)
    .max_attempts(1000);

let minimal = config.shrink(&failing_sequence, &initial_state, |seq| {
    test.run(seq).is_err()
});

// The minimal sequence is guaranteed to:
// 1. Still fail the test
// 2. Respect all preconditions
// 3. Maintain invariants during execution
```

See the [sequence_shrinking example](examples/sequence_shrinking.rs) for complete demonstrations.

### Basic Shrinking

Operation sequences also have basic shrinking built-in:

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
