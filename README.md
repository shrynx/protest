# ✊ Protest

**Property-Based Testing for Rust** - An ergonomic, powerful, and feature-rich property testing library with minimal boilerplate.

[![Crates.io](https://img.shields.io/crates/v/protest.svg)](https://crates.io/crates/protest)
[![Documentation](https://docs.rs/protest/badge.svg)](https://docs.rs/protest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- 🚀 **Ergonomic API** - Test properties with closures, no boilerplate
- 🎯 **Automatic Generator Inference** - Smart type-based generator selection
- 🔧 **Derive Macros** - `#[derive(Generator)]` for custom types
- 📦 **Declarative Macros** - `property!`, `assert_property!`, `generator!`
- ⚡ **Async Support** - First-class async property testing
- 🔄 **Smart Shrinking** - Automatic minimal counterexample finding
- 💾 **Failure Persistence** - Save and replay failing test cases (optional)
- 🔧 **CLI Tool** - Manage failures from the command line ([protest-cli](protest-cli/))
- 🎨 **Fluent Builders** - Chain configuration methods naturally
- 🧪 **Common Patterns** - Built-in helpers for mathematical properties
- 🔀 **Parallel Execution** - Run tests in parallel for speed
- 📊 **Statistics & Coverage** - Track generation and test coverage
- 🎭 **Flexible** - Works with any type, sync or async

## Quick Start

Add Protest to your `Cargo.toml`:

```toml
[dev-dependencies]
protest = { version = "0.1", features = ["derive", "persistence"] }
protest-extras = "0.1"       # Optional: Extra generators (network, datetime, text, etc.)
protest-stateful = "0.1"     # Optional: Stateful property testing for state machines
```

**CLI Tool** (optional, for managing test failures):
```bash
cargo install protest-cli
```

### Ultra-Simple Example

```rust
use protest::*;

#[test]
fn test_addition_commutative() {
    // Test that addition is commutative with just one line!
    property!(generator!(i32, -100, 100), |(a, b)| a + b == b + a);
}
```

### Ergonomic API Example

```rust
use protest::ergonomic::*;

#[test]
fn test_reverse_twice_is_identity() {
    property(|mut v: Vec<i32>| {
        let original = v.clone();
        v.reverse();
        v.reverse();
        v == original
    })
    .iterations(1000)
    .run_with(VecGenerator::new(IntGenerator::new(-50, 50), 0, 100))
    .expect("Property should hold");
}
```

### Attribute Macro Example

```rust
use protest::property_test;

#[property_test(iterations = 100)]
fn test_string_length(s: String) {
    // Generator automatically inferred from type
    assert!(s.len() >= 0);
}
```

### Custom Struct Example

```rust
use protest::Generator;

#[derive(Debug, Clone, PartialEq, Generator)]
struct User {
    #[generator(range = "1..1000")]
    id: u32,

    #[generator(length = "5..50")]
    name: String,

    age: u8,
    active: bool,
}

#[property_test]
fn test_user_id(user: User) {
    assert!(user.id > 0 && user.id < 1000);
}
```

## API Styles

Protest offers multiple API styles - use what fits your needs:

### 1. Declarative Macros (Most Concise)

```rust
use protest::*;

// Simple property test
property!(generator!(i32, 0, 100), |x| x >= 0);

// With configuration
property!(
    generator!(i32, 0, 100),
    iterations = 1000,
    seed = 42,
    |x| x >= 0
);

// Assert style (panics on failure)
assert_property!(
    generator!(i32, 0, 100),
    |x| x * 2 > x,
    "Doubling should increase positive numbers"
);
```

### 2. Fluent Builder API (Most Flexible)

```rust
use protest::ergonomic::*;

property(|x: i32| x.abs() >= 0)
    .iterations(1000)
    .seed(42)
    .max_shrink_iterations(500)
    .run_with(IntGenerator::new(-100, 100))
    .expect("Absolute value is always non-negative");
```

### 3. Attribute Macros (Most Integrated)

```rust
use protest::property_test;

#[property_test(iterations = 100, seed = 42)]
fn test_vec_operations(v: Vec<i32>) {
    let mut sorted = v.clone();
    sorted.sort();
    assert!(sorted.windows(2).all(|w| w[0] <= w[1]));
}
```

### 4. Direct API (Most Control)

```rust
use protest::*;

struct MyProperty;
impl Property<i32> for MyProperty {
    type Output = ();
    fn test(&self, input: i32) -> Result<(), PropertyError> {
        if input >= 0 {
            Ok(())
        } else {
            Err(PropertyError::property_failed("negative number"))
        }
    }
}

let result = check(IntGenerator::new(0, 100), MyProperty);
assert!(result.is_ok());
```

## Common Property Patterns

Protest includes built-in helpers for common mathematical properties:

```rust
use protest::ergonomic::patterns::*;

// Commutativity: f(a, b) == f(b, a)
commutative(|a: i32, b: i32| a + b);

// Associativity: f(f(a, b), c) == f(a, f(b, c))
associative(|a: i32, b: i32| a + b);

// Idempotence: f(f(x)) == f(x)
idempotent(|x: i32| x.abs());

// Round-trip: decode(encode(x)) == x
round_trip(
    |x: i32| x.to_string(),
    |s: String| s.parse().unwrap()
);

// Inverse functions: f(g(x)) == x && g(f(x)) == x
inverse(|x: i32| x * 2, |x: i32| x / 2);

// Identity element: f(x, e) == x
has_identity(|a: i32, b: i32| a + b, 0);

// Monotonicity
monotonic_increasing(|x: i32| x * x);

// Distributivity
distributive(
    |a: i32, b: i32| a * b,
    |a: i32, b: i32| a + b
);
```

## Async Support

Full support for runtime-agnostic async property testing. Works with any async runtime (tokio, async-std, smol):

```rust
use protest::*;

struct AsyncFetchProperty;

impl AsyncProperty<u32> for AsyncFetchProperty {
    type Output = ();

    async fn test(&self, id: u32) -> Result<(), PropertyError> {
        let user = fetch_user(id).await;
        if id > 0 && user.is_none() {
            Err(PropertyError::property_failed("User not found"))
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_async_property() {
    let result = check_async(
        IntGenerator::new(1, 100),
        AsyncFetchProperty
    ).await;

    assert!(result.is_ok());
}
```

**Note:** Protest is runtime-agnostic - you bring your own async runtime. Add tokio, async-std, or smol to your dev-dependencies as needed.

## Automatic Generator Inference

Protest automatically infers generators for common types:

```rust
use protest::ergonomic::AutoGen;

// All primitive types
i32::auto_generator();
String::auto_generator();
bool::auto_generator();

// Collections
Vec::<i32>::auto_generator();
HashMap::<String, i32>::auto_generator();

// Tuples
<(i32, String)>::auto_generator();

// Options
Option::<i32>::auto_generator();

// Your custom types with #[derive(Generator)]
User::auto_generator();
```

## Shrinking

When a property fails, Protest automatically finds the minimal counterexample:

```rust
property!(generator!(i32, 1, 100), |x| x < 50);
// Fails with: Property failed with input 50 (shrunk from larger value)
//           Focus on input: 50
```

## Configuration

Extensive configuration options:

```rust
use protest::*;
use std::time::Duration;

let config = TestConfig {
    iterations: 1000,                            // Number of test cases
    seed: Some(42),                               // For reproducibility
    max_shrink_iterations: 500,                  // Shrinking limit
    shrink_timeout: Duration::from_secs(10),     // Shrinking timeout
    generator_config: GeneratorConfig {
        size_hint: 100,                          // Size for collections
        max_depth: 5,                            // For nested structures
        ..GeneratorConfig::default()
    },
    ..TestConfig::default()
};
```

## Failure Persistence & Replay

Save failing test cases and automatically replay them for debugging and regression testing (requires `persistence` feature):

```toml
[dev-dependencies]
protest = { version = "0.1", features = ["persistence"] }
```

### Automatic Replay

Protest automatically saves failures and replays them on subsequent test runs:

```rust
use protest::*;

PropertyTestBuilder::new()
    .test_name("my_critical_test")
    .persist_failures()  // Enable automatic failure saving & replay
    .iterations(10000)
    .run(u32::arbitrary(), |x: u32| {
        // Your property test
        if x > 1000 {
            Err("Value too large")
        } else {
            Ok(())
        }
    });
```

**What happens:**
1. **First run**: If test fails, seed and input are saved to `.protest/failures/my_critical_test/failure_seed_{seed}.json`
2. **Subsequent runs**: Before running new test cases, saved failures are replayed using their seeds
3. **Auto-cleanup**: If a replayed failure now passes, it's automatically deleted
4. **Regression detection**: If failures still fail, they're reported with their seeds for easy reproduction

Output example:
```
🔄 Replaying 1 saved failure(s) for 'my_critical_test'...
  Replay 1/1: seed=12345
    ❌ Still failing: Property failed: Value too large

⚠️  1 failure(s) still failing:
    seed=12345
```

### Manual Failure Management

```rust
use protest::{FailureSnapshot, FailureCase, PersistenceConfig};

// Custom persistence configuration
let config = PersistenceConfig::enabled()
    .with_failure_dir(".custom/failures")
    .enable_corpus();

PropertyTestBuilder::new()
    .test_name("custom_test")
    .persistence_config(config)
    .iterations(1000)
    .run(generator, property);

// Load and inspect saved failures
let snapshot = FailureSnapshot::new(".protest/failures")?;
let failures = snapshot.load_failures("my_critical_test")?;

for failure in failures {
    println!("Seed: {}", failure.seed);
    println!("Input: {}", failure.input);
    println!("Error: {}", failure.error_message);
    println!("Shrink steps: {}", failure.shrink_steps);
}

// Manually delete a fixed failure
snapshot.delete_failure("my_critical_test", 12345)?;
```

### CLI Tool for Managing Failures

Install the CLI tool to manage failures from the command line:

```bash
cargo install --path protest-cli
```

Or use directly from the workspace:

```bash
cargo run -p protest-cli -- <command>
```

**List all tests with failures:**
```bash
protest list
protest list --verbose  # Show detailed information
```

Output:
```
Found 2 test(s) with failures:

  ● example_test (2 failures)
  ● parser_test (1 failure)

Tip: Use --verbose for more details
```

**Show details for a specific test:**
```bash
protest show example_test
```

Output:
```
Failures for test 'example_test':

Failure #1
  Seed: 12345
  Input: 571962454
  Error: Property failed: Value too large
  Shrink steps: 15
  Timestamp: 2025-01-02 16:00:00 UTC

  Reproduce: cargo test -- --nocapture
  Or use: .seed(12345)
```

**Show statistics:**
```bash
protest stats
```

Output:
```
Failure Statistics

  Total tests with failures: 2
  Total failures: 3
  Average failures per test: 1.5
  Total shrink steps: 26
  Average shrink steps per failure: 8.7
  Oldest failure: 2025-01-02 16:00:00 UTC
  Newest failure: 2025-01-02 16:03:20 UTC
```

**Clean failures:**
```bash
# Delete a specific failure
protest clean example_test --seed 12345

# Delete all failures for a test
protest clean example_test

# Delete all failures (with confirmation)
protest clean

# Skip confirmation prompt
protest clean -y
```

**Custom failure directory:**
```bash
protest --dir .custom/failures list
```

### Test Corpus

Build a corpus of interesting test cases:

```rust
use protest::TestCorpus;

let mut corpus = TestCorpus::new(".protest/corpus/parser")?;

// Add interesting cases manually
corpus.add_case(
    r#"{"nested": {"deeply": {"value": 42}}}"#.to_string(),
    "Complex nested JSON".to_string(),
)?;

// Load and use corpus cases in tests
corpus.load_all()?;
for case in corpus.cases() {
    println!("Testing: {} - {}", case.reason, case.input);
}
```

## CLI Tool

The **protest-cli** tool provides a command-line interface for managing test failures:

```bash
# Install
cargo install protest-cli

# List all failures
protest list

# Show details for a test
protest show my_test

# View statistics
protest stats

# Clean failures
protest clean my_test --seed 12345

# Generate regression tests
protest generate my_test
```

See the [CLI documentation](protest-cli/README.md) for more details.

### Regression Test Generation

Automatically convert saved failures into permanent regression tests:

```rust
use protest::{RegressionConfig, RegressionGenerator, FailureSnapshot};

let snapshot = FailureSnapshot::new(".protest/failures")?;
let config = RegressionConfig::new("tests/regressions");
let generator = RegressionGenerator::new(config);

// Generate regression tests for all failures
let files = generator.generate_all(&snapshot)?;

for file in files {
    println!("Generated: {}", file.display());
}
```

Or use the CLI:
```bash
# Generate for specific test
protest generate my_test

# Generate for all tests
protest generate

# Custom output directory
protest generate --output tests/custom_regressions
```

Generated test example:
```rust
/// Regression test for failure with seed 12345
///
/// Original error: Property failed: Value too large
/// Input: 571962454
/// Discovered: 2025-01-02 16:00:00 UTC
#[test]
fn regression_my_test_seed_12345() {
    PropertyTestBuilder::new()
        .iterations(1)
        .seed(12345)
        .run(your_generator, your_property)
        .expect("Regression: failure should not reoccur");
}
```

### Coverage-Guided Corpus Building

Build a corpus of interesting test cases based on code coverage:

```rust
use protest::{CoverageCorpusConfig, CoverageCorpus, path_hash};

let config = CoverageCorpusConfig::new(".protest/corpus")
    .with_min_coverage(5.0)  // 5% minimum coverage increase
    .with_max_size(1000)     // Max 1000 inputs
    .auto_optimize(true);    // Auto-remove redundant cases

let mut corpus = CoverageCorpus::new(config)?;

// During property testing, track coverage
property(|x: i32| {
    // Your property logic here
    let path = path_hash(&[x, /* execution path markers */]);

    // Add to corpus if it increases coverage
    corpus.try_add(&x, path)?;

    Ok(())
})
```

Get coverage statistics:
```rust
let stats = corpus.stats();
println!("Total unique paths: {}", stats.total_paths);
println!("Corpus size: {}", stats.corpus_size);
```

## Stateful Property Testing

**protest-stateful** provides a powerful DSL for testing stateful systems like state machines, databases, and APIs.

### Testing State Machines

```rust
use protest_stateful::prelude::*;

#[derive(Debug, Clone)]
struct Stack { items: Vec<i32> }

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
    let test = StatefulTest::new(Stack { items: vec![] })
        .invariant("length_non_negative", |s| s.items.len() >= 0);

    let mut seq = OperationSequence::new();
    seq.push(StackOp::Push(10));
    seq.push(StackOp::Pop);

    assert!(test.run(&seq).is_ok());
}
```

### Model-Based Testing

Compare your system against a reference implementation:

```rust
use protest_stateful::prelude::*;

#[derive(Debug, Clone)]
struct SimpleModel {
    data: HashMap<String, String>,
}

impl Model for SimpleModel {
    type SystemState = MyComplexSystem;
    type Operation = MyOp;

    fn execute_model(&mut self, op: &Self::Operation) {
        // Execute on simple model
    }

    fn matches(&self, system: &Self::SystemState) -> bool {
        // Compare model to actual system
        true
    }
}
```

### Temporal Properties

```rust
use protest_stateful::temporal::*;

let states = vec![/* execution trace */];

// "Eventually" - property must hold at some point
let prop = Eventually::new("reaches_goal", |s| s.is_goal());
assert!(prop.check(&states));

// "Always" - property must hold at every point
let prop = Always::new("non_negative", |s| s.value >= 0);
assert!(prop.check(&states));
```

### Concurrent Testing

Test parallel operations on concurrent data structures:

```rust
use protest_stateful::concurrent::*;

let config = ConcurrentConfig {
    thread_count: 4,
    operations_per_thread: 100,
    check_linearizability: true,
};

let result = run_concurrent(initial_state, operations, config);
assert!(result.is_ok());
```

**Learn more:** See [protest-stateful README](protest-stateful/README.md) for full documentation.

## Examples

The repository includes comprehensive examples:

- [`basic_usage.rs`](examples/basic_usage.rs) - Getting started
- [`ergonomic_api_demo.rs`](examples/ergonomic_api_demo.rs) - All ergonomic features
- [`custom_structs.rs`](examples/custom_structs.rs) - Custom types with derive
- [`async_properties.rs`](examples/async_properties.rs) - Async testing
- [`advanced_patterns.rs`](examples/advanced_patterns.rs) - Advanced techniques

Run examples:
```bash
cargo run --example ergonomic_api_demo
cargo run --example custom_structs
cargo run --example async_properties
```

## Feature Flags

```toml
[features]
default = ["derive"]
derive = ["protest-derive"]    # Derive macros for Generator trait
persistence = ["serde", "serde_json"]  # Failure persistence & replay
```

Protest has minimal dependencies and no required runtime dependencies. Async support is built-in and runtime-agnostic. The `persistence` feature is optional and adds `serde` for JSON serialization of test failures.

## Comparison with Other Libraries

| Feature | Protest | proptest | quickcheck |
|---------|---------|----------|------------|
| Ergonomic API | ✅ | ❌ | ❌ |
| Automatic Inference | ✅ | ❌ | Partial |
| Derive Macros | ✅ | ✅ | ✅ |
| Async Support | ✅ | ❌ | ❌ |
| Declarative Macros | ✅ | ❌ | ❌ |
| Fluent Builders | ✅ | Partial | ❌ |
| Pattern Helpers | ✅ | ❌ | ❌ |
| Shrinking | ✅ | ✅ | ✅ |
| Statistics | ✅ | Partial | ❌ |
| Failure Persistence | ✅ | Partial | ❌ |
| Test Corpus | ✅ | ❌ | ❌ |

## Documentation

Full documentation is available on [docs.rs](https://docs.rs/protest).

### Key Modules

- `protest::ergonomic` - Ergonomic API (closures, builders, patterns)
- `protest::primitives` - Built-in generators (int, string, vec, hashmap, etc.)
- `protest::generator` - Generator trait and utilities
- `protest::property` - Property trait and execution
- `protest::shrink` - Shrinking infrastructure
- `protest::persistence` - Failure persistence and replay (optional)
- `protest::config` - Configuration types
- `protest::statistics` - Coverage and statistics

### Protest Extras

The [`protest-extras`](protest-extras/) crate provides 23 additional specialized generators and enhanced shrinking strategies:

See the [protest-extras README](protest-extras/README.md) for detailed examples and documentation.


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Inspired by:
- [proptest](https://github.com/proptest-rs/proptest) - Rust property testing
- [QuickCheck](https://github.com/BurntSushi/quickcheck) - Original Rust QuickCheck
- [Hypothesis](https://hypothesis.works/) - Python property testing

## Roadmap

- [x] More built-in generators (protest-extras)
- [x] Enhanced shrinking strategies (protest-extras)
- [x] Property test replay and persistence
- [x] Stateful property testing DSL (protest-stateful)
- [ ] Integration with more test frameworks
- [ ] Coverage-guided generation (advanced)

---

Made with ❤️ for the Rust community
