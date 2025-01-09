#![feature(portable_simd)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use frizbee::simd::*;

use frizbee::r#const::*;
use smith_waterman_macro::{self, generate_smith_waterman};
use std::ops::{BitAnd, BitOr, Not};
use std::simd::cmp::*;
use std::simd::num::SimdUint;
use std::simd::{Mask, Simd};

generate_smith_waterman!(8);

fn criterion_benchmark(c: &mut Criterion) {
    let needle = "banny";
    let haystacks = [
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
    ];

    c.bench_function("simd_macro", |b| {
        b.iter(|| {
            smith_waterman_inter_simd_8(black_box(needle), black_box(&haystacks));
        })
    });

    c.bench_function("simd_generic", |b| {
        b.iter(|| {
            smith_waterman::<u8, 8>(black_box(needle), black_box(&haystacks));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
