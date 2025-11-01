# Protest Roadmap

This document outlines the planned features and enhancements for the Protest property-based testing library.

## Quick Overview: Phased Development Plan

> **Note:** Each feature requires complete deliverables:
> Implementation + Tests + Docs + Examples + README updates
> See the [Feature Completion Checklist](#-feature-completion-checklist) below for details.

```
✅ Phase 1: Complete Stateful Testing (COMPLETED)
   ├─ ✅ Linearizability verification
   ├─ ✅ stateful_test! macro
   ├─ ✅ #[derive(Operation)] macro
   └─ ✅ Weight-based operation generation
   Package: protest-stateful-derive ✅

📅 Phase 2: Property-Based Benchmarking (NEXT)
   └─ Criterion integration
   Package: protest-criterion

📅 Phase 3: Snapshot Testing
   └─ Insta integration
   Package: protest-insta

📅 Phase 4: Migration Support
   └─ Proptest compatibility
   Package: protest-proptest-compat

📅 Phase 5: Coverage-Guided Fuzzing
   ├─ LLVM coverage integration
   ├─ Energy scheduling
   └─ Advanced mutations
   Package: TBD
```

## Project Status

### ✅ Previously Completed

- ✅ **Core Property Testing Framework** - Full QuickCheck-style testing
- ✅ **Comprehensive Generators** - 23+ generators in protest-extras
- ✅ **Enhanced Shrinking Strategies** - Advanced shrinking algorithms (protest-extras)
- ✅ **Property Test Replay and Persistence** - Seed persistence, failure database, CLI tool
- ✅ **Stateful Property Testing DSL** - Full state machine testing (protest-stateful)
- ✅ **Advanced Sequence Shrinking** - Delta debugging and smart shrinking (protest-stateful)
- ✅ **Basic Coverage-Guided Corpus Building** - Path tracking and corpus management

### ✅ Phase 1: Complete Stateful Testing - COMPLETED

**Goal:** Finish all stateful testing features before moving to integrations

**Completed Features:**

1. **✅ Linearizability Verification**
   - Wing & Gong backtracking algorithm
   - History tracking with timestamps
   - Sequential specification trait
   - Timeline and conflict visualization
   - 5 comprehensive tests + example
   - File: `protest-stateful/src/concurrent/linearizability.rs`

2. **✅ #[derive(Operation)] Macro**
   - Automatic Operation trait implementation
   - Attributes: `#[execute]`, `#[precondition]`, `#[weight]`, `#[description]`
   - Support for unit, unnamed, and named field variants
   - Zero-warning field binding
   - 5 integration tests + 4 doctests + example
   - File: `protest-stateful-derive/src/operation.rs`

3. **✅ stateful_test! Macro**
   - Declarative DSL for test configuration
   - Invariant integration
   - Config options (iterations, sequence length, seed)
   - Full documentation with examples
   - File: `protest-stateful-derive/src/stateful_test.rs`

4. **✅ Weight-Based Operation Generation**
   - `WeightedGenerator` for realistic operation frequencies
   - Automatic weight extraction from `#[weight(N)]` attributes
   - Statistical distribution analysis
   - 5 comprehensive tests + example
   - File: `protest-stateful/src/operations/generator.rs`

**Metrics:**
- 📦 New Package: `protest-stateful-derive`
- ✅ 588 tests passing (0 failures)
- ✅ 7 doctests passing (0 ignored)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- 📝 Complete documentation and examples

---

## Upcoming Work

### 📅 Phase 2: Criterion Integration - NEXT

**Goal:** Property-based benchmarking with Criterion

**Package:** `protest-criterion`

#### 2.1 Criterion Integration

**Goal:** Property-based benchmarking

```rust
use criterion::{criterion_group, Criterion};
use protest_criterion::*;

fn bench_sort_property(c: &mut Criterion) {
    c.bench_property("sort maintains elements", |v: Vec<i32>| {
        let mut sorted = v.clone();
        sorted.sort();
        sorted.len() == v.len()
    });
}

criterion_group!(benches, bench_sort_property);
```

**Features:**
- Integrate with Criterion benchmarking framework
- Generate diverse inputs for benchmarks
- Statistical analysis of property performance
- Regression detection for properties

**Priority:** High
**Complexity:** Medium
**Benefit:** Performance testing with property-based inputs

**Deliverables:**
- [ ] `protest-criterion` crate created
- [ ] Criterion trait integration
- [ ] Benchmark macros
- [ ] Statistical reporting
- [ ] Examples and documentation
- [ ] README updates

---

### 📅 Phase 3: Insta Snapshot Integration

**Goal:** Snapshot testing with property-based inputs

**Package:** `protest-insta`

#### 3.1 Insta Integration

```rust
use protest_insta::*;
use insta::assert_snapshot;

#[property_snapshot_test]
fn test_serialization(value: MyStruct) {
    let json = serde_json::to_string_pretty(&value).unwrap();
    assert_snapshot!(json);
}
```

**Features:**
- Generate diverse inputs for snapshot tests
- Automatic snapshot naming
- Regression detection
- Integration with insta's review workflow

**Priority:** Medium
**Complexity:** Low
**Benefit:** Visual regression testing with properties

**Deliverables:**
- [ ] `protest-insta` crate created
- [ ] Insta integration macros
- [ ] Snapshot management
- [ ] Examples and documentation
- [ ] README updates

---

### 📅 Phase 4: Proptest Compatibility

**Goal:** Migration path from Proptest

**Package:** `protest-proptest-compat`

#### 4.1 Proptest Compatibility Layer

```rust
// Drop-in replacement for proptest
use protest_proptest_compat::prelude::*;

proptest! {
    #[test]
    fn test_addition(a in 0..100i32, b in 0..100i32) {
        assert!(a + b >= a);
        assert!(a + b >= b);
    }
}
```

**Features:**
- Compatible with proptest macros
- Strategy compatibility
- Migration guide
- Side-by-side comparison

**Priority:** Medium
**Complexity:** High
**Benefit:** Easy migration for existing proptest users

**Deliverables:**
- [ ] `protest-proptest-compat` crate created
- [ ] proptest! macro compatibility
- [ ] Strategy adapters
- [ ] Migration guide
- [ ] Examples and documentation
- [ ] README updates

---

### 📅 Phase 5: Advanced Coverage-Guided Generation

**Goal:** AFL-style coverage-guided fuzzing

**Package:** TBD (architecture discussion needed)

#### 5.1 LLVM Coverage Integration

**Status:** 🔴 NOT STARTED - Requires architecture discussion

**Potential approaches:**
1. Separate crate using LLVM instrumentation
2. Integration with existing tools (AFL, libFuzzer)
3. Custom instrumentation layer

**Questions to answer:**
- Should this be a separate crate?
- Compile-time vs runtime instrumentation?
- Platform support (Linux/macOS/Windows)?
- Integration with existing fuzzing infrastructure?

**Priority:** High
**Complexity:** Very High
**Benefit:** Deep bug finding

#### 5.2 Energy Scheduling

Track which inputs discover new paths and prioritize them:

```rust
pub struct EnergyScheduler {
    energy_map: HashMap<InputId, f64>,
}

// More energy = more mutations, more testing time
```

**Priority:** Medium
**Complexity:** Medium

#### 5.3 Advanced Input Mutations

Smarter mutations based on coverage feedback:

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
**Complexity:** Very High

---

## Implementation Timeline

### ✅ Completed - Phase 1
- ✅ Linearizability verification with Wing & Gong algorithm
- ✅ History visualization for concurrent operations
- ✅ `#[derive(Operation)]` procedural macro
- ✅ `stateful_test!` declarative macro
- ✅ Weight-based operation generation with `WeightedGenerator`
- ✅ `protest-stateful-derive` package created
- ✅ Comprehensive tests (588 tests passing)
- ✅ Complete documentation and examples

### 📅 Next Up - Phase 2
- [ ] protest-criterion crate for property-based benchmarking

### 📅 Future Phases
- [ ] Phase 3: protest-insta for snapshot testing
- [ ] Phase 4: protest-proptest-compat for migration
- [ ] Phase 5: Advanced coverage-guided fuzzing

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
- [ ] Rustdoc comments on public APIs
- [ ] Module-level docs with examples
- [ ] Doc tests written and passing
- [ ] At least one runnable example

### Integration
- [ ] Package README updated
- [ ] Root README updated
- [ ] CHANGELOG.md updated
- [ ] Feature flag added (if applicable)

### Quality
- [ ] No compiler warnings
- [ ] Clippy passes
- [ ] Formatted with rustfmt
- [ ] Documentation builds without warnings
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on contributing to the Protest roadmap.

## Questions or Suggestions?

Open an issue on GitHub to discuss:
- Feature priorities
- Implementation approaches
- New feature ideas
- Architecture decisions (especially for Phase 5)
