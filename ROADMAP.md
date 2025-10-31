# Protest Roadmap

This document provides detailed elaborations on the planned features and enhancements for the Protest property-based testing library.

## Table of Contents

1. [More Built-in Generators](#1-more-built-in-generators)
2. [Enhanced Shrinking Strategies](#2-enhanced-shrinking-strategies)
3. [Integration with More Test Frameworks](#3-integration-with-more-test-frameworks)
4. [Property Test Replay and Persistence](#4-property-test-replay-and-persistence)
5. [Coverage-Guided Generation](#5-coverage-guided-generation)
6. [Stateful Property Testing DSL](#6-stateful-property-testing-dsl)

---

## 1. More Built-in Generators

### Overview
Expand the library's collection of ready-to-use generators for common types and patterns, reducing the need for users to write custom generators for standard data types.

### Planned Generators

#### Network/Web Generators
```rust
// Network/Web generators
pub struct IpAddressGenerator; // IPv4/IPv6
pub struct UrlGenerator; // Valid URLs with different schemes
pub struct EmailGenerator; // RFC-compliant email addresses
pub struct JsonGenerator; // Valid JSON with configurable depth
```

#### Date/Time Generators
```rust
// Date/Time generators
pub struct DateTimeGenerator; // Dates within ranges
pub struct DurationGenerator; // Time durations
pub struct TimestampGenerator; // Unix timestamps
```

#### Domain-Specific Generators
```rust
// Domain-specific generators
pub struct UuidGenerator; // UUIDs (v4, v5, etc.)
pub struct Base64Generator; // Valid base64 strings
pub struct HexGenerator; // Hexadecimal strings
pub struct PathGenerator; // Valid file system paths
```

#### Complex Collection Generators
```rust
// Complex collection generators
pub struct NonEmptyVecGenerator<T>; // Ensures vec.len() >= 1
pub struct SortedVecGenerator<T>; // Pre-sorted vectors
pub struct UniqueVecGenerator<T>; // No duplicates
pub struct BoundedMapGenerator<K, V>; // HashMaps with size constraints
```

#### Constrained Numeric Generators
```rust
// Constrained numeric generators
pub struct PositiveIntGenerator<T>;
pub struct EvenNumberGenerator<T>;
pub struct PrimeNumberGenerator;
pub struct PercentageGenerator; // 0.0..=100.0
```

#### Text Generators
```rust
// Text generators
pub struct AlphabeticGenerator; // Only letters
pub struct AlphanumericGenerator; // Letters + numbers
pub struct IdentifierGenerator; // Valid Rust/programming identifiers
pub struct SentenceGenerator; // Realistic sentences
pub struct ParagraphGenerator; // Multiple sentences
```

### Usage Example
```rust
use protest::generators::web::EmailGenerator;

#[property_test]
fn test_email_validation(email: String) {
    // email is automatically a valid email format
    assert!(email.contains('@'));
    assert!(email.split('@').count() == 2);
}
```

### Benefits
- Reduces boilerplate for common testing scenarios
- Ensures correctness of generated data (e.g., valid emails, URLs)
- Provides domain-specific constraints out of the box
- Improves test coverage by generating realistic data

---

## 2. Enhanced Shrinking Strategies

### Overview
Improve how Protest finds minimal counterexamples when a property fails. Current shrinking is basic (reduce numbers toward 0, shorten strings). Enhanced shrinking will be smarter and context-aware.

### Planned Features

#### Smart Shrinking with Invariants
```rust
// Smart shrinking based on type structure
pub trait SmartShrink {
    // Shrink while maintaining invariants
    fn shrink_preserving<F>(&self, invariant: F) -> Box<dyn Iterator<Item = Self>>
    where
        F: Fn(&Self) -> bool;
}

// Example: Shrink a Vec but keep it sorted
let shrunk = sorted_vec.shrink_preserving(|v| {
    v.windows(2).all(|w| w[0] <= w[1])
});
```

#### Integrated Shrinking
```rust
// Integrated shrinking - shrink related values together
pub struct IntegratedShrinker<T> {
    // Knows relationships between fields
    dependencies: HashMap<String, Vec<String>>,
}

// Example: When shrinking a User struct
#[derive(Generator, Shrink)]
struct User {
    #[shrink(preserve = "age >= 18")]
    age: u8,

    #[shrink(dependent_on = "age")]
    years_employed: u8, // Always <= age
}
```

#### Delta Debugging
```rust
// Delta debugging shrinking
pub struct DeltaDebugShrinker<T> {
    // Binary search through collections to find minimal failing subset
}

// Use case: Find minimal failing subset of operations
let ops = vec![Op1, Op2, Op3, ..., Op100];
// Shrinks to find: vec![Op5, Op42] is minimal failing case
```

#### Targeted Shrinking
```rust
// Targeted shrinking
pub struct TargetedShrinker<T> {
    target_value: T, // Shrink toward specific value, not just "simple"
}
```

### Advanced Shrinking Example
```rust
#[property_test]
fn test_json_parser(json: JsonValue) {
    // If parsing fails, shrink to minimal invalid JSON
    match parse(json.to_string()) {
        Err(e) => {
            // Protest finds: {"a": } is minimal failure
            // Instead of full complex nested structure
        }
    }
}
```

### Benefits
- Faster debugging with minimal counterexamples
- Maintains structural invariants during shrinking
- Better handling of dependent fields
- More intelligent search through failure space

---

## 3. Integration with More Test Frameworks

### Overview
Make Protest work seamlessly with popular Rust testing frameworks and tools, enabling developers to use Protest in their existing testing infrastructure.

### Planned Integrations

#### 1. Native #[test] Integration
```rust
// Already works
#[test]
fn test_with_native() {
    protest::check_property(...);
}
```

#### 2. Criterion (Benchmarking) Integration
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use protest::criterion::PropertyBenchmark;

fn bench_sort_property(c: &mut Criterion) {
    c.bench_function("sort maintains length", |b| {
        b.iter_property(
            VecGenerator::new(IntGenerator::new(0, 1000), 0, 100),
            |v: Vec<i32>| {
                let mut sorted = v.clone();
                sorted.sort();
                assert_eq!(sorted.len(), v.len());
            }
        );
    });
}
```

#### 3. Proptest Migration Helper
```rust
use protest::compat::proptest;

// Automatically convert proptest strategies to Protest generators
let strategy = any::<u32>().prop_map(|x| x * 2);
let generator = protest::from_proptest(strategy);
```

#### 4. QuickCheck Compatibility Layer
```rust
use protest::compat::quickcheck;

impl quickcheck::Arbitrary for MyType {
    fn arbitrary(g: &mut Gen) -> Self {
        MyType::auto_generator().generate(g, &config)
    }
}
```

#### 5. Insta (Snapshot Testing) Integration
```rust
use protest::insta::PropertySnapshot;

#[property_test]
fn test_serialization_format(data: MyData) {
    let json = serde_json::to_string(&data).unwrap();

    // Create snapshots for interesting generated cases
    if data.is_edge_case() {
        insta::assert_snapshot!(json);
    }
}
```

#### 6. Tokio Integration (Async Testing)
```rust
#[tokio::test]
#[property_test(async)]
async fn test_async_operation(input: Input) {
    let result = async_operation(input).await;
    assert!(result.is_ok());
}
```

#### 7. Custom Test Harness
```rust
// Allow Protest to be the main test runner
fn main() {
    protest::test_runner::run_all_properties();
}
```

### Benefits
- Easier adoption in existing projects
- Leverage existing testing infrastructure
- Combine property-based testing with other testing paradigms
- Better IDE and tooling support

---

## 4. Property Test Replay and Persistence

### Overview
Add ability to save failing test cases and replay them deterministically, enabling regression testing and faster debugging.

### Planned Features

#### Seed Persistence
```rust
#[property_test(seed_file = ".protest/seeds.json")]
fn test_something(x: i32) {
    // Automatically saves failing seeds
    // Next run replays with same seed first
}
```

#### Failure Case Database
```rust
use protest::persistence::FailureDB;

let db = FailureDB::new(".protest/failures/");

// When test fails, save the input
#[property_test(persist_failures = true)]
fn test_parser(input: String) {
    // Failure saved to: .protest/failures/test_parser/case_001.json
    // {
    //   "seed": 12345,
    //   "input": "problematic input",
    //   "timestamp": "2025-01-15T10:30:00Z",
    //   "error": "ParseError at position 5"
    // }
}
```

#### Regression Test Generation
```rust
use protest::regression::generate_test;

// After fixing a bug, generate a permanent test case
protest::regression::generate_test!(
    "test_parser_regression_001",
    test_parser,
    seed = 12345
);

// Generated code:
#[test]
fn test_parser_regression_001() {
    let input = /* exact failing input */;
    test_parser(input);
}
```

#### Corpus Management
```rust
pub struct TestCorpus {
    interesting_cases: Vec<TestCase>,
    minimal_examples: Vec<TestCase>,
    edge_cases: Vec<TestCase>,
}

impl TestCorpus {
    // Add interesting generated values to corpus
    pub fn add_if_interesting(&mut self, value: T) {
        if self.is_interesting(&value) {
            self.interesting_cases.push(value);
        }
    }

    // Next test run includes corpus values
    pub fn as_generator(&self) -> CorpusGenerator<T>;
}
```

#### CLI Tool
```bash
# CLI tool for managing test cases
$ protest replay --seed 12345 --test test_parser
$ protest list-failures
$ protest minimize-failure --case test_parser/case_001.json
$ protest export-corpus --format=rust-tests
```

### Usage Example
```rust
// First run - finds bug
#[property_test(
    persist_failures = true,
    corpus_dir = "tests/corpus/parser"
)]
fn test_parser(input: String) {
    parse(input).unwrap(); // Fails with seed 42
}

// Protest saves:
// tests/corpus/parser/failure_001.json

// Second run - automatically tests saved failures first
// Then continues with random generation
// If bug is fixed, moves to regression suite
```

### Benefits
- Deterministic test reproduction
- Automatic regression test suite building
- Faster CI/CD (test known failures first)
- Historical tracking of edge cases
- Shareable failure cases across team

---

## 5. Coverage-Guided Generation

### Overview
Use code coverage feedback to guide test input generation toward unexplored code paths, similar to AFL/libFuzzer. This dramatically improves the quality of generated test inputs.

### How It Works

```rust
use protest::coverage::CoverageGuided;

#[property_test(
    coverage_guided = true,
    target_coverage = 95.0
)]
fn test_complex_function(input: ComplexInput) {
    // Protest monitors which branches are hit
    complex_function(input);
}
```

### Implementation Architecture

```rust
pub struct CoverageGuidedGenerator<T> {
    base_generator: Box<dyn Generator<T>>,
    coverage_map: HashMap<BranchId, usize>,
    interesting_inputs: Vec<(T, CoverageSignature)>,
}

impl<T> CoverageGuidedGenerator<T> {
    fn generate(&self, rng: &mut Rng, config: &Config) -> T {
        // 1. Generate new input
        let mut input = self.base_generator.generate(rng, config);

        // 2. Execute and measure coverage
        let coverage = self.measure_coverage(&input);

        // 3. If new coverage found, mutate this input more
        if self.is_new_coverage(&coverage) {
            self.interesting_inputs.push((input.clone(), coverage));

            // 4. Mutate interesting input to explore nearby space
            input = self.mutate_near(&input);
        }

        input
    }

    // Mutations that explore code space
    fn mutate_near(&self, input: &T) -> T {
        // Bit flips, boundary values, arithmetic operations, etc.
    }
}
```

### Practical Example

```rust
// Example: Testing a parser
#[property_test(coverage_guided = true)]
fn test_json_parser(json: String) {
    // First iterations: random JSON strings
    // Coverage feedback: "haven't seen nested objects yet"
    // Later iterations: generates more nested structures
    // Coverage feedback: "haven't seen arrays in objects"
    // Later iterations: generates {"key": [1, 2, 3]}

    let result = parse_json(&json);
    // Eventually covers all parser code paths
}
```

### LLVM Integration

```rust
// Integration with LLVM coverage
pub struct LLVMCoverageGuided {
    // Uses LLVM sanitizer coverage hooks
    // Tracks basic blocks, edges, comparisons
}
```

### Energy Scheduling

```rust
// Energy allocation based on coverage
pub struct EnergyScheduler {
    // Spend more test iterations on inputs that find new coverage
    energy_per_input: HashMap<InputId, f64>,
}
```

### Advanced Features

```rust
// Custom coverage metrics
#[property_test(
    coverage_guided = true,
    metrics = [BranchCoverage, PathCoverage, DataFlowCoverage]
)]
fn test_with_multiple_metrics(input: Input) {
    // Track multiple coverage dimensions
}

