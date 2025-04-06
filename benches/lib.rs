use blink_pairs::parser::{parse_filetype, State};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse simd - c", |b| {
        let text = include_str!("./languages/c.c");
        let lines = text.lines().collect::<Vec<_>>();
        b.iter(|| parse_filetype("c", black_box(&lines), State::Normal))
    });

    c.bench_function("parse simd - rust", |b| {
        let text = include_str!("./languages/rust.rs");
        let lines = text.lines().collect::<Vec<_>>();
        b.iter(|| parse_filetype("rust", black_box(&lines), State::Normal))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
