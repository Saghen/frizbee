use std::hint::black_box;

use frizbee::prefilter::bitmask::{string_to_bitmask, string_to_bitmask_scalar};

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
    let needle = black_box(needle).as_bytes();
    let haystack = black_box(haystack).as_bytes();

    let length = haystack.len();
    let mut group = c.benchmark_group(format!("prefilter/{length}"));

    // group.bench_function("scalar", |b| {
    //     b.iter(|| ordered::scalar::match_haystack(needle, haystack))
    // });
    //
    // // Generic SIMD implementation
    // group.bench_function("generic<16>", |b| {
    //     b.iter(|| ordered::simd::prefilter_simd::<W>(needle, haystack))
    // });
    //
    // // Manual SSE4.2 implementation
    // group.bench_function("sse4.2<16>", |b| {
    //     b.iter(|| unsafe { ordered::x86_64::match_haystack_avx2::<W>(needle, haystack) })
    // });
    // group.bench_function("sse4.2<16>/insensitive", |b| {
    //     let needle_chars = needle
    //         .iter()
    //         .map(|&c| unsafe {
    //             if c.is_ascii_uppercase() {
    //                 (_mm_set1_epi8(c as i8), _mm_set1_epi8((c | 0x20) as i8))
    //             } else if c.is_ascii_lowercase() {
    //                 (_mm_set1_epi8(c as i8), _mm_set1_epi8((c & !0x20) as i8))
    //             } else {
    //                 (_mm_set1_epi8(c as i8), _mm_set1_epi8(c as i8))
    //             }
    //         })
    //         .collect::<Vec<_>>();
    //     b.iter(|| unsafe {
    //         ordered::x86_64::match_haystack_insensitive_avx2::<W>(
    //             needle,
    //             black_box(&needle_chars),
    //             haystack,
    //         )
    //     })
    // });
    //
    // // Typo
    // group.bench_function("memchr with typo", |b| {
    //     b.iter(|| ordered::memchr::match_haystack_insensitive_typo(needle, haystack))
    // });
    //
    // // Unordered
    // group.bench_function("sse4.2<16>/unordered", |b| {
    //     b.iter(|| unsafe {
    //         unordered::x86_64::match_haystack_unordered_avx2::<W>(needle, haystack)
    //     })
    // });
    // group.bench_function("sse4.2<16>/unordered/typos", |b| {
    //     b.iter(|| unsafe {
    //         unordered::x86_64::match_haystack_unordered_typos_avx2::<W>(needle, haystack, 1)
    //     })
    // });
    // group.bench_function("simd<16>/unordered", |b| {
    //     b.iter(|| unordered::simd::prefilter_simd_unordered::<W>(needle, haystack))
    // });

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
