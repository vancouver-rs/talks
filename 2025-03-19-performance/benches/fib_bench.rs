use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(unused_imports)]
use perfrs::{fib_recursive, fib_iterative, fib_lookup, fib_const};

pub fn criterion_benchmark(c: &mut Criterion) {

    // Slow
    c.bench_function("fib_recursive 20",|b| b.iter(|| fib_recursive(black_box(20))));


    //// Much faster
    c.bench_function("fib_iterative 20", |b| b.iter(|| fib_iterative(black_box(20))));

    // Even faster
    c.bench_function("fib_lookup 20", |b| b.iter(|| fib_lookup(black_box(20))));

    // Slight improvement on lookup
    c.bench_function("fib_const 20", |b| b.iter(|| fib_const(black_box(20))));
    c.bench_function("fib_const const 20", |b| b.iter(|| fib_const(20)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
