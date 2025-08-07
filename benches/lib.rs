#![feature(portable_simd)]
#![feature(array_repeat)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

mod interleave;
mod match_list;
mod prefilter;

use interleave::{interleave_bench, interleave_misaligned_bench};
use match_list::{match_list_bench, match_list_generated_bench};
use prefilter::prefilter_bench;

fn criterion_benchmark(c: &mut Criterion) {
    prefilter_bench(c);

    interleave_bench(c);
    interleave_misaligned_bench(c);

    // Bench on real data
    let haystack_bytes = std::fs::read("benches/match_list/data.txt")
        .expect("Failed to read benchmark data. Run `wget -O benches/match_list/data.txt https://gist.github.com/ii14/637689ef8d071824e881a78044670310/raw/dc1dbc859daa38b62f4b9a69dec1fc599e4735e7/data.txt`");
    let haystack_str =
        String::from_utf8(haystack_bytes).expect("Failed to parse chromium benchmark data");
    let haystack = haystack_str.split('\n').collect::<Vec<_>>();

    match_list_bench(c, "Chromium", "linux", &haystack);

    // Bench on synthetic data
    for (name, (match_percentage, partial_match_percentage)) in [
        ("Partial Match", (0.05, 0.20)),
        ("All Match", (1.0, 0.0)),
        ("No Match", (0.0, 0.0)),
    ] {
        match_list_generated_bench(c, name, match_percentage, partial_match_percentage);
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
