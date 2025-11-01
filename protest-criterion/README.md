# protest-criterion

**Property-Based Benchmarking with Criterion**

[![Crates.io](https://img.shields.io/crates/v/protest-criterion.svg)](https://crates.io/crates/protest-criterion)
[![Documentation](https://docs.rs/protest-criterion/badge.svg)](https://docs.rs/protest-criterion)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)

Integration between [Protest](https://crates.io/crates/protest) property-based testing and [Criterion](https://crates.io/crates/criterion) benchmarking framework.

## Overview

Property-based benchmarking allows you to:
- 📊 **Benchmark with diverse inputs** - Use generators to create realistic test data
- 📈 **Understand performance distribution** - See how your code performs across the input space
- 🔍 **Detect performance regressions** - Leverage Criterion's statistical analysis
- ⚡ **Find performance edge cases** - Discover worst-case scenarios automatically
- 🎯 **Reproducible benchmarks** - Seed-based generation for consistent results

## Quick Start

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
protest = "0.1"
protest-criterion = "0.1"
criterion = "0.5"
```

Create `benches/my_benchmark.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use protest_criterion::PropertyBencher;
use protest::primitives::IntGenerator;

fn bench_abs(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "i32::abs",
        |b, input: &i32| b.iter(|| input.abs()),
        IntGenerator::new(-1000, 1000),
        100, // number of samples
    );
}

criterion_group!(benches, bench_abs);
criterion_main!(benches);
```

Run with:
```bash
cargo bench
```

## Features

### 1. Benchmark Functions with Generated Inputs

Use `bench_function_over_inputs` to benchmark a function with diverse inputs:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use protest_criterion::PropertyBencher;
use protest::primitives::IntGenerator;

fn bench_multiplication(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "i32 multiplication",
        |b, input: &(i32, i32)| {
            let (a, b_val) = *input;
            b.iter(|| a * b_val)
        },
        protest::primitives::TupleGenerator::new((
            IntGenerator::new(-1000, 1000),
            IntGenerator::new(-1000, 1000),
        )),
        50,
    );
}

criterion_group!(benches, bench_multiplication);
criterion_main!(benches);
```

### 2. Benchmark Property Tests

Use `bench_property` to benchmark property checks:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use protest_criterion::PropertyBencher;
use protest::primitives::VecGenerator;
use protest::Generator;

fn bench_reverse_property(c: &mut Criterion) {
    c.bench_property(
        "vec reverse is involutive",
        VecGenerator::new(
            protest::primitives::IntGenerator::new(0, 100),
            0,
            1000,
        ),
        |v: &Vec<i32>| {
            let mut reversed = v.clone();
            reversed.reverse();
            reversed.reverse();
            assert_eq!(v, &reversed);
        },
        100,
    );
}

criterion_group!(benches, bench_reverse_property);
criterion_main!(benches);
```

### 3. Benchmark by Input Size

Compare performance across different input sizes:

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use protest_criterion::PropertyBenchmarkGroup;
use protest::primitives::{IntGenerator, VecGenerator};
use protest::Generator;

fn bench_sort_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_by_size");

    for size in [10, 100, 1000, 10000] {
        let generator = VecGenerator::new(
            IntGenerator::new(0, 1000),
            size,
            size,
        );

        group.bench_generated(
            &size.to_string(),
            generator,
            |v: &Vec<i32>| {
                let mut sorted = v.clone();
                sorted.sort();
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_sort_by_size);
criterion_main!(benches);
```

## Use Cases

### Sorting Algorithms

Benchmark sorting with various input distributions:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use protest_criterion::PropertyBencher;
use protest::primitives::{IntGenerator, VecGenerator};
use protest::Generator;

fn bench_sorting(c: &mut Criterion) {
    // Random data
    c.bench_property(
        "sort/random",
        VecGenerator::new(IntGenerator::new(0, 10000), 1000, 1000),
        |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        },
        50,
    );

    // Nearly sorted data
    c.bench_property(
        "sort/nearly_sorted",
        VecGenerator::new(IntGenerator::new(0, 100), 1000, 1000),
        |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        },
        50,
    );
}

criterion_group!(benches, bench_sorting);
criterion_main!(benches);
```

### String Operations

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use protest_criterion::PropertyBencher;
use protest::primitives::StringGenerator;

fn bench_string_ops(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "String::to_uppercase",
        |b, input: &String| b.iter(|| input.to_uppercase()),
        StringGenerator::new(10, 100),
        50,
    );

    c.bench_function_over_inputs(
        "String::contains",
        |b, input: &String| b.iter(|| input.contains("test")),
        StringGenerator::new(10, 1000),
        50,
    );
}

criterion_group!(benches, bench_string_ops);
criterion_main!(benches);
```

### Hash Map Operations

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use protest_criterion::PropertyBencher;
use protest::primitives::{IntGenerator, HashMapGenerator, StringGenerator};
use std::collections::HashMap;

fn bench_hashmap(c: &mut Criterion) {
    c.bench_property(
        "HashMap insert and lookup",
        HashMapGenerator::new(
            StringGenerator::new(5, 20),
            IntGenerator::new(0, 1000),
            10,
            100,
        ),
        |map: &HashMap<String, i32>| {
            let mut m = map.clone();
            m.insert("test_key".to_string(), 42);
            let _ = m.get("test_key");
        },
        50,
    );
}

criterion_group!(benches, bench_hashmap);
criterion_main!(benches);
```

## API Reference

### `PropertyBencher` Trait

Extension trait for `Criterion` providing property-based benchmarking methods.

#### `bench_function_over_inputs`

```rust
fn bench_function_over_inputs<I, G, F>(
    &mut self,
    name: &str,
    bench_fn: F,
    generator: G,
    sample_count: usize,
) where
    I: Clone + 'static,
    G: Generator<I>,
    F: FnMut(&mut criterion::Bencher, &I);
```

Benchmarks a function with inputs generated by a property-based generator.

**Parameters:**
- `name` - Name of the benchmark
- `bench_fn` - Function to benchmark (receives bencher and input reference)
- `generator` - Generator for creating test inputs
- `sample_count` - Number of different inputs to generate and benchmark

#### `bench_property`

```rust
fn bench_property<I, G, P>(
    &mut self,
    name: &str,
    generator: G,
    property: P,
    sample_count: usize,
) where
    I: Clone + 'static,
    G: Generator<I>,
    P: Fn(&I) + 'static;
```

Benchmarks a property test function.

**Parameters:**
- `name` - Name of the benchmark
- `generator` - Generator for creating test inputs
- `property` - Property function to benchmark (can include assertions)
- `sample_count` - Number of different inputs to generate and benchmark

### `PropertyBenchmarkGroup` Trait

Extension trait for `BenchmarkGroup` for more ergonomic grouped benchmarks.

#### `bench_generated`

```rust
fn bench_generated<I, G, F>(
    &mut self,
    id: &str,
    generator: G,
    f: F,
) where
    I: Clone + 'static,
    G: Generator<I>,
    F: FnMut(&I) + 'static;
```

Benchmarks with generated inputs within a benchmark group.

## Best Practices

### 1. Choose Appropriate Sample Counts

```rust
// For fast operations, use more samples
c.bench_function_over_inputs(
    "fast_op",
    |b, input: &i32| b.iter(|| input + 1),
    IntGenerator::new(0, 100),
    200, // More samples for stable statistics
);

// For slow operations, fewer samples
c.bench_function_over_inputs(
    "slow_op",
    |b, input: &Vec<i32>| b.iter(|| expensive_operation(input)),
    VecGenerator::new(IntGenerator::new(0, 1000), 1000, 1000),
    20, // Fewer samples to keep benchmark time reasonable
);
```

### 2. Use Realistic Input Distributions

```rust
// Good: Reflects real-world data
VecGenerator::new(IntGenerator::new(0, 1_000_000), 100, 10000)

// Bad: Unrealistic range
VecGenerator::new(IntGenerator::new(0, 10), 10, 10)
```

### 3. Benchmark Different Scenarios

```rust
let mut group = c.benchmark_group("scenarios");

// Best case
group.bench_generated("best_case", sorted_generator, |v| {
    let mut result = v.clone();
    result.sort();
});

// Worst case
group.bench_generated("worst_case", reverse_sorted_generator, |v| {
    let mut result = v.clone();
    result.sort();
});

// Average case
group.bench_generated("average_case", random_generator, |v| {
    let mut result = v.clone();
    result.sort();
});

group.finish();
```

### 4. Use Seeds for Reproducibility

```rust
use rand::{SeedableRng, rngs::StdRng};

// For reproducible benchmarks
let mut rng = StdRng::seed_from_u64(42);
```

## Integration with Protest

protest-criterion works seamlessly with all Protest generators:

- **Primitives**: `IntGenerator`, `StringGenerator`, `BoolGenerator`, etc.
- **Collections**: `VecGenerator`, `HashMapGenerator`, `HashSetGenerator`
- **Composite**: `TupleGenerator`, `OptionGenerator`
- **Custom**: Any type implementing `Generator<T>`

See [protest documentation](https://docs.rs/protest) for available generators.

## Examples

See the [`benches/`](benches/) directory for complete examples:
- `example_benchmarks.rs` - Basic usage

## Performance Tips

1. **Warm up**: Criterion automatically warms up, but consider longer warm-up for complex operations
2. **Input caching**: Pre-generate inputs if generation is expensive
3. **Batch sizes**: Use `criterion::BatchSize` appropriately
4. **Sample size**: Balance statistical significance with benchmark duration

## Comparison with Traditional Benchmarking

### Traditional Criterion

```rust
fn bench_traditional(c: &mut Criterion) {
    c.bench_function("sort", |b| {
        b.iter(|| {
            let mut v = vec![5, 2, 8, 1, 9];
            v.sort();
        });
    });
}
```

**Limitations:**
- Single input case
- No coverage of edge cases
- Manual input creation
- No distribution analysis

### With protest-criterion

```rust
fn bench_property_based(c: &mut Criterion) {
    c.bench_property(
        "sort",
        VecGenerator::new(IntGenerator::new(0, 1000), 0, 1000),
        |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        },
        100, // Tests across 100 different inputs
    );
}
```

**Benefits:**
- Tests across input space
- Automatic edge case discovery
- Statistical distribution of performance
- Realistic input generation

## Contributing

Contributions are welcome! Please see the main [Protest repository](https://github.com/shrynx/protest) for contribution guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Related Crates

- [protest](https://crates.io/crates/protest) - Property-based testing framework
- [criterion](https://crates.io/crates/criterion) - Statistics-driven micro-benchmarking
- [protest-extras](https://crates.io/crates/protest-extras) - Additional generators
- [protest-stateful](https://crates.io/crates/protest-stateful) - Stateful property testing

## Acknowledgments

Built on top of:
- [Criterion.rs](https://github.com/bheisler/criterion.rs) - Excellent benchmarking framework
- [Protest](https://github.com/shrynx/protest) - Property-based testing for Rust
