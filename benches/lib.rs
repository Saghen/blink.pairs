use blink_pairs_parser::parser::{State, tokenize_filetype};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benches(c: &mut Criterion) {
    let c_src = include_str!("./languages/c.c");
    let rust_src = include_str!("./languages/rust.rs");

    c.bench_function("parse simd - c", |b| {
        b.iter(|| {
            tokenize_filetype("c", black_box(c_src).lines().map(str::as_bytes), State::Normal)
                .unwrap()
                .count()
        })
    });

    c.bench_function("parse simd - rust", |b| {
        b.iter(|| {
            tokenize_filetype("rust", black_box(rust_src).lines().map(str::as_bytes), State::Normal)
                .unwrap()
                .count()
        })
    });
}

criterion_group!(benches, criterion_benches);
criterion_main!(benches);
