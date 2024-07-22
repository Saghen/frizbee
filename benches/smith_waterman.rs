use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fzrs::simd::*;

fn criterion_benchmark(c: &mut Criterion) {
    let needle = "banny";
    let haystacks = [
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
    ];

    c.bench_function("simd", |b| {
        //let needle = needle
        //    .as_bytes()
        //    .iter()
        //    .map(|x| *x as u8)
        //    .collect::<Vec<u8>>();
        b.iter(|| {
            smith_waterman_inter_simd_8(black_box(needle), black_box(&haystacks));
        })
    });
    //c.bench_function("interleave_strings", |b| {
    //    b.iter(|| {
    //        black_box(interleave_strings_8(&haystacks));
    //    })
    //});
    //c.bench_function("reference", |b| {
    //    b.iter(|| {
    //        for target in haystacks.iter() {
    //            smith_waterman_reference(
    //                black_box(needle.as_bytes()),
    //                black_box(target.as_bytes()),
    //            );
    //        }
    //    })
    //});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
