use std::hint::black_box;

use frizbee::prefilter::{prefilter, string_to_bitmask};

pub fn prefilter_bench(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("prefilter");

    group.bench_function("bitmask - 16", |b| {
        b.iter(|| {
            (string_to_bitmask(black_box("test".as_bytes()))
                & string_to_bitmask(black_box("testtesttesttest".as_bytes())))
            .count_ones()
        })
    });

    group.bench_function("memchr - 16", |b| {
        b.iter(|| prefilter(black_box("test"), black_box("________test____")))
    });
}
