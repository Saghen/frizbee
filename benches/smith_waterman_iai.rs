use frizbee::smith_waterman::simd::smith_waterman;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

const NEEDLE: &str = "print";
const HAYSTACK: [&str; 16] = [
    "XOpBuawtjG",
    "t6GuC",
    "rmLingPLt",
    "dpDrlcint",
    "tNprinta9duM",
    "BaMmlfqEW5xz",
    "V0884Xjfp",
    "YBeQ41Y",
    "evXfkcR7iFz7nt",
    "pYfrintXRv",
    "print",
    "9p7r6iOnt",
    "TsNCxC05L4D4Y",
    "vaAuDPiQDenHt",
    "NQOK2K3cU",
    "bnTt6e4i",
];

#[library_benchmark]
#[bench::print(NEEDLE, HAYSTACK)]
fn bench_smith_waterman_simd_u8_16_16(needle: &str, haystack: [&str; 16]) -> [u16; 16] {
    black_box(smith_waterman::<u8, 16, 16>(needle, &haystack).0)
}

library_benchmark_group!(name = benches; benchmarks = bench_smith_waterman_simd_u8_16_16);
main!(library_benchmark_groups = benches);
