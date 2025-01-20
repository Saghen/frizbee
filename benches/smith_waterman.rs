use frizbee::simd::smith_waterman;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

#[library_benchmark]
#[bench::print("print", ["print", "prnit", "println", "_pr_i__nt", "print()", "println", "nltnirp", "pri()nt", "irpnnt", "PrINt", "println()", "tnirp", "nrnlipt", "println!(", "()TNRIP", "(_pr_- int/)"])]
fn bench_smith_waterman_simd_u8_16_16(needle: &str, haystack: [&str; 16]) -> [u16; 16] {
    black_box(smith_waterman::<u8, 16, 16>(needle, &haystack).0)
}

library_benchmark_group!(name = bench_smith_waterman_group; benchmarks = bench_smith_waterman_simd_u8_16_16);
main!(library_benchmark_groups = bench_smith_waterman_group);
