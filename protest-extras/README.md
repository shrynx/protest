# Protest Extras

Additional generators and strategies for the [Protest](https://github.com/yourusername/protest) property testing library.

[![Crates.io](https://img.shields.io/crates/v/protest-extras.svg)](https://crates.io/crates/protest-extras)
[![Documentation](https://docs.rs/protest-extras/badge.svg)](https://docs.rs/protest-extras)

## Features

All generators use **std library only** (no external dependencies except `rand`).

### 23 Extra Generators

- **Network**: IP addresses (IPv4/IPv6), URLs, email addresses
- **DateTime**: Unix timestamps, durations, system time ranges
- **Text**: Alphabetic, alphanumeric, identifiers, sentences, paragraphs
- **Collections**: Non-empty vectors, sorted collections, unique elements, bounded maps
- **Numeric**: Positive integers, even numbers, prime numbers, percentages
- **Domain**: UUID v4, Base64 strings, hex strings, file paths

### Enhanced Shrinking Strategies

**Basic Strategies:**
- **Smart Shrinking**: Shrink values while preserving invariants/predicates
- **Delta Debugging**: Find minimal failing subsets using binary search
- **Targeted Shrinking**: Shrink toward specific target values instead of zero/empty

**Advanced Strategies:**
- **Cascading Shrinker**: Apply multiple shrinking strategies in sequence for thorough exploration
- **Guided Shrinker**: Use test feedback to efficiently find minimal counterexamples
- **Configurable Shrinker**: Choose between breadth-first (thorough) or depth-first (fast) search strategies

## Installation

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
protest = "0.1"
protest-extras = "0.1"
```

All generators are included by default - no feature flags needed!

## Quick Start

```rust
use protest_extras::prelude::*;
use protest::check;

#[test]
fn test_email_validation() {
    let gen = EmailGenerator::new();
    check(gen, |email: String| {
        assert!(email.contains('@'));
        assert_eq!(email.matches('@').count(), 1);
        Ok(())
    });
}
```

## Documentation

See the [API documentation](https://docs.rs/protest-extras) for comprehensive examples of all 23 generators.

Quick links to generator categories:
- [Network Generators](https://docs.rs/protest-extras/latest/protest_extras/generators/network/index.html) - IP addresses, URLs, emails
- [DateTime Generators](https://docs.rs/protest-extras/latest/protest_extras/generators/datetime/index.html) - Timestamps, durations
- [Text Generators](https://docs.rs/protest-extras/latest/protest_extras/generators/text/index.html) - Alphabetic strings, identifiers, sentences
- [Collection Generators](https://docs.rs/protest-extras/latest/protest_extras/generators/collections/index.html) - Non-empty, sorted, unique vectors
- [Numeric Generators](https://docs.rs/protest-extras/latest/protest_extras/generators/numeric/index.html) - Positive, even, prime numbers
- [Domain Generators](https://docs.rs/protest-extras/latest/protest_extras/generators/domain/index.html) - Hex, Base64, paths, UUIDs
- [Shrinking Strategies](https://docs.rs/protest-extras/latest/protest_extras/shrinking/index.html) - Smart shrinking, delta debugging, targeted shrinking

## Generator Summary

| Category | Generator | Description |
|----------|-----------|-------------|
| **Network** | `IpAddressGenerator` | IPv4/IPv6 addresses |
| | `EmailGenerator` | RFC-compliant email addresses |
| | `UrlGenerator` | HTTP/HTTPS URLs |
| **DateTime** | `TimestampGenerator` | Unix timestamps (i64) |
| | `DurationGenerator` | std::time::Duration |
| | `SystemTimeGenerator` | std::time::SystemTime |
| **Text** | `AlphabeticGenerator` | Letters only (a-z, A-Z) |
| | `AlphanumericGenerator` | Letters and digits |
| | `IdentifierGenerator` | Valid programming identifiers |
| | `SentenceGenerator` | Sentence-like text |
| | `ParagraphGenerator` | Multiple sentences |
| **Collections** | `NonEmptyVecGenerator` | Guaranteed non-empty vectors |
| | `SortedVecGenerator` | Pre-sorted vectors |
| | `UniqueVecGenerator` | Vectors with unique elements |
| | `BoundedMapGenerator` | HashMaps with size bounds |
| **Numeric** | `PositiveIntGenerator<T>` | Positive integers (generic) |
| | `EvenNumberGenerator<T>` | Even numbers (generic) |
| | `PrimeNumberGenerator` | Prime numbers |
| | `PercentageGenerator` | 0.0 to 100.0 |
| **Domain** | `HexGenerator` | Hexadecimal strings |
| | `Base64Generator` | Base64 encoded strings |
| | `PathGenerator` | File system paths |
| | `UuidV4Generator` | UUID v4 (random UUIDs) |

## Example Usage

### Network Generators

```rust
use protest_extras::prelude::*;
use protest::check;

// Generate IPv4 addresses
let gen = IpAddressGenerator::ipv4();
check(gen, |ip: String| {
    let parts: Vec<&str> = ip.split('.').collect();
    assert_eq!(parts.len(), 4);
    Ok(())
});

// Generate valid emails
let gen = EmailGenerator::new();
check(gen, |email: String| {
    assert!(email.contains('@'));
    Ok(())
});
```

### DateTime Generators

```rust
use protest_extras::prelude::*;

// Generate recent timestamps
let gen = TimestampGenerator::recent();

// Generate durations
let gen = DurationGenerator::seconds();

// Generate system times around now
let gen = SystemTimeGenerator::around_now();
```

### Collection Generators

```rust
use protest_extras::prelude::*;
use protest::IntGenerator;

// Generate non-empty vectors
let gen = NonEmptyVecGenerator::new(IntGenerator::new(0, 100), 1, 10);

// Generate sorted vectors
let gen = SortedVecGenerator::new(IntGenerator::new(0, 100), 0, 20);

// Generate vectors with unique elements
let gen = UniqueVecGenerator::new(IntGenerator::new(0, 1000), 5, 20);
```

### Domain Generators

```rust
use protest_extras::prelude::*;

// Generate hexadecimal strings
let gen = HexGenerator::new(8, 32);

// Generate Base64 strings
let gen = Base64Generator::new(6, 32);

// Generate file paths
let gen = PathGenerator::new(1, 4);

// Generate UUIDs
let gen = UuidV4Generator::new();
```

## Advanced Shrinking Strategies

When a property test fails, shrinking finds the minimal counterexample. Protest-extras provides advanced shrinking strategies for complex scenarios:

### CascadingShrinker - Thorough Exploration

Applies multiple shrinking strategies in sequence (element removal, chunking, etc.):

```rust
use protest_extras::prelude::*;

let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
let shrinker = CascadingShrinker::new(original);

// Generates many candidates: single removals, halves, thirds, etc.
let candidates: Vec<_> = shrinker.shrink().collect();
```

**Use when:** You want to explore all possible shrinking approaches systematically.

### GuidedShrinker - Efficient Minimization

Uses test feedback to iteratively find the minimal failing example:

```rust
use protest_extras::prelude::*;

let original = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
let shrinker = GuidedShrinker::new(original);

// Find minimal subset where sum > 200
let (minimal, iterations) = shrinker.find_minimal_with_stats(|v| {
    v.iter().sum::<i32>() > 200
});

// Result: [10, 20, 30, 40, 50, 60] (sum = 210)
```

**Use when:** You can run tests quickly and want the smallest counterexample with minimal overhead.

### ConfigurableShrinker - Search Strategy Control

Choose between breadth-first (thorough) or depth-first (fast) search:

```rust
use protest_extras::prelude::*;

let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Breadth-first: finds truly minimal counterexample
let bfs = ConfigurableShrinker::new(original.clone(), ShrinkStrategy::BreadthFirst)
    .with_max_depth(20);
let minimal_bfs = bfs.find_minimal(|v| v.len() >= 3);

// Depth-first: faster but may not be absolutely minimal
let dfs = ConfigurableShrinker::new(original, ShrinkStrategy::DepthFirst)
    .with_max_depth(20);
let minimal_dfs = dfs.find_minimal(|v| v.len() >= 3);
```

**Use when:** You need to balance between finding the absolute minimal (BFS) and performance (DFS).

### Run the Example

See all shrinking strategies in action:

```bash
cargo run --example advanced_shrinking
```

## No External Dependencies

All generators use **std library only**! The only dependency is `rand` for random number generation, which you already have if you're using Protest.

- UUID v4 implementation: std library only
- Base64 encoding: std library only
- All other generators: std library only

## License

MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)