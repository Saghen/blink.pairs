use blink_pairs::{
    buffer::ParsedBuffer,
    parser::{
        languages::{Rust, C},
        parse_filetype, tokenize, Matcher, State,
    },
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benches(c: &mut Criterion) {
    let c_text: &str = include_str!("./languages/c.c");
    let rust_text: &str = include_str!("./languages/rust.rs");
    let c_lines = c_text.lines().collect::<Box<[_]>>();
    let rust_lines = rust_text.lines().collect::<Box<[_]>>();

    c.bench_function("parse simd - c", |b| {
        b.iter(|| ParsedBuffer::parse("c", black_box(&c_lines)))
    });

    c.bench_function("parse simd - rust", |b| {
        b.iter(|| {
            ParsedBuffer::parse("rust", black_box(&rust_lines));
        })
    });

    c.bench_function("tokenize simd - c", |b| {
        b.iter(|| {
            tokenize::<64>(black_box(c_text), black_box(C::TOKENS)).for_each(|c| {
                black_box(c);
            })
        })
    });

    c.bench_function("tokenize simd - rust", |b| {
        b.iter(|| {
            tokenize::<64>(black_box(rust_text), black_box(Rust::TOKENS)).for_each(|c| {
                black_box(c);
            })
        })
    });
}

criterion_group!(benches, criterion_benches);
criterion_main!(benches);
