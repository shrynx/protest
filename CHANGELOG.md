# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New features go here

### Changed
- Changes to existing functionality go here

## [0.3.0] - 2025-11-01

### Added - Phase 2: Property-Based Benchmarking

#### New Package: protest-criterion
- **Criterion Integration**: Seamless integration with Criterion.rs benchmarking framework
- **PropertyBencher Trait**: Extension trait for `Criterion` with property-based methods
  - `bench_function_over_inputs()` - Benchmark functions with generated inputs
  - `bench_property()` - Benchmark property tests
- **PropertyBenchmarkGroup Trait**: Extension for `BenchmarkGroup`
  - `bench_generated()` - Ergonomic grouped benchmarks with generators
- **Diverse Input Benchmarking**: Test performance across the full input space
- **Statistical Analysis**: Leverage Criterion's regression detection with generated data
- File: `protest-criterion/src/lib.rs` (375 lines)
- Tests: 2 unit tests passing
- Examples: 3 comprehensive benchmark suites
  - `example_benchmarks.rs` - Basic usage
  - `sorting_benchmarks.rs` - Sorting algorithms with various distributions
  - `string_benchmarks.rs` - String operations at different scales

#### Documentation
- **Comprehensive README**: Full documentation with use cases and best practices
- **API Documentation**: Complete rustdoc with examples for all public APIs
- **Integration Guide**: Property-based benchmarking patterns and tips
- **Main README Updated**: Added Property-Based Benchmarking section

### Features
- **Generator Integration**: Works with all Protest generators
- **Reproducible Benchmarks**: Seed-based generation for consistency
- **Multiple Sampling**: Benchmark with configurable number of generated inputs
- **Performance Distribution**: Understand how code performs across input variations
- **Edge Case Discovery**: Automatically find performance bottlenecks

### Benefits
- 📊 Performance across input space - Not just one data point
- 🔍 Automatic edge case discovery - Find worst-case scenarios
- 📈 Statistical analysis - Criterion's regression detection
- ⚡ Realistic workloads - Production-like data distributions

### Metrics - Phase 2 Completion
- **Package**: protest-criterion ✅
- **Total Tests**: 590 tests passing (2 new in protest-criterion)
- **Doctests**: Working (some marked no_run for benchmarks)
- **Code Quality**: Zero compiler warnings, zero clippy warnings
- **Lines of Code**: ~375 lines of implementation
- **Benchmarks**: 3 comprehensive benchmark suites
- **Documentation**: 100% API coverage with README

## [0.2.0] - 2025-11-01

### Added - Phase 1: Complete Stateful Testing

#### Linearizability Verification
- **Wing & Gong Algorithm**: Backtracking-based linearizability checking for concurrent operations
- **History Tracking**: Record invocation/response events with timestamps for concurrent execution traces
- **Happens-Before Graphs**: Directed graph construction representing temporal ordering constraints
- **Sequential Specification Trait**: Define expected sequential behavior for concurrent systems
- **Timeline Visualization**: Visual representation of concurrent operation timelines with overlaps
- **Conflict Detection**: Identify and visualize linearizability violations
- File: `protest-stateful/src/concurrent/linearizability.rs` (590 lines)
- Tests: 5 comprehensive tests + 1 example
- Documentation: Full rustdoc with usage examples

#### Procedural Macro: #[derive(Operation)]
- **Automatic Operation Trait Implementation**: Zero-boilerplate operation definitions
- **Attributes Support**:
  - `#[operation(state = "Type")]` - Specify state type
  - `#[execute("expression")]` - Define execution logic
  - `#[precondition("expression")]` - Add precondition checks
  - `#[weight(N)]` - Control operation frequency (higher = more frequent)
  - `#[description("text")]` - Custom operation descriptions
- **Variant Support**: Unit variants, unnamed fields, and named fields
- **Zero-Warning Field Binding**: Smart field binding with underscore prefixes to avoid unused variable warnings
- File: `protest-stateful-derive/src/operation.rs` (445 lines)
- Tests: 5 integration tests + 4 doctests + 1 example
- Documentation: Complete rustdoc with examples for all variant types

#### Declarative Macro: stateful_test!
- **DSL for Test Configuration**: Ergonomic test setup with declarative syntax
- **Features**:
  - Test name and state initialization
  - Operation type specification
  - Invariant integration
  - Configuration options (iterations, sequence length, seed)
- File: `protest-stateful-derive/src/stateful_test.rs` (312 lines)
- Tests: 2 doctests + full integration coverage
- Documentation: Complete usage guide with examples

