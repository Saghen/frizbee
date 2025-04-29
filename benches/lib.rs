use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use frizbee::*;
use generate::{generate_haystack, HaystackGenerationOptions};
use nucleo::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher as NucleoMatcher,
};

mod generate;

const SEED: u64 = 12345;

fn criterion_benchmark(c: &mut Criterion) {
    let needle = "deadbeef";

    for (name, (match_percentage, partial_match_percentage)) in [
        ("Partial Match", (0.05, 0.20)),
        ("All Match", (1.0, 0.0)),
        ("No Match", (0.0, 0.0)),
    ] {
        let mut group = c.benchmark_group(name);

        for median_length in [16, 32, 64, 128] {
            // Generate haystacks
            let options = HaystackGenerationOptions {
                seed: SEED,
                partial_match_percentage,
                match_percentage,
                median_length,
                std_dev_length: median_length / 4,
                num_samples: 100_000,
            };
            let haystack_owned = generate_haystack(needle, options.clone());
            let haystack = &haystack_owned
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<_>>();

            group.throughput(criterion::Throughput::Bytes(options.estimate_size()));

            // Sequential
            group.bench_with_input(
                BenchmarkId::new("Frizbee", median_length),
                haystack,
                |b, haystack| b.iter(|| match_list_bench(needle, haystack, Some(0))),
            );
            group.bench_with_input(
                BenchmarkId::new("Nucleo", median_length),
                haystack,
                |b, haystack| {
                    let mut matcher = NucleoMatcher::new(Config::DEFAULT);
                    let atom = Atom::new(
                        needle,
                        CaseMatching::Ignore,
                        Normalization::Never,
                        AtomKind::Fuzzy,
                        false,
                    );
                    b.iter(|| atom.match_list(black_box(haystack.iter()), &mut matcher))
                },
            );
            group.bench_with_input(
                BenchmarkId::new("Frizbee: All Scores", median_length),
                haystack,
                |b, haystack| b.iter(|| match_list_bench(needle, haystack, None)),
            );
            group.bench_with_input(
                BenchmarkId::new("Frizbee: 1 Typo", median_length),
                haystack,
                |b, haystack| b.iter(|| match_list_bench(needle, haystack, Some(1))),
            );

            // Parallel
            group.bench_with_input(
                BenchmarkId::new("Frizbee (Parallel)", median_length),
                haystack,
                |b, haystack| b.iter(|| match_list_parallel_bench(needle, haystack, Some(0), 8)),
            );
            group.bench_with_input(
                BenchmarkId::new("Frizbee: All Scores (Parallel)", median_length),
                haystack,
                |b, haystack| b.iter(|| match_list_parallel_bench(needle, haystack, None, 8)),
            );
        }
        group.finish();
    }
}

fn match_list_bench(needle: &str, haystack: &[&str], max_typos: Option<u16>) -> Vec<Match> {
    match_list(
        black_box(needle),
        black_box(haystack),
        Options {
            max_typos,
            ..Default::default()
        },
    )
}

fn match_list_parallel_bench(
    needle: &str,
    haystack: &[&str],
    max_typos: Option<u16>,
    num_threads: usize,
) -> Vec<Match> {
    match_list_parallel(
        black_box(needle),
        black_box(haystack),
        Options {
            max_typos,
            ..Default::default()
        },
        num_threads,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_secs(1));
    targets = criterion_benchmark
}
criterion_main!(benches);