// Coverage-guided shrinking
// When failure found, shrink while maintaining same coverage path
let minimal = shrink_preserving_coverage(failing_input);
```

### Benefits
- Discovers edge cases that random generation misses
- Systematically explores all code paths
- Finds bugs faster with intelligent input generation
- Reduces wasted iterations on redundant inputs
- Achieves higher code coverage with fewer test cases

---

## 6. Stateful Property Testing DSL

### Overview
A domain-specific language for testing systems with state and sequences of operations. This enables testing complex stateful systems like databases, file systems, concurrent data structures, and APIs.

### Problem It Solves

Testing that a sequence of operations maintains invariants:
- Database operations (insert, update, delete, query)
- File system operations (create, write, read, delete)
- UI interactions (click, type, navigate)
- Concurrent data structure operations
- Protocol implementations (state machines)

### Basic DSL Design

```rust
use protest::stateful::*;

// Define the system under test
struct Stack<T> {
    items: Vec<T>,
}

// Define operations as an enum
#[derive(Debug, Clone, Generator)]
enum StackOp {
    Push(i32),
    Pop,
    Peek,
    Clear,
    Length,
}

// Define the state machine
stateful_test! {
    name: stack_properties,
    state: Stack<i32>,
    operations: StackOp,

    // Initialize state
    init: || Stack { items: vec![] },

    // Define how operations execute
    execute: |state, op| {
        match op {
            StackOp::Push(x) => state.items.push(x),
            StackOp::Pop => { state.items.pop(); },
            StackOp::Peek => { state.items.last(); },
            StackOp::Clear => state.items.clear(),
            StackOp::Length => { state.items.len(); },
        }
    },

    // Invariants that must hold after each operation
    invariants: [
        "length_non_negative" => |state| state.items.len() >= 0,
        "pop_decreases_length" => |state, op, old_state| {
            match op {
                StackOp::Pop if !old_state.items.is_empty() => {
                    state.items.len() == old_state.items.len() - 1
                },
                _ => true
            }
        },
    ],

    // Preconditions for operations
    preconditions: {
        StackOp::Pop => |state| !state.items.is_empty(),
        StackOp::Peek => |state| !state.items.is_empty(),
    },
}
```

### Model-Based Testing

```rust
// More advanced example: Testing a key-value store
stateful_test! {
    name: kv_store_properties,
    state: HashMap<String, String>,
    operations: KVOp,

    init: || HashMap::new(),

    // Model-based testing
    model: {
        type Model = HashMap<String, String>;

        execute_model: |model, op| {
            match op {
                KVOp::Set(k, v) => { model.insert(k, v); },
                KVOp::Get(k) => { model.get(k); },
                KVOp::Delete(k) => { model.remove(k); },
            }
        },

        // Assert real system matches model
        check_equivalence: |state, model| {
            state.len() == model.len() &&
            model.iter().all(|(k, v)| state.get(k) == Some(v))
        },
    },

    // Parallel operation testing
    parallel: {
        thread_count: 4,
        check_linearizability: true,
    },
}
```

### File System Testing

```rust
// Testing a file system
stateful_test! {
    name: filesystem_properties,
    state: FileSystem,
    operations: FSOp,

    init: || FileSystem::new_temp(),

    cleanup: |state| {
        state.cleanup(); // Remove temp files
    },

    execute: |state, op| {
        match op {
            FSOp::CreateFile(path) => state.create_file(path),
            FSOp::WriteFile(path, data) => state.write(path, data),
            FSOp::ReadFile(path) => state.read(path),
            FSOp::DeleteFile(path) => state.delete(path),
            FSOp::CreateDir(path) => state.mkdir(path),
        }
    },

    invariants: [
        "no_orphaned_files" => |state| {
            // All files have valid parent directories
            state.all_files().iter().all(|f| {
                state.parent_exists(f)
            })
        },
        "write_read_consistency" => |state, op, old_state| {
            match op {
                FSOp::WriteFile(path, data) => {
                    state.read(path) == Ok(data)
                },
                _ => true
            }
        },
    ],
}
```

### Generated Test Output

```rust
#[test]
fn test_stack_properties() {
    // Protest generates sequences like:
    // [Push(5), Push(3), Pop, Push(7), Length, Clear]
    // And verifies invariants after each operation
}