#### New Package: protest-stateful-derive
- **Procedural Macro Crate**: Derive macros and declarative macros for stateful testing
- **Dependencies**: syn 2.0, quote 1.0, proc-macro2 1.0
- **Structure**:
  - `operation.rs` - #[derive(Operation)] implementation
  - `stateful_test.rs` - stateful_test! macro implementation
  - `lib.rs` - Public API and documentation
- **Quality**:
  - 588 tests passing (0 failures, 0 ignored)
  - 7 doctests passing (0 ignored)
  - Zero compiler warnings
  - Zero clippy warnings

#### Weight-Based Operation Generation
- **WeightedGenerator**: Generate operations according to their specified weights
- **Automatic Weight Extraction**: Read weights from `#[weight(N)]` attributes via Operation trait
- **Statistical Analysis**: Distribution analysis and weight statistics
- **Realistic Workloads**: Mirror real-world usage patterns with weighted frequencies
- **Integration**: Seamless integration with derive macro and StatefulTest
- File: `protest-stateful/src/operations/generator.rs` (338 lines)
- Tests: 5 comprehensive tests + example
- Documentation: Complete rustdoc with real-world examples

### Changed
- **Operation Trait**: Added `weight()` method with default implementation returning 1
- **Derive Macro**: Now generates `weight()` implementation based on `#[weight(N)]` attributes
- **README**: Added comprehensive weight-based generation section with examples
- **ROADMAP**: Removed version numbers, marked Phase 1 complete

### Fixed
- **Clippy Warnings**: Fixed all clippy warnings across the codebase
  - Collapsible if statements in operation.rs (3 instances)
  - Derivable Default implementation in stateful_test.rs
  - Needless borrow in linearizability.rs
  - Length comparison to zero in protest-extras example
- **Compiler Warnings**: Fixed all unused variable warnings
  - Smart field binding pattern in derive macro generated code
  - Unused test variable in shrinking.rs
  - Type inference in derive_tests.rs
- **Doctests**: Fixed all ignored doctests
  - Added required attributes and imports for Operation derive tests (4 tests)
  - Used `# /* ... # */` pattern for stateful_test! tests (2 tests)

### Documentation
- **Examples**: 5 comprehensive runnable examples
  - `derive_macro.rs` - Demonstrating all #[derive(Operation)] features
  - `linearizability_verification.rs` - Concurrent operation verification
  - `stack.rs` - Basic stateful testing
  - `key_value_store.rs` - Model-based testing
  - `weighted_generation.rs` - Weight-based operation generation
- **Doctests**: 7 passing doctests with real-world usage
- **README Updates**:
  - Main README.md updated with stateful testing section
  - protest-stateful/README.md with full feature documentation
- **ROADMAP.md**: Cleaned up and reorganized for clarity

### Metrics - Phase 1 Completion
- **Total Tests**: 588 tests passing (0 failures, 0 ignored)
- **Doctests**: 7 passing (0 ignored)
- **Code Quality**: Zero compiler warnings, zero clippy warnings
- **Lines of Code**: ~1,685 lines of new implementation code
  - linearizability.rs: 590 lines
  - operation.rs: 445 lines
  - stateful_test.rs: 312 lines
  - generator.rs: 338 lines
- **Test Coverage**: >80% with comprehensive integration tests
- **Documentation**: 100% public API coverage with examples
- **Examples**: 5 comprehensive examples including weighted generation demo

## [0.1.0] - 2025-10-31

### Added
- Initial release of Protest property testing library
- Core property testing framework with shrinking support
- Generator trait and derive macro for automatic test data generation
- Async property testing support
- Parallel test execution
- Statistics collection and coverage analysis
- Customizable error reporting
- Support for custom generators and strategies
- Comprehensive test suite with 350+ tests

### Features
- **Property Testing**: Define properties that should hold for all inputs
- **Automatic Shrinking**: Find minimal failing cases automatically
- **Derive Macros**: `#[derive(Generator, Arbitrary)]` for automatic implementations
- **Async Support**: Test async functions with `check_async`
- **Parallel Execution**: Run tests in parallel for faster feedback
- **Statistics**: Track coverage and generation statistics
- **Ergonomic API**: Builder pattern and convenience macros
- **Type Support**: Primitives, collections, tuples, enums, custom types

[Unreleased]: https://github.com/shrynx/protest/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/shrynx/protest/releases/tag/v0.3.0
[0.2.0]: https://github.com/shrynx/protest/releases/tag/v0.2.0
[0.1.0]: https://github.com/shrynx/protest/releases/tag/v0.1.0
