# protest-insta

Property-based snapshot testing integration for [Protest](https://crates.io/crates/protest) and [Insta](https://crates.io/crates/insta).

## Overview

`protest-insta` combines the power of property-based testing with snapshot testing, allowing you to:

- **Test with diverse inputs** while maintaining visual regression testing
- **Detect unexpected changes** in serialization, formatting, or computation results
- **Document behavior** through automatically captured snapshots
- **Review changes** using Insta's powerful review workflow

## Quick Start

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
protest = "0.1"
protest-insta = "0.1"
insta = "1.41"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Example

```rust
use protest::{Generator, primitives::IntGenerator, config::GeneratorConfig};
use protest_insta::PropertySnapshots;
use serde::Serialize;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[derive(Serialize)]
struct Point { x: i32, y: i32 }

#[test]
fn test_point_serialization() {
    let mut rng = StdRng::seed_from_u64(42);
    let config = GeneratorConfig::default();
    let generator = IntGenerator::new(0, 100);

    let mut snapshots = PropertySnapshots::new("point_serialization");

    for _ in 0..5 {
        let x = generator.generate(&mut rng, &config);
        let y = generator.generate(&mut rng, &config);
        let point = Point { x, y };
        snapshots.assert_json_snapshot(&point);
    }
}
```

## Features

### PropertySnapshots Helper

The `PropertySnapshots` struct manages multiple snapshots with automatic sequential naming:

```rust
use protest_insta::PropertySnapshots;

let mut snapshots = PropertySnapshots::new("my_test");

// Creates snapshots named: my_test_0, my_test_1, my_test_2, ...
snapshots.assert_json_snapshot(&data1);
snapshots.assert_json_snapshot(&data2);
snapshots.assert_debug_snapshot(&data3);
```

### Supported Snapshot Types

#### JSON Snapshots

Perfect for testing serialization of structured data:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Config {
    port: u16,
    host: String,
    debug: bool,
}

let config = Config {
    port: 8080,
    host: "localhost".to_string(),
    debug: true
};

snapshots.assert_json_snapshot(&config);
```

#### Debug Snapshots

Great for testing computation results and non-serializable types:

```rust
let results: Vec<i32> = vec![1, 2, 3, 4, 5];
snapshots.assert_debug_snapshot(&results);
```

#### YAML Snapshots

For YAML-formatted snapshots:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Settings {
    timeout: u64,
    retries: u8,
}

let settings = Settings { timeout: 30, retries: 3 };
snapshots.assert_yaml_snapshot(&settings);
```

### Helper Function

The `property_snapshot_test` function provides a concise API:

```rust
use protest::primitives::IntGenerator;
use protest_insta::property_snapshot_test;
use serde::Serialize;

#[derive(Serialize)]
struct Square { value: i32, squared: i32 }

#[test]
fn test_squaring() {
    property_snapshot_test(
        "square_function",
        IntGenerator::new(1, 10),
        5,      // sample count
        42,     // seed
        |value, snapshots| {
            let squared = value * value;
            let result = Square { value, squared };
            snapshots.assert_json_snapshot(&result);
        }
    );
}
```

## Use Cases

### 1. Serialization Testing

Test that your types serialize consistently across different inputs:

```rust
use protest::primitives::VecGenerator;
use serde::Serialize;

#[derive(Serialize)]
struct Report {
    data: Vec<i32>,
    summary: String,
}

#[test]
fn test_report_serialization() {
    let generator = VecGenerator::new(IntGenerator::new(0, 100), 1, 10);
    let mut snapshots = PropertySnapshots::new("reports");

    // Test with various vector sizes and contents
    for _ in 0..5 {
        let data = generator.generate(&mut rng, &config);
        let report = Report {
            data: data.clone(),
            summary: format!("Count: {}", data.len()),
        };
        snapshots.assert_json_snapshot(&report);
    }
}
```

### 2. API Response Testing

Verify API responses remain stable:

```rust
#[derive(Serialize)]
struct ApiResponse {
    status: u16,
    body: String,
    headers: HashMap<String, String>,
}

#[test]
fn test_api_responses() {
    let mut snapshots = PropertySnapshots::new("api_responses");

    for status in [200, 404, 500] {
        let response = create_response(status);
        snapshots.assert_json_snapshot(&response);
    }
}
```

### 3. Computation Result Testing

Document computation behavior across inputs:

```rust
#[test]
fn test_factorial_outputs() {
    property_snapshot_test(
        "factorial",
        IntGenerator::new(1, 10),
        10,
        123,
        |n, snapshots| {
            let result = factorial(n);
            snapshots.assert_debug_snapshot(&result);
        }
    );
}
```

### 4. Visual Regression Testing

Detect unexpected changes in output format:

```rust
#[test]
fn test_markdown_generation() {
    let mut snapshots = PropertySnapshots::new("markdown");

    for _ in 0..5 {
        let document = generate_document(&mut rng);
        let markdown = document.to_markdown();
        snapshots.assert_debug_snapshot(&markdown);
    }
}
```

## Best Practices

### 1. Use Deterministic Generators

Always use a seeded RNG for reproducible snapshots:

```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

let mut rng = StdRng::seed_from_u64(42);  // ✅ Reproducible
// let mut rng = rand::thread_rng();       // ❌ Non-deterministic
```

### 2. Limit Sample Count

Keep snapshot counts reasonable (5-10) to make reviews manageable:

```rust
let mut snapshots = PropertySnapshots::new("test");

for _ in 0..5 {  // ✅ Reasonable
    // ...
}

// for _ in 0..1000 {  // ❌ Too many snapshots
```

### 3. Use Descriptive Base Names

Choose clear, descriptive names for snapshot groups:

```rust
PropertySnapshots::new("user_profile_json")  // ✅ Clear
PropertySnapshots::new("test1")              // ❌ Unclear
```

### 4. Group Related Snapshots

Use the same base name for related test scenarios:

```rust
let mut snapshots = PropertySnapshots::new("sorting_algorithms");

snapshots.assert_debug_snapshot(&bubble_sort_result);
snapshots.assert_debug_snapshot(&quick_sort_result);
snapshots.assert_debug_snapshot(&merge_sort_result);
```

### 5. Review Snapshots Regularly

Use Insta's review workflow:

```bash
# Review all pending snapshots
cargo insta review

# Accept all snapshots
cargo insta accept

# Reject all snapshots
cargo insta reject
```

## Comparison with Traditional Testing

### Traditional Snapshot Testing

```rust
#[test]
fn test_serialization() {
    let data = MyStruct { value: 42 };
    insta::assert_json_snapshot!(data);
}
```

**Limitations:**
- Tests only one fixed input
- May miss edge cases
- Requires manual input selection

### Property-Based Snapshot Testing

```rust
#[test]
fn test_serialization_property_based() {
    property_snapshot_test(
        "serialization",
        IntGenerator::new(0, 1000),
        10,
        42,
        |value, snapshots| {
            let data = MyStruct { value };
            snapshots.assert_json_snapshot(&data);
        }
    );
}
```

**Benefits:**
- Tests multiple diverse inputs automatically
- Discovers edge cases
- Better coverage with less code

## Examples

See the `examples/` directory for complete working examples:

- **`json_snapshots.rs`** - JSON snapshot testing with complex structures
- **`debug_snapshots.rs`** - Debug snapshots for computation results
- **`property_snapshot_test.rs`** - Using the helper function

Run examples with:

```bash
cargo run --example json_snapshots
cargo run --example debug_snapshots
cargo run --example property_snapshot_test
```

## API Reference

### `PropertySnapshots`

Manages a group of related snapshots with automatic naming.

#### Methods

- `new(base_name)` - Create a new snapshot helper
- `assert_json_snapshot(&value)` - Create a JSON snapshot
- `assert_debug_snapshot(&value)` - Create a debug snapshot
- `assert_yaml_snapshot(&value)` - Create a YAML snapshot
- `reset()` - Reset the counter to 0
- `count()` - Get the current counter value

### `property_snapshot_test`

Helper function for concise property-based snapshot testing.

#### Parameters

- `test_name: &str` - Base name for snapshots
- `generator: G` - Generator for test inputs
- `sample_count: usize` - Number of samples to generate
- `seed: u64` - RNG seed for reproducibility
- `test_fn: F` - Test function receiving each generated value

## Integration with Insta

This crate is built on top of [Insta](https://insta.rs/), so all Insta features work seamlessly:

- **Snapshot review workflow** - `cargo insta review`
- **Inline snapshots** - Use Insta's inline snapshot macros
- **Settings** - Configure Insta with `insta::Settings`
- **Filters** - Apply Insta's redaction filters

## FAQ

### Q: How do I review snapshots?

A: Use Insta's CLI tool:

```bash
cargo install cargo-insta
cargo insta review
```

### Q: Where are snapshots stored?

A: By default, in a `snapshots/` directory next to your test file. Insta manages this automatically.

### Q: Should I commit snapshots to git?

A: Yes! Snapshots are part of your test suite and should be version controlled.

### Q: How many samples should I generate?

A: Start with 5-10. More samples give better coverage but make reviews more tedious.

### Q: Can I use this with existing Insta tests?

A: Absolutely! `protest-insta` is just a helper layer on top of Insta. Mix and match freely.

## Contributing

Contributions are welcome! Please see the [main Protest repository](https://github.com/shrynx/protest) for contribution guidelines.

## License

Licensed under the MIT license. See [LICENSE](../LICENSE) for details.