// Shrinking sequences
// If [Op1, Op2, Op3, Op4, Op5] fails
// Shrink to minimal: [Op2, Op4]
```

### Temporal Properties

```rust
stateful_test! {
    name: temporal_properties,
    state: System,
    operations: Op,

    temporal_invariants: [
        // "Eventually, the system reaches steady state"
        eventually: |history| {
            history.windows(10).any(|w| w.all_equal())
        },

        // "Always, if condition A then eventually condition B"
        always_eventually: |history| {
            history.iter().all(|(state, op)| {
                if is_condition_a(state) {
                    future_states.any(|s| is_condition_b(s))
                } else {
                    true
                }
            })
        },
    ],
}
```

### Concurrency Testing

```rust
stateful_test! {
    name: concurrent_operations,
    state: AtomicCounter,
    operations: CounterOp,

    parallel: {
        threads: 8,
        operations_per_thread: 100,

        // Check for race conditions
        check_linearizability: true,
        check_serializability: true,
    },
}
```

### Benefits
- Test complex stateful systems systematically
- Automatically generate realistic operation sequences
- Verify invariants hold across state transitions
- Find subtle bugs in state machine implementations
- Test concurrent systems for race conditions
- Model-based testing for correctness verification
- Minimal failing sequences through intelligent shrinking

---

## Implementation Priority

Based on community feedback and practical impact, suggested implementation order:

1. **More Built-in Generators** - High impact, relatively straightforward
2. **Property Test Replay and Persistence** - Critical for debugging and CI/CD
3. **Enhanced Shrinking Strategies** - Improves developer experience significantly
4. **Integration with More Test Frameworks** - Increases adoption
5. **Coverage-Guided Generation** - Advanced feature, high complexity but high value
6. **Stateful Property Testing DSL** - Most complex, but enables entirely new use cases

## Contributing

We welcome contributions to any of these roadmap items! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

For discussions about roadmap priorities or new feature ideas, please open an issue on GitHub.

---

*Last updated: 2025-01-15*
