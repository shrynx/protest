# Protest Roadmap

This document outlines the planned features and enhancements for the Protest property-based testing library.

## Quick Overview: Phased Development Plan

> **Note:** Each feature requires complete deliverables:
> Implementation + Tests + Docs + Examples + README updates
> See the [Feature Completion Checklist](#-feature-completion-checklist) below for details.

```
✅ v0.1.1: Core + Advanced Shrinking (COMPLETED)
   ├─ Core property testing
   ├─ Comprehensive generators (protest-extras)
   ├─ Stateful testing (protest-stateful)
   ├─ Delta debugging for sequences
   └─ Smart shrinking with constraints

⚡ v0.2.0: Complete Stateful Testing (IN PROGRESS - Phase 1)
   ├─ Linearizability verification
   ├─ stateful_test! macro
   ├─ #[derive(Operation)] macro
   └─ Weight-based operation generation
   Package: protest-stateful-derive

📅 v0.3.0: Property-Based Benchmarking (Phase 2)
   └─ Criterion integration
   Package: protest-criterion

📅 v0.4.0: Snapshot Testing (Phase 3)
   └─ Insta integration
   Package: protest-insta

📅 v0.5.0: Migration Support (Phase 4)
   └─ Proptest compatibility
   Package: protest-proptest-compat

📅 v0.6.0+: Coverage-Guided Fuzzing (Phase 5)
   ├─ LLVM coverage integration
   ├─ Energy scheduling
   └─ Advanced mutations
   Package: TBD
```

## Project Status

### ✅ Completed (v0.1.1)

- ✅ **Core Property Testing Framework** - Full QuickCheck-style testing
- ✅ **Comprehensive Generators** - 23+ generators in protest-extras
- ✅ **Enhanced Shrinking Strategies** - Advanced shrinking algorithms (protest-extras)
- ✅ **Property Test Replay and Persistence** - Seed persistence, failure database, CLI tool
- ✅ **Stateful Property Testing DSL** - Full state machine testing (protest-stateful)
- ✅ **Advanced Sequence Shrinking** - Delta debugging and smart shrinking (protest-stateful)
- ✅ **Basic Coverage-Guided Corpus Building** - Path tracking and corpus management

### ⚡ Phase 1: Complete Stateful Testing (v0.2.0) - IN PROGRESS

**Goal:** Finish all stateful testing features before moving to integrations

**Next Up:**
1. Linearizability verification for concurrent systems
2. Procedural macros for better ergonomics
3. Weight-based operation generation

---

## Table of Contents

1. [Integration with Test Frameworks](#1-integration-with-test-frameworks)
2. [Advanced Coverage-Guided Generation](#2-advanced-coverage-guided-generation)
3. [Advanced Stateful Testing Features](#3-advanced-stateful-testing-features)
4. [Procedural Macros for Stateful Testing](#4-procedural-macros-for-stateful-testing)

---

## 1. Integration with Test Frameworks

**Status:** 🟡 PARTIAL - Basic support exists, needs expansion

### 1.1 Criterion Integration

**Goal:** Property-based benchmarking

```rust
use criterion::{criterion_group, Criterion};
use protest::prelude::*;

fn bench_sort_property(c: &mut Criterion) {
    c.bench_property("sort maintains elements", |v: Vec<i32>| {
        let mut sorted = v.clone();
        sorted.sort();
        sorted.len() == v.len()
    });
}

criterion_group!(benches, bench_sort_property);
```

**Priority:** Medium
**Complexity:** Low
**Benefit:** Detect performance regressions via properties

### 1.2 Proptest Compatibility Layer

**Goal:** Support proptest strategies in protest

```rust
use protest::proptest_compat::*;

#[property_test]
fn test_with_proptest_strategy(x: impl Strategy<Value = i32>) {
    // Use existing proptest strategies in protest tests
}
```

**Priority:** Low
**Complexity:** Medium
**Benefit:** Easier migration from proptest

### 1.3 Insta Snapshot Integration

**Goal:** Combine property testing with snapshot testing

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

## 2. Advanced Coverage-Guided Generation

**Status:** 🟡 PARTIAL - Basic corpus building done, advanced instrumentation remains

### 2.1 LLVM Coverage Integration

**Goal:** Industry-standard coverage instrumentation

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

**Implementation:**
- Use `llvm-cov` instrumentation
- Track basic block coverage
- Track edge coverage
- Track comparison feedback (cmp hooks)

**Priority:** High
**Complexity:** High
**Benefit:** Industry-standard coverage instrumentation

### 2.2 Energy Scheduling

**Goal:** Prioritize inputs based on coverage potential

```rust
pub struct EnergyScheduler {
    input_energy: HashMap<InputId, f64>,
}

// Inputs that discover more coverage get more energy
// More energy = more mutations, more testing time
```

**Algorithm:**
- Assign energy based on novelty
- Reward inputs that find new coverage
- Deprioritize saturated inputs

**Priority:** Medium
**Complexity:** Medium
**Benefit:** Faster path to deep coverage

### 2.3 Advanced Input Mutations

**Goal:** Smarter mutations based on coverage

```rust
pub struct CoverageMutator {
    comparison_feedback: Vec<ComparisonTrace>,
}

// If code compares: if x == 42
// Mutator tries: x = 42, x = 41, x = 43
```

**Mutations:**
- Dictionary-based mutations
- Comparison-guided mutations
- Structural mutations (for complex types)
- AFL-style bit flips, arithmetic, etc.

**Priority:** High
**Complexity:** High
**Benefit:** Much deeper code coverage

---

## 3. Advanced Stateful Testing Features

**Status:** 🟡 PARTIAL - Advanced shrinking complete, linearizability remains

### 3.1 Advanced Sequence Shrinking ✅ COMPLETED

#### Delta Debugging for Sequences ✅

**Implementation:** `DeltaDebugSequenceShrinker` in protest-stateful

Uses binary search to find minimal failing subsequences in O(n log n) tests:

```rust
use protest_stateful::operations::shrinking::*;

let shrinker = DeltaDebugSequenceShrinker::new(sequence);
let (minimal, test_count) = shrinker.minimize_with_stats(|seq| {
    test.run(seq).is_err()
});
// Finds minimal subsequence that still fails
```

**Features:**
- Binary search over subsequences
- Chunk-based reduction (halves, thirds, etc.)
- Individual operation removal
- Statistics tracking (test count)

**Result:** ✅ Implemented with comprehensive tests and examples

#### Smart Shrinking that Preserves Invariants ✅

**Implementation:** `SmartSequenceShrinking` in protest-stateful

Shrink while maintaining invariants and preconditions:

```rust
let config = SmartSequenceShrinking::new()
    .preserve_invariants(true)
    .preserve_preconditions(true)
    .max_attempts(1000);

let minimal = config.shrink(&sequence, &initial_state, |seq| {
    test.run(seq).is_err()
});

// Shrinking produces sequences that:
// 1. Still fail the property
// 2. Maintain all invariants
// 3. Respect all preconditions
```

**Features:**
- Configurable constraint preservation
- Precondition validation during shrinking
- Max attempts limiting
- Statistics tracking

**Result:** ✅ Implemented with comprehensive tests and examples

### 3.2 Linearizability Verification

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
    // Thread 3:                  Dequeue() -> 2  |-----|  ❌
    // Thread 4:                                   Dequeue() -> 1 |-----|
    //
    // Violation: Dequeue returned 2 before 1, but 1 was enqueued first
}
```

**Priority:** Medium
**Complexity:** Medium
**Benefit:** Easy debugging of concurrency issues

---

## 4. Procedural Macros for Stateful Testing

**Status:** 📋 NOT STARTED - Would create new `protest-stateful-derive` package

### 4.1 `stateful_test!` Procedural Macro

**Goal:** Reduce boilerplate for defining stateful tests

**Current Approach (Verbose):**
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

**With Macro (Concise):**
```rust
stateful_test! {
    name: stack_operations,
    state: Stack,

    operations {
        Push(value: i32) {
            execute: |state| state.items.push(value),
        }

        Pop {
            precondition: |state| !state.items.is_empty(),
            execute: |state| { state.items.pop(); },
        }
    }

    invariants {
        length_non_negative: |state| state.items.len() >= 0,
        empty_when_zero: |state| {
            state.items.is_empty() == (state.items.len() == 0)
        },
    }
}
```

**Priority:** Medium
**Complexity:** Medium
**Benefit:** 50% less boilerplate code

### 4.2 `#[derive(Operation)]` Macro

**Goal:** Auto-implement Operation trait

```rust
#[derive(Debug, Clone, Operation)]
#[operation(state = "Stack")]
enum StackOp {
    #[execute(state.items.push(value))]
    Push { value: i32 },

    #[execute(state.items.pop())]
    #[precondition(!state.items.is_empty())]
    Pop,
}
```

**Priority:** Low
**Complexity:** Medium
**Benefit:** Even less boilerplate

### 4.3 Property-Based Operation Generation

**Goal:** Automatically generate operations from types

```rust
#[derive(Debug, Clone, OperationGenerator)]
enum StackOp {
    #[weight(5)]  // More likely to be generated
    Push(#[gen(range(0..100))] i32),

    #[weight(2)]  // Less likely
    Pop,

    #[weight(1)]  // Rare
    Clear,
}

// Auto-generates:
impl Generator<StackOp> for StackOpGenerator { ... }
```

**Features:**
- Weight-based operation selection
- Automatic generator derivation
- Custom generation strategies per field

**Priority:** Medium
**Complexity:** High
**Benefit:** Automatic test generation

---

## Implementation Timeline

### Completed (v0.1.1)
- ✅ Delta debugging for sequence shrinking
- ✅ Smart shrinking preserving invariants

---

## 📋 Feature Completion Checklist

**For every feature, ensure all deliverables are complete:**

### Required Deliverables
- ✅ **Implementation** - Working code with proper error handling
- ✅ **Unit Tests** - Comprehensive test coverage (aim for >80%)
- ✅ **Integration Tests** - Real-world usage scenarios
- ✅ **Documentation**:
  - Rustdoc comments on all public APIs
  - Module-level documentation with examples
  - Usage examples in doc comments
- ✅ **Examples** - At least one runnable example demonstrating the feature
- ✅ **README Updates**:
  - Update package-specific README (e.g., `protest-stateful/README.md`)
  - Update root `README.md` to mention the new feature
  - Add feature to feature list and quick start if applicable
- ✅ **CHANGELOG** - Document changes in CHANGELOG.md

### Quality Standards
- All tests pass (`cargo test`)
- No compiler warnings (`cargo clippy`)
- Proper formatting (`cargo fmt`)
- Documentation builds without warnings (`cargo doc`)
- Examples run successfully (`cargo run --example <name>`)

### Feature Implementation Template

Use this checklist when implementing each feature:

```markdown
## Feature: [Feature Name]

### Implementation
- [ ] Core implementation complete
- [ ] Error handling implemented
- [ ] Public API finalized

### Testing
- [ ] Unit tests written (>80% coverage)
- [ ] Integration tests written
- [ ] Edge cases tested
- [ ] All tests passing

### Documentation
- [ ] Public API has rustdoc comments
- [ ] Module-level docs with examples
- [ ] Usage examples in doc comments
- [ ] Doc tests passing

### Examples
- [ ] At least one runnable example created
- [ ] Example demonstrates key features
- [ ] Example documented with comments
- [ ] Example runs without errors

### README Updates
- [ ] Package README updated (if applicable)
- [ ] Root README.md updated
- [ ] Feature added to feature list
- [ ] Quick start updated (if needed)

### CHANGELOG
- [ ] Changes documented in CHANGELOG.md
- [ ] Breaking changes noted (if any)
- [ ] Migration guide written (if needed)

### Quality Checks
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied
- [ ] `cargo doc` builds without warnings
- [ ] Examples run successfully
```

---

### Phase 1: Complete Stateful Testing (v0.2.0)
**Goal:** Finish all stateful testing features and ergonomics

1. ⚡ **Linearizability Verification** - Critical for concurrent data structure testing
   - **Deliverables:** Implementation + Tests + Docs + Example + README updates

2. ⚡ **`stateful_test!` Procedural Macro** - Reduce boilerplate, improve DX
   - **Deliverables:** Macro implementation + Tests + Docs + Example + README updates

3. ⚡ **`#[derive(Operation)]` Macro** - Auto-implement Operation trait
   - **Deliverables:** Derive macro + Tests + Docs + Example + README updates

4. ⚡ **Weight-based Operation Generation** - Control operation frequency in tests
   - **Deliverables:** Generator implementation + Tests + Docs + Example + README updates

**Package:** `protest-stateful` + new `protest-stateful-derive` crate for macros

### Phase 2: Criterion Integration (v0.3.0)
**Goal:** Property-based benchmarking

5. 📅 **protest-criterion** - New crate for Criterion integration
   - **Deliverables:**
     - Criterion trait implementations
     - Property-based benchmark macros
     - Comprehensive tests
     - Benchmark examples
     - README with quick start guide
     - Root README update with benchmarking section

**Package:** New `protest-criterion` crate

### Phase 3: Snapshot Testing Integration (v0.4.0)
**Goal:** Combine property testing with snapshot testing

6. 📅 **protest-insta** - New crate for Insta integration
   - **Deliverables:**
     - Insta integration layer
     - Property + snapshot macros
     - Tests with snapshot fixtures
     - Examples showing edge case discovery
     - README with usage patterns
     - Root README update with snapshot testing section

**Package:** New `protest-insta` crate

### Phase 4: Proptest Migration Path (v0.5.0)
**Goal:** Easy migration from proptest

7. 📅 **protest-proptest-compat** - New crate for proptest compatibility
   - **Deliverables:**
     - Strategy adapter implementations
     - Conversion utilities
     - Migration guide documentation
     - Side-by-side comparison examples
     - README with migration checklist
     - Root README update mentioning compatibility

**Package:** New `protest-proptest-compat` crate

### Phase 5: Advanced Coverage-Guided Generation (v0.6.0+)
**Goal:** Deep coverage instrumentation and intelligent fuzzing

8. 📅 **LLVM Coverage Integration** - Industry-standard instrumentation
   - **Deliverables:** TBD based on architecture decisions

9. 📅 **Energy Scheduling** - Prioritize high-value inputs
   - **Deliverables:** TBD based on architecture decisions

10. 📅 **Advanced Input Mutations** - Comparison-guided, dictionary-based mutations
    - **Deliverables:** TBD based on architecture decisions

**Package:** TBD - To be determined based on architecture needs

---

## Package Organization

### Current Packages (v0.1.1)
1. ✅ **protest** - Core property testing framework
2. ✅ **protest-derive** - Procedural macros for core
3. ✅ **protest-extras** - Extended generators and shrinking strategies
4. ✅ **protest-cli** - Command-line tools for test management
5. ✅ **protest-stateful** - Stateful property testing with advanced shrinking

### Planned Packages (In Order)

#### Phase 1: Stateful Testing (v0.2.0)
6. 📦 **protest-stateful-derive** - Procedural macros for stateful testing
   - `stateful_test!` macro
   - `#[derive(Operation)]` macro
   - Operation generator macros

#### Phase 2: Benchmarking (v0.3.0)
7. 📦 **protest-criterion** - Criterion integration for property-based benchmarks

#### Phase 3: Snapshot Testing (v0.4.0)
8. 📦 **protest-insta** - Insta snapshot integration

#### Phase 4: Migration Support (v0.5.0)
9. 📦 **protest-proptest-compat** - Proptest compatibility layer

#### Phase 5: Coverage (v0.6.0+)
10. 📦 **protest-coverage** - LLVM coverage instrumentation (TBD)

---

## Current Development Focus

### Active Work (Phase 1: v0.2.0)
We are currently completing all stateful testing features:

1. **Next Up:** Linearizability Verification
2. **Then:** `stateful_test!` procedural macro
3. **Then:** `#[derive(Operation)]` macro
4. **Then:** Weight-based operation generation

### Contributing

Contributions are welcome! The current focus is Phase 1, but contributions to any area are appreciated.

#### High Impact Areas
- **Linearizability Verification** - Critical for concurrent testing
- **Procedural Macros** - Improve developer experience
- **Additional Generators** - Expand protest-extras
- **Documentation & Examples** - Always valuable

#### Getting Started
- Review the phase plan above
- Check existing implementations for patterns
- Open an issue to discuss major features
- Start with documentation or examples for first contributions

---

## Summary

**Current State:** Production-ready core with advanced features (v0.1.1)

- ✅ Core property testing framework
- ✅ Comprehensive generators (protest-extras)
- ✅ Stateful testing with advanced shrinking
- ✅ CLI tools for test management
- ✅ Delta debugging and smart shrinking

**Development Strategy:**

The roadmap follows a **phased approach**, completing one major feature area before moving to the next:

1. **Phase 1 (v0.2.0):** Complete stateful testing - Add linearizability verification and ergonomic macros
2. **Phase 2 (v0.3.0):** Property-based benchmarking with Criterion
3. **Phase 3 (v0.4.0):** Snapshot testing integration with Insta
4. **Phase 4 (v0.5.0):** Migration support from Proptest
5. **Phase 5 (v0.6.0+):** Advanced coverage-guided generation

**Philosophy:**
- Ship complete, polished features one at a time
- Each phase delivers standalone value
- Build on proven foundations
- Respond to user feedback between phases

The library is **production-ready today**, with each upcoming phase adding powerful new capabilities for specific use cases.
