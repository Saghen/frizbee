#![feature(portable_simd)]
#![feature(array_repeat)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

mod interleave;
mod match_list;
mod prefilter;

use interleave::{interleave_bench, interleave_misaligned_bench};
use match_list::match_list_bench;
use prefilter::prefilter_bench;

fn criterion_benchmark(c: &mut Criterion) {
    prefilter_bench(c);

    interleave_bench(c);
    interleave_misaligned_bench(c);

    for (name, (match_percentage, partial_match_percentage)) in [
        ("Partial Match", (0.05, 0.20)),
        ("All Match", (1.0, 0.0)),
        ("No Match", (0.0, 0.0)),
    ] {
        match_list_bench(c, name, match_percentage, partial_match_percentage);
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
