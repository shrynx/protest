//! Example benchmarks demonstrating protest-criterion

use criterion::{Criterion, criterion_group, criterion_main};
use protest::primitives::{IntGenerator, VecGenerator};
use protest_criterion::PropertyBencher;

fn bench_abs(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "i32::abs",
        |b, input: &i32| b.iter(|| input.abs()),
        IntGenerator::new(-1000, 1000),
        10,
    );
}

fn bench_sort(c: &mut Criterion) {
    c.bench_property(
        "vec sort",
        VecGenerator::new(IntGenerator::new(0, 1000), 100, 100),
        |v: &Vec<i32>| {
            let mut sorted = v.clone();
            sorted.sort();
        },
        10,
    );
}

criterion_group!(benches, bench_abs, bench_sort);
criterion_main!(benches);
