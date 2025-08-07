use std::hint::black_box;

use frizbee::prefilter::{
    Prefilter,
    bitmask::{string_to_bitmask, string_to_bitmask_scalar},
    scalar, simd,
};

fn create_haystack(text: &str, text_pos: usize, length: usize) -> String {
    let mut haystack = String::new();
    for _ in 0..text_pos {
        haystack.push('_');
    }
    haystack.push_str(text);
    for _ in 0..(length - text.len() - text_pos) {
        haystack.push('_');
    }
    haystack
}

pub fn prefilter_bench(c: &mut criterion::Criterion) {
    let needle = "test";

    run_prefilter_bench::<64>(c, needle, &create_haystack(needle, 40, 52));
    run_prefilter_bench::<16>(c, needle, &create_haystack(needle, 10, 14));
}

fn run_prefilter_bench<const W: usize>(c: &mut criterion::Criterion, needle: &str, haystack: &str) {
    let needle_cased = &black_box(Prefilter::case_needle(needle));
    let needle = black_box(needle).as_bytes();
    let haystack = black_box(haystack).as_bytes();

    let length = haystack.len();
    let mut group = c.benchmark_group(format!("prefilter/{length}"));

    // Ordered
    group.bench_function("scalar", |b| {
        b.iter(|| scalar::match_haystack(needle, haystack))
    });
    group.bench_function("simd", |b| {
        b.iter(|| simd::match_haystack(needle, haystack))
    });
    #[cfg(target_arch = "x86_64")]
    group.bench_function("x86_64", |b| {
        b.iter(|| unsafe { frizbee::prefilter::x86_64::match_haystack(needle, haystack) })
    });

    // Ordered Insensitive
    group.bench_function("scalar/insensitive", |b| {
        b.iter(|| scalar::match_haystack_insensitive(needle_cased, haystack))
    });
    group.bench_function("simd/insensitive", |b| {
        b.iter(|| simd::match_haystack_insensitive(needle_cased, haystack))
    });
    group.bench_function("x86_64/insensitive", |b| {
        b.iter(|| unsafe {
            frizbee::prefilter::x86_64::match_haystack_insensitive(needle_cased, haystack)
        })
    });

    // Unordered
    group.bench_function("simd/unordered", |b| {
        b.iter(|| simd::match_haystack_unordered(needle, haystack))
    });
    #[cfg(target_arch = "x86_64")]
    group.bench_function("x86_64/unordered", |b| {
        b.iter(|| unsafe { frizbee::prefilter::x86_64::match_haystack_unordered(needle, haystack) })
    });

    // Unordered Insensitive
    group.bench_function("simd/unordered/insensitive", |b| {
        b.iter(|| simd::match_haystack_unordered_insensitive(needle_cased, haystack))
    });
    #[cfg(target_arch = "x86_64")]
    group.bench_function("x86_64/unordered/insensitive/avx2", |b| {
        let needle = unsafe { frizbee::prefilter::x86_64::needle_to_avx2(needle_cased) };
        b.iter(|| unsafe {
            frizbee::prefilter::x86_64::match_haystack_unordered_insensitive_avx2(
                black_box(&needle),
                haystack,
            )
        })
    });
    #[cfg(target_arch = "x86_64")]
    group.bench_function("x86_64/unordered/insensitive", |b| {
        b.iter(|| unsafe {
            frizbee::prefilter::x86_64::match_haystack_unordered_insensitive(needle_cased, haystack)
        })
    });

    // Unordered Typos
    group.bench_function("simd/unordered/typos", |b| {
        b.iter(|| simd::match_haystack_unordered(needle, haystack))
    });
    #[cfg(target_arch = "x86_64")]
    group.bench_function("x86_64/unordered/typos", |b| {
        b.iter(|| unsafe {
            frizbee::prefilter::x86_64::match_haystack_unordered_typos(needle, haystack, 1)
        })
    });

    // Unordered Typos Insensitive
    group.bench_function("simd/unordered/typos/insensitive", |b| {
        b.iter(|| simd::match_haystack_unordered_typos_insensitive(needle_cased, haystack, 1))
    });
    #[cfg(target_arch = "x86_64")]
    group.bench_function("x86_64/unordered/typos/insensitive", |b| {
        b.iter(|| unsafe {
            frizbee::prefilter::x86_64::match_haystack_unordered_typos_insensitive(
                needle_cased,
                haystack,
                1,
            )
        })
    });

    // Bitmask
    group.bench_function("bitmask", |b| {
        let needle_bitmask = black_box(string_to_bitmask(needle));
        b.iter(|| needle_bitmask & string_to_bitmask(haystack) == needle_bitmask)
    });
    group.bench_function("bitmask/scalar", |b| {
        let needle_bitmask = black_box(string_to_bitmask_scalar(needle));
        b.iter(|| needle_bitmask & string_to_bitmask_scalar(haystack) == needle_bitmask)
    });
    group.bench_function("bitmask/typo", |b| {
        let needle_bitmask = black_box(string_to_bitmask(needle));
        b.iter(|| (needle_bitmask & string_to_bitmask(haystack)).count_ones() <= 1)
    });

    group.finish();
}
