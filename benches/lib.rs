use blink_pairs::{
    nom,
    parser::{parse_filetype, ParseState},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("nom - c", |b| {
        let text = include_str!("./languages/c.c");
        let lines = text.lines().collect::<Vec<_>>();
        b.iter(|| {
            let mut tokens_by_line = Vec::with_capacity(lines.len());
            for line in black_box(&lines).iter() {
                let (_, line_tokens) = nom::parse(line).unwrap();
                tokens_by_line.push(line_tokens);
            }
        })
    });

    c.bench_function("parse - c", |b| {
        let text = include_str!("./languages/c.c");
        let lines = text.lines().collect::<Vec<_>>();
        b.iter(|| parse_filetype("c", black_box(&lines), ParseState::Normal))
    });

    c.bench_function("parse - rust", |b| {
        let text = include_str!("./languages/rust.rs");
        let lines = text.lines().collect::<Vec<_>>();
        b.iter(|| parse_filetype("rust", black_box(&lines), ParseState::Normal))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
