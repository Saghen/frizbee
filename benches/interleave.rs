use std::hint::black_box;

use frizbee::smith_waterman::simd::interleave;

pub fn interleave_bench(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("interleave");
    let str = "testtesttesttesttesttesttesttest";

    group.bench_function("32", |b| {
        let data = [str; 32];
        b.iter(|| interleave::<32, 32>(black_box(data)))
    });
    group.bench_function("16", |b| {
        let data = [str; 16];
        b.iter(|| interleave::<32, 16>(black_box(data)))
    });
    group.bench_function("8", |b| {
        let data = [str; 8];
        b.iter(|| interleave::<32, 8>(black_box(data)))
    });
}

pub fn interleave_misaligned_bench(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("interleave/misaligned");
    let str = "testtesttesttesttesttesttesttes"; // length = 31

    group.bench_function("32", |b| {
        let data = [str; 32];
        b.iter(|| interleave::<32, 32>(black_box(data)))
    });
    group.bench_function("16", |b| {
        let data = [str; 16];
        b.iter(|| interleave::<32, 16>(black_box(data)))
    });
    group.bench_function("8", |b| {
        let data = [str; 8];
        b.iter(|| interleave::<32, 8>(black_box(data)))
    });
}
