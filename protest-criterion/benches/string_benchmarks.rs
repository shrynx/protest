//! Vector operation benchmarks (demonstrating property-based benchmarking)
//!
//! Note: This example uses vectors as StringGenerator is in protest-extras.
//! The patterns shown here apply equally to any generator type.

use criterion::{Criterion, criterion_group, criterion_main};
use protest::primitives::{IntGenerator, VecGenerator};
use protest_criterion::PropertyBencher;

/// Benchmark vector operations with various sizes
fn bench_vec_ops(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "Vec::len/small",
        |b, input: &Vec<i32>| b.iter(|| input.len()),
        VecGenerator::new(IntGenerator::new(0, 100), 10, 100),
        50,
    );

    c.bench_function_over_inputs(
        "Vec::contains",
        |b, input: &Vec<i32>| b.iter(|| input.contains(&42)),
        VecGenerator::new(IntGenerator::new(0, 100), 10, 1000),
        50,
    );

    c.bench_function_over_inputs(
        "Vec::iter::count",
        |b, input: &Vec<i32>| b.iter(|| input.len()),
        VecGenerator::new(IntGenerator::new(0, 100), 10, 1000),
        50,
    );
}

/// Benchmark operations by input size
fn bench_by_size(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "Vec::is_empty/small",
        |b, input: &Vec<i32>| b.iter(|| input.is_empty()),
        VecGenerator::new(IntGenerator::new(0, 100), 0, 20),
        50,
    );

    c.bench_function_over_inputs(
        "Vec::is_empty/medium",
        |b, input: &Vec<i32>| b.iter(|| input.is_empty()),
        VecGenerator::new(IntGenerator::new(0, 100), 50, 200),
        50,
    );

    c.bench_function_over_inputs(
        "Vec::is_empty/large",
        |b, input: &Vec<i32>| b.iter(|| input.is_empty()),
        VecGenerator::new(IntGenerator::new(0, 100), 500, 2000),
        30,
    );
}

criterion_group!(benches, bench_vec_ops, bench_by_size);
criterion_main!(benches);
