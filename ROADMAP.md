# Protest Roadmap

This document provides detailed elaborations on the planned features and enhancements for the Protest property-based testing library.

## Status Overview

### ✅ Completed Features

1. **More Built-in Generators** - ✅ COMPLETE (protest-extras)
2. **Enhanced Shrinking Strategies** - ✅ COMPLETE (protest-extras)
3. **Property Test Replay and Persistence** - ✅ COMPLETE (protest core + protest-cli)
4. **Stateful Property Testing DSL** - ✅ COMPLETE (protest-stateful)

### 🚧 In Progress / Partially Complete

5. **Integration with More Test Frameworks** - 🟡 PARTIAL
6. **Coverage-Guided Generation** - 🟡 PARTIAL

### 📋 Remaining Work

This roadmap now focuses on **enhancements to existing features** and **new advanced capabilities**.

---

## Table of Contents

1. [Completed: More Built-in Generators](#1-completed-more-built-in-generators)
2. [Completed: Enhanced Shrinking Strategies](#2-completed-enhanced-shrinking-strategies)
3. [Remaining: Integration with More Test Frameworks](#3-remaining-integration-with-more-test-frameworks)
4. [Completed: Property Test Replay and Persistence](#4-completed-property-test-replay-and-persistence)
5. [Remaining: Advanced Coverage-Guided Generation](#5-remaining-advanced-coverage-guided-generation)
6. [Completed: Stateful Property Testing DSL](#6-completed-stateful-property-testing-dsl)
7. [New: Advanced Stateful Testing Features](#7-new-advanced-stateful-testing-features)
8. [New: Procedural Macros for Stateful Testing](#8-new-procedural-macros-for-stateful-testing)

---

## 1. Completed: More Built-in Generators

### ✅ Status: COMPLETE

Implemented in **protest-extras** package.

### What Was Delivered

#### Network/Web Generators ✅
```rust
pub struct IpAddressGenerator;  // IPv4/IPv6
pub struct UrlGenerator;        // Valid URLs
pub struct EmailGenerator;      // RFC-compliant emails
pub struct JsonGenerator;       // Valid JSON
```

#### Date/Time Generators ✅
```rust
pub struct DateTimeGenerator;   // Dates within ranges
pub struct DurationGenerator;   // Time durations
pub struct TimestampGenerator;  // Unix timestamps
```

#### Domain-Specific Generators ✅
```rust
pub struct UuidGenerator;       // UUIDs (v4)
pub struct Base64Generator;     // Valid base64 strings
pub struct HexGenerator;        // Hexadecimal strings
pub struct PathGenerator;       // Valid file system paths
```

#### Complex Collection Generators ✅
```rust
pub struct NonEmptyVecGenerator<T>;  // Ensures vec.len() >= 1
pub struct SortedVecGenerator<T>;    // Pre-sorted vectors
pub struct UniqueVecGenerator<T>;    // No duplicates
```

#### Constrained Numeric Generators ✅
```rust
pub struct PositiveIntGenerator<T>;
pub struct EvenNumberGenerator<T>;
pub struct PrimeNumberGenerator;
pub struct PercentageGenerator;  // 0.0..=100.0
```

#### Text Generators ✅
```rust
pub struct AlphabeticGenerator;     // Only letters
pub struct AlphanumericGenerator;   // Letters + numbers
pub struct IdentifierGenerator;     // Valid Rust identifiers
pub struct SentenceGenerator;       // Realistic sentences
```

---

## 2. Completed: Enhanced Shrinking Strategies

### ✅ Status: COMPLETE

Implemented in **protest-extras** package.

### What Was Delivered

#### Smart Shrinking with Invariants ✅
```rust
pub trait SmartShrink {
    fn shrink_preserving<F>(&self, invariant: F) -> Box<dyn Iterator<Item = Self>>
    where F: Fn(&Self) -> bool;
}
```

#### Delta Debugging ✅
```rust
pub struct DeltaDebugShrinker<T> {
    // Binary search through collections to find minimal failing subset
}
```

#### Targeted Shrinking ✅
```rust
pub struct TargetedShrinker<T> {
    target_value: T,  // Shrink toward specific value
}
```

### Usage Example
```rust
use protest_extras::shrinking::*;

let vec = vec![1, 3, 5, 7, 9];
let shrunk = vec.shrink_preserving(|v| {
    v.windows(2).all(|w| w[0] <= w[1])  // Keep sorted
});
```

---

## 3. Remaining: Integration with More Test Frameworks

### 🟡 Status: PARTIAL (Native #[test] and Tokio work)

### What's Done ✅

1. **Native #[test] Integration** ✅
2. **Tokio Async Integration** ✅

### What's Remaining 📋

#### 1. Criterion (Benchmarking) Integration
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

**Priority:** Medium
**Complexity:** Low
**Benefit:** Combine property testing with performance benchmarks

#### 2. Proptest Migration Helper
```rust
use protest::compat::proptest;

// Automatically convert proptest strategies to Protest generators
let strategy = any::<u32>().prop_map(|x| x * 2);
let generator = protest::from_proptest(strategy);
```

**Priority:** Low
**Complexity:** Medium
**Benefit:** Easy migration for existing proptest users

#### 3. QuickCheck Compatibility Layer
```rust
use protest::compat::quickcheck;

impl quickcheck::Arbitrary for MyType {
    fn arbitrary(g: &mut Gen) -> Self {
        MyType::auto_generator().generate(g, &config)
    }
}
```

**Priority:** Low
**Complexity:** Low
**Benefit:** Interop with existing QuickCheck code

#### 4. Insta (Snapshot Testing) Integration
```rust
use protest::insta::PropertySnapshot;

#[property_test]
fn test_serialization_format(data: MyData) {
    let json = serde_json::to_string(&data).unwrap();

    if data.is_edge_case() {
        insta::assert_snapshot!(json);
    }
}
```

**Priority:** Medium
**Complexity:** Medium
**Benefit:** Combine property testing with snapshot testing

---

## 4. Completed: Property Test Replay and Persistence

### ✅ Status: COMPLETE

Implemented across **protest**, **protest-cli**, and **protest-stateful**.

### What Was Delivered

#### Seed Persistence ✅
```rust
#[property_test]
fn test_something(x: i32) {
    // Automatically saves failing seeds
    // Next run replays with same seed first
}
```

#### Failure Case Database ✅
```rust
// Saved to: .protest/failures/test_parser/case_001.json
```

#### Regression Test Generation ✅
```rust
// CLI command generates permanent test files
protest generate my_test
```

#### Corpus Management ✅
```rust
pub struct TestCorpus {
    interesting_cases: Vec<TestCase>,
}
```

#### CLI Tool ✅
```bash
protest list              # List all failures
protest show test_name    # Show failure details
protest clean test_name   # Remove failures
protest generate          # Generate regression tests
protest stats            # Show statistics
```

---

## 5. Remaining: Advanced Coverage-Guided Generation

### 🟡 Status: PARTIAL (Basic corpus building complete)

### What's Done ✅

**Basic Coverage-Guided Corpus Building** ✅
```rust
pub struct CoverageCorpus {
    // Tracks unique execution paths
    // Builds corpus of interesting inputs
}
```

### What's Remaining 📋

#### 1. LLVM Coverage Integration

```rust
pub struct LLVMCoverageGuided {
    // Uses LLVM sanitizer coverage hooks
    // Tracks basic blocks, edges, comparisons
}

#[property_test(coverage = "llvm")]
fn test_parser(input: String) {
    // Automatically uses LLVM coverage feedback
    parse(input);
}
```

**Priority:** High
**Complexity:** High
**Benefit:** Industry-standard coverage instrumentation

#### 2. Energy Scheduling

```rust
pub struct EnergyScheduler {
    // Spend more iterations on inputs that find new coverage
    energy_per_input: HashMap<InputId, f64>,
}
```

**Priority:** Medium
**Complexity:** Medium
**Benefit:** More efficient input generation

#### 3. Advanced Mutations

```rust
pub struct CoverageGuidedMutator {
    // Intelligent mutations based on coverage feedback
    // Bit flips, arithmetic operations, boundary values
}
```

**Priority:** High
**Complexity:** High
**Benefit:** Better exploration of input space

#### 4. Custom Coverage Metrics

```rust
#[property_test(
    coverage_guided = true,
    metrics = [BranchCoverage, PathCoverage, DataFlowCoverage]
)]
fn test_with_multiple_metrics(input: Input) {
    // Track multiple coverage dimensions
}
```

**Priority:** Low
**Complexity:** High
**Benefit:** More precise coverage tracking

---

## 6. Completed: Stateful Property Testing DSL

### ✅ Status: COMPLETE

Implemented in **protest-stateful** package.

### What Was Delivered

#### Core Stateful Testing ✅
```rust
pub struct StatefulTest<State, Op> { /* ... */ }
pub trait Operation { /* ... */ }
pub struct OperationSequence<Op> { /* ... */ }
```

#### Model-Based Testing ✅
```rust
pub trait Model {
    type SystemState;
    type Operation;
    fn execute_model(&mut self, op: &Self::Operation);
    fn matches(&self, system: &Self::SystemState) -> bool;
}
```

#### Temporal Properties ✅
```rust
pub struct Eventually<State, F> { /* ... */ }
pub struct Always<State, F> { /* ... */ }
pub struct Never<State, F> { /* ... */ }
pub struct LeadsTo<State, F1, F2> { /* ... */ }
```

#### Concurrent Testing ✅
```rust
pub trait ConcurrentOperation: Operation + Send + Sync { /* ... */ }
pub fn run_concurrent<Op>(...) -> Result<Op::State, ConcurrentTestFailure>
```

---

## 7. New: Advanced Stateful Testing Features

### 📋 Status: NOT STARTED

These are enhancements to the existing protest-stateful package.

### 7.1 Advanced Shrinking for Operation Sequences

#### Delta Debugging Integration ✅ (Basic) → 🚧 (Advanced)

**Current:**
```rust
let shrunk = sequence.shrink();  // Basic size reduction
```

**Planned:**
```rust
let shrinker = DeltaDebugSequenceShrinker::new(sequence);
let minimal = shrinker.minimize_preserving_failure(test_fn);
// Finds minimal subsequence that still fails
```

**Priority:** High
**Complexity:** Medium
**Benefit:** Much faster debugging with minimal failing sequences

#### Smart Shrinking that Preserves Invariants

```rust
let test = StatefulTest::new(initial_state)
    .invariant("balance_positive", |s| s.balance > 0)
    .shrinking_strategy(SmartSequenceShrinking {
        preserve_invariants: true,
        preserve_preconditions: true,
    });

// Shrinking will only produce sequences that:
// 1. Still fail the property
// 2. Maintain all invariants
// 3. Respect all preconditions
```

**Priority:** High
**Complexity:** High
**Benefit:** More meaningful minimal counterexamples

### 7.2 Actual Linearizability Verification

#### Current State
```rust
let config = ConcurrentConfig {
    check_linearizability: true,  // Currently stubbed
};
```

#### Planned Implementation

**History-Based Linearizability Checking:**
```rust
pub struct LinearizabilityChecker<Op> {
    history: Vec<HistoryEvent<Op>>,
}

pub enum HistoryEvent<Op> {
    Invoke { thread_id: usize, op: Op, time: Instant },
    Return { thread_id: usize, result: OpResult, time: Instant },
}

impl<Op> LinearizabilityChecker<Op> {
    /// Check if concurrent history is linearizable
    pub fn check_linearizable(&self) -> Result<LinearOrder, NonLinearizableError> {
        // Wing & Gong algorithm or similar
    }
}
```

**Usage:**
```rust
#[test]
fn test_concurrent_queue_linearizability() {
    let checker = LinearizabilityChecker::new();

    let config = ConcurrentConfig {
        thread_count: 4,
        check_linearizability: true,
        history_checker: Some(checker),
    };

    let result = run_concurrent(initial, operations, config);

    // Automatically verifies linearizability
    // Reports violations with counterexample
    assert!(result.is_ok());
}
```

**Priority:** Very High
**Complexity:** Very High
**Benefit:** Critical for verifying concurrent data structures

#### Visualization of Non-Linearizable Histories

```rust
if let Err(e) = result {
    println!("{}", e.visualize());
    // Outputs:
    // Thread 1: Enqueue(1) |------|
    // Thread 2:             Enqueue(2) |-----|
    // Thread 3:                  Dequeue() -> 2  |-----|  ❌ Not linearizable!
    // Thread 4:                                   Dequeue() -> 1 |-----|
    //
    // Violation: Dequeue returned 2 before 1, but 1 was enqueued first
}
```

**Priority:** Medium
**Complexity:** Medium
**Benefit:** Easy debugging of concurrency issues

---

## 8. New: Procedural Macros for Stateful Testing

### 📋 Status: NOT STARTED

Create a new **protest-stateful-derive** package.

### 8.1 `stateful_test!` Procedural Macro

**Goal:** Reduce boilerplate for defining stateful tests.

**Current Approach:**
```rust
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
```

**With Macro:**
```rust
stateful_test! {
    name: stack_operations,
    state: Stack,

    operations {
        Push(value: i32) {
            execute: |state| state.items.push(value),
            precondition: |_state| true,
        },

        Pop {
            execute: |state| { state.items.pop(); },
            precondition: |state| !state.items.is_empty(),
        },
    }

    invariants {
        length_non_negative: |state| state.items.len() >= 0,
        capacity_reasonable: |state| state.items.capacity() < 10000,
    }
}
```

**Priority:** High
**Complexity:** High
**Benefit:** Much more ergonomic API

### 8.2 `#[derive(Operation)]` Macro

**Automatic Implementation:**
```rust
#[derive(Debug, Clone, Operation)]
#[operation(state = "Stack")]
enum StackOp {
    #[operation(execute = "state.items.push(value)")]
    Push { value: i32 },

    #[operation(
        execute = "state.items.pop()",
        precondition = "!state.items.is_empty()"
    )]
    Pop,
}
```

**Priority:** Medium
**Complexity:** High
**Benefit:** Less boilerplate

### 8.3 Automatic Operation Generation

**Goal:** Generate operations from type signatures.

```rust
#[derive(Debug, Clone)]
struct BankAccount {
    balance: i32,
}

// Automatically generate operations
#[derive(GenerateOperations)]
#[generate(
    deposit(amount: i32) -> precondition = "amount > 0",
    withdraw(amount: i32) -> precondition = "state.balance >= amount",
    check_balance() -> {},
)]
impl BankAccount {
    fn deposit(&mut self, amount: i32) {
        self.balance += amount;
    }

    fn withdraw(&mut self, amount: i32) {
        self.balance -= amount;
    }

    fn check_balance(&self) -> i32 {
        self.balance
    }
}
```

**Priority:** Medium
**Complexity:** Very High
**Benefit:** Zero boilerplate for simple cases

### 8.4 Weight-Based Operation Selection

```rust
stateful_test! {
    name: weighted_operations,
    state: MyState,

    operations {
        #[weight(10)]  // More common
        Read { /* ... */ },

        #[weight(2)]   // Less common
        Write { /* ... */ },

        #[weight(1)]   // Rare
        Delete { /* ... */ },
    }
}

// Or programmatically:
let generator = WeightedOperationGenerator::new()
    .with_weight(StackOp::Push(gen_int()), 10)
    .with_weight(StackOp::Pop, 3)
    .with_weight(StackOp::Clear, 1);
```

**Priority:** High
**Complexity:** Medium
**Benefit:** More realistic operation distributions

---

## Implementation Priority

### Immediate (Next Release)

1. **Linearizability Checking** - Critical for concurrent testing
2. **Advanced Sequence Shrinking** - Better debugging experience
3. **Weight-Based Operation Selection** - More realistic tests

### Short Term (1-2 Releases)

4. **`stateful_test!` Macro** - Improved ergonomics
5. **LLVM Coverage Integration** - Better coverage guidance
6. **Criterion Integration** - Property-based benchmarks

### Medium Term (3-6 Releases)

7. **Advanced Mutations** - Smarter input generation
8. **`#[derive(Operation)]`** - Less boilerplate
9. **Insta Integration** - Snapshot testing
10. **Energy Scheduling** - Efficient fuzzing

### Long Term (Future)

11. **Automatic Operation Generation** - Zero boilerplate
12. **Proptest Migration** - Ecosystem compatibility
13. **QuickCheck Compat** - Ecosystem compatibility
14. **Custom Coverage Metrics** - Advanced use cases

---

## Contributing

We welcome contributions to any of these roadmap items!

### How to Contribute

1. Check existing issues for the feature
2. Open a discussion issue to align on approach
3. Implement with comprehensive tests
4. Add documentation and examples
5. Submit PR

For major features (especially procedural macros and coverage integration), please discuss the design first.

---

## Package Organization

- **protest** - Core library ✅
- **protest-derive** - Derive macros ✅
- **protest-extras** - Extra generators & shrinking ✅
- **protest-cli** - CLI tool ✅
- **protest-stateful** - Stateful testing DSL ✅
- **protest-stateful-derive** - Stateful macros 📋 (Future)
- **protest-coverage** - LLVM coverage integration 📋 (Future)

---

*Last updated: 2025-01-15*
