//! Date parsing benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use taskwarriorlib::*;

fn benchmark_date_parsing(c: &mut Criterion) {
    c.bench_function("iso_date_parsing", |b| {
        b.iter(|| {
            // TODO: Implement benchmark when DateParser is ready
            black_box("2025-09-18")
        })
    });
}

criterion_group!(benches, benchmark_date_parsing);
criterion_main!(benches);
