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
- 🎨 **Fluent Builders** - Chain configuration methods naturally
- 🧪 **Common Patterns** - Built-in helpers for mathematical properties
- 🔀 **Parallel Execution** - Run tests in parallel for speed
- 📊 **Statistics & Coverage** - Track generation and test coverage
- 🎭 **Flexible** - Works with any type, sync or async

## Quick Start

Add Protest to your `Cargo.toml`:

```toml
[dev-dependencies]
protest = { version = "0.1", features = ["derive"] }
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

## Documentation

Full documentation is available on [docs.rs](https://docs.rs/protest).

### Key Modules

- `protest::ergonomic` - Ergonomic API (closures, builders, patterns)
- `protest::primitives` - Built-in generators
- `protest::generator` - Generator trait and utilities
- `protest::property` - Property trait and execution
- `protest::shrink` - Shrinking infrastructure
- `protest::config` - Configuration types
- `protest::statistics` - Coverage and statistics

## Feature Flags

```toml
[features]
default = ["derive"]
derive = ["protest-derive"]  # Derive macros
```

Protest has minimal dependencies and no required runtime dependencies. Async support is built-in and runtime-agnostic.

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

- [ ] More built-in generators
- [ ] Enhanced shrinking strategies
- [ ] Integration with more test frameworks
- [ ] Property test replay and persistence
- [ ] Coverage-guided generation
- [ ] Stateful property testing DSL

---

Made with ❤️ for the Rust community
