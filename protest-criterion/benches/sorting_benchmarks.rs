//! Comprehensive sorting algorithm benchmarks with property-based inputs

use criterion::{Criterion, criterion_group, criterion_main};
use protest::primitives::{IntGenerator, VecGenerator};
use protest_criterion::{PropertyBencher, PropertyBenchmarkGroup};

/// Benchmark std::sort with various input sizes
fn bench_sort_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_by_size");

    for size in [10, 100, 1000, 5000] {
        let generator = VecGenerator::new(IntGenerator::new(0, 10000), size, size);

        group.bench_generated(&size.to_string(), generator, |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        });
    }

    group.finish();
}

/// Benchmark sorting with different input distributions
fn bench_sort_distributions(c: &mut Criterion) {
    // Random distribution
    c.bench_property(
        "sort/random",
        VecGenerator::new(IntGenerator::new(0, 100000), 1000, 1000),
        |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        },
        30,
    );

    // Small range (more duplicates)
    c.bench_property(
        "sort/many_duplicates",
        VecGenerator::new(IntGenerator::new(0, 10), 1000, 1000),
        |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        },
        30,
    );

    // Large range (fewer duplicates)
    c.bench_property(
        "sort/few_duplicates",
        VecGenerator::new(IntGenerator::new(0, 1000000), 1000, 1000),
        |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        },
        30,
    );
}

/// Benchmark sort_unstable vs sort
fn bench_stable_vs_unstable(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_comparison");

    let generator1 = VecGenerator::new(IntGenerator::new(0, 10000), 1000, 1000);

    group.bench_generated("stable", generator1, |v: &Vec<i32>| {
        let mut sorted = v.clone();
        sorted.sort();
    });

    let generator2 = VecGenerator::new(IntGenerator::new(0, 10000), 1000, 1000);

    group.bench_generated("unstable", generator2, |v: &Vec<i32>| {
        let mut sorted = v.clone();
        sorted.sort_unstable();
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sort_by_size,
    bench_sort_distributions,
    bench_stable_vs_unstable
);
criterion_main!(benches);
