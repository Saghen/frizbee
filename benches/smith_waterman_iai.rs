use frizbee::smith_waterman::simd::smith_waterman;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

const NEEDLE: &str = "print";
const HAYSTACK: [&str; 8] = [
    "XOpBuawtjG",
    "t6GuC",
    "rmLingPLt",
    "dpDrlcint",
    "tNprinta9duM",
    "BaMmlfqEW5xz",
    "V0884Xjfp",
    "YBeQ41Y",
];

#[library_benchmark]
#[bench::print(NEEDLE, HAYSTACK)]
fn bench_smith_waterman_simd_u16_16_8(needle: &str, haystack: [&str; 8]) -> [u16; 8] {
    black_box(smith_waterman::<u16, 16, 8>(needle, &haystack).0)
}

library_benchmark_group!(name = benches; benchmarks = bench_smith_waterman_simd_u16_16_8);
main!(library_benchmark_groups = benches);
