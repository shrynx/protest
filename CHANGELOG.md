# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New features go here

### Changed
- Changes to existing functionality go here

### Deprecated
- Soon-to-be removed features go here

### Removed
- Removed features go here

### Fixed
- Bug fixes go here

### Security
- Security fixes go here

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

[Unreleased]: https://github.com/shrynx/protest/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/shrynx/protest/releases/tag/v0.1.0
