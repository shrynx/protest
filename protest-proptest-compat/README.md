# protest-proptest-compat

Migration helpers for transitioning from [proptest](https://crates.io/crates/proptest) to [Protest](https://crates.io/crates/protest).

## Overview

This crate provides utilities and guidance to make migrating from proptest to Protest easier. Rather than being a drop-in replacement, it focuses on:

- **Helper functions** for common migration patterns
- **Strategy adapters** to bridge proptest and Protest concepts
- **Side-by-side examples** showing proptest vs Protest equivalents
- **Comprehensive migration guide**

## Quick Migration Guide

### Step 1: Update Dependencies

```toml
[dev-dependencies]
# Remove:
# proptest = "1.0"

# Add:
protest = "0.1"
protest-proptest-compat = "0.1"  # For migration helpers
```

### Step 2: Update Imports

```rust
// Before:
use proptest::prelude::*;

// After:
use protest::*;
// or
use protest::ergonomic::*;
```

### Step 3: Convert Tests

## Common Migration Patterns

### Pattern 1: Simple Range Properties

**Before (Proptest):**
```rust
proptest! {
    #[test]
    fn test_addition(a in 0..100i32, b in 0..100i32) {
        assert!(a + b >= a);
        assert!(a + b >= b);
    }
}
```

**After (Protest):**
```rust
#[test]
fn test_addition() {
    property!(generator!(i32, 0, 100), |(a, b)| {
        a + b >= a && a + b >= b
    });
}
```

### Pattern 2: Vector Properties

**Before (Proptest):**
```rust
proptest! {
    #[test]
    fn reverse_twice_is_identity(v: Vec<i32>) {
        let mut v2 = v.clone();
        v2.reverse();
        v2.reverse();
        assert_eq!(v, v2);
    }
}
```

**After (Protest - Ergonomic API):**
```rust
#[test]
fn reverse_twice_is_identity() {
    property(|mut v: Vec<i32>| {
        let original = v.clone();
        v.reverse();
        v.reverse();
        v == original
    })
    .iterations(100)
    .run()
    .expect("property should hold");
}
```

### Pattern 3: Custom Strategies

**Before (Proptest):**
```rust
use proptest::strategy::Strategy;

fn user_strategy() -> impl Strategy<Value = User> {
    (0..1000u32, "[a-z]{5,10}").prop_map(|(id, name)| {
        User { id, name }
    })
}
```

**After (Protest):**
```rust
use protest::Generator;

#[derive(Generator)]
struct User {
    #[generator(range = "0..1000")]
    id: u32,

    #[generator(length = "5..10")]
    name: String,
}

// Or manually:
struct UserGenerator;
impl Generator<User> for UserGenerator {
    fn generate(&self, rng: &mut dyn RngCore, config: &GeneratorConfig) -> User {
        User {
            id: IntGenerator::new(0, 1000).generate(rng, config),
            name: StringGenerator::new(5, 10).generate(rng, config),
        }
    }
}
```

### Pattern 4: Option and Result Types

**Before (Proptest):**
```rust
use proptest::option;

prop::option::of(0..100i32)
```

**After (Protest with helpers):**
```rust
use protest_proptest_compat::option_generator;
use protest::primitives::IntGenerator;

option_generator(IntGenerator::new(0, 100), 0.5)  // 50% Some, 50% None
```

### Pattern 5: Collections

**Before (Proptest):**
```rust
use proptest::collection::vec;

vec(0..100i32, 0..10)  // Vec with 0-10 elements
```

**After (Protest with helpers):**
```rust
use protest_proptest_compat::vec_generator;
use protest::primitives::IntGenerator;

vec_generator(IntGenerator::new(0, 100), 0, 10)
```

## Migration Helpers API

### `range_to_generator`

Convert a proptest range to a Protest generator:

```rust
use protest_proptest_compat::range_to_generator;

let generator = range_to_generator(0, 100);
```

### `vec_generator`

Create a vector generator:

```rust
use protest_proptest_compat::vec_generator;
use protest::primitives::IntGenerator;

let generator = vec_generator(IntGenerator::new(0, 100), 5, 10);
```

### `option_generator`

Create an option generator with custom probability:

```rust
use protest_proptest_compat::option_generator;
use protest::primitives::IntGenerator;

let generator = option_generator(IntGenerator::new(0, 100), 0.7);  // 70% Some
```

### `GeneratorAdapter`

Wrap a Protest generator for easier use:

```rust
use protest_proptest_compat::GeneratorAdapter;
use protest::primitives::IntGenerator;

let adapter = GeneratorAdapter::new(IntGenerator::new(0, 100));
let value = adapter.generate(&mut rng);
```

## Complete Migration Checklist

- [ ] **Replace dependencies** - Remove proptest, add protest
- [ ] **Update imports** - Replace `proptest::prelude::*` with `protest::*`
- [ ] **Remove `proptest!` macro** - Convert to regular `#[test]` functions
- [ ] **Convert ranges** - `0..100i32` → `generator!(i32, 0, 100)`
- [ ] **Convert collections** - Use `VecGenerator`, etc.
- [ ] **Convert strategies** - Implement `Generator` trait
- [ ] **Update assertions** - `prop_assert!` → `assert!`
- [ ] **Remove `prop_map`** - Use generator methods or custom generators
- [ ] **Test thoroughly** - Ensure all tests pass with same coverage

## Key Differences

### Macro Syntax

| Proptest | Protest |
|----------|---------|
| `proptest! { #[test] fn ... }` | `#[test] fn ...` with `property!` or `property()` inside |
| `a in 0..100` | `generator!(i32, 0, 100)` or manual generator |

### Strategies vs Generators

| Concept | Proptest | Protest |
|---------|----------|---------|
| **Trait** | `Strategy` | `Generator` |
| **Range** | `0..100` | `IntGenerator::new(0, 100)` |
| **Vector** | `vec(strategy, size)` | `VecGenerator::new(gen, min, max)` |
| **Option** | `prop::option::of(...)` | `OptionGenerator::new(...)` |
| **Custom** | `impl Strategy` | `impl Generator` |

### Configuration

| Feature | Proptest | Protest |
|---------|----------|---------|
| **Iterations** | ProptestConfig | `.iterations(N)` |
| **Shrinking** | Built-in | Built-in (automatic) |
| **Mapping** | `.prop_map()` | Generator methods or custom impl |
| **Filtering** | `.prop_filter()` | Preconditions in property |

## Advantages of Protest

After migration, you'll benefit from:

- **Simpler API** - Less boilerplate, more ergonomic
- **Better type inference** - Automatic generator selection
- **Derive macros** - `#[derive(Generator)]` for custom types
- **Async support** - Built-in async property testing
- **Fluent builders** - Chain configuration methods naturally
- **Better shrinking** - Automatic minimal counterexample finding
- **Stateful testing** - Built-in state machine testing (protest-stateful)
- **Benchmarking** - Property-based benchmarking (protest-criterion)
- **Snapshot testing** - Property-based snapshots (protest-insta)

## Examples

See the `examples/` directory for:

- **`migration_example.rs`** - Side-by-side comparisons
- **`using_helpers.rs`** - Using migration helper functions

Run examples with:
```bash
cargo run --example migration_example
cargo run --example using_helpers
```

## FAQ

### Q: Is this a drop-in replacement for proptest?

A: No. This crate provides helpers and guidance for migration, but you'll need to update your test code. The migration is straightforward for most cases.

### Q: Can I use proptest and Protest side by side?

A: Yes! You can migrate gradually, keeping some tests in proptest while converting others to Protest.

### Q: What about proptest's `prop_compose!` macro?

A: Use Protest's `#[derive(Generator)]` or implement the `Generator` trait manually for more control.

### Q: How do I handle proptest's `prop_assert!` and `prop_assume!`?

A: Use regular `assert!` for assertions. For assumptions (filtering), use preconditions in your property or generator logic.

### Q: What about regular expressions in proptest?

A: Protest-extras provides `RegexGenerator` for regex-based string generation.

## Contributing

Found a migration pattern not covered here? Please open an issue or PR at the [main Protest repository](https://github.com/shrynx/protest).

## License

Licensed under the MIT license. See [LICENSE](../LICENSE) for details.
