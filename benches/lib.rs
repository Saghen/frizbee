use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use frizbee::*;
use generate::generate_haystack;
use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher,
};

mod generate;

fn criterion_benchmark(c: &mut Criterion) {
    // TODO: vary needle, partial match percent, match percent, median length and num samples
    let needle = "deadbe";
    let haystack = generate_haystack(
        needle,
        generate::HaystackGenerationOptions {
            seed: 12345,
            partial_match_percentage: 0.05,
            match_percentage: 0.05,
            median_length: 16,
            std_dev_length: 4,
            num_samples: 1000,
        },
    );
    let haystack_ref = haystack.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    c.bench_function("frizbee", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options::default(),
            );
        })
    });
    c.bench_function("frizbee_0_typos", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options {
                    max_typos: Some(0),
                    ..Default::default()
                },
            );
        })
    });
    c.bench_function("frizbee_1_typos", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options {
                    max_typos: Some(1),
                    ..Default::default()
                },
            );
        })
    });
    c.bench_function("nucleo", |b| {
        let mut matcher = Matcher::new(Config::DEFAULT);
        let atom = Atom::new(
            needle,
            CaseMatching::Respect,
            Normalization::Never,
            AtomKind::Fuzzy,
            false,
        );
        b.iter(|| {
            atom.match_list(haystack.iter(), &mut matcher);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_secs(1)).with_plots();
    targets = criterion_benchmark
}
criterion_main!(benches);
