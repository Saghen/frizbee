use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fzrs::simd::*;

fn criterion_benchmark(c: &mut Criterion) {
    let needle = "banny";
    let haystacks = [
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
    ];

    c.bench_function("simd", |b| {
        b.iter(|| {
            smith_waterman_inter_simd_8(black_box(needle), black_box(&haystacks));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
