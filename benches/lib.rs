use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use frizbee::{incremental::IncrementalMatcher, one_shot::*, *};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
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
            num_samples: 10000,
        },
    );
    let haystack_ref = haystack.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    c.bench_function("frizbee", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options::default(),
            )
        })
    });
    c.bench_function("frizbee_incremental", |b| {
        b.iter(|| {
            IncrementalMatcher::new(black_box(&haystack_ref))
                .match_needle(black_box(needle), Options::default())
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
            )
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
            )
        })
    });
    c.bench_function("frizbee_2_typos", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options {
                    max_typos: Some(2),
                    ..Default::default()
                },
            )
        })
    });

    c.bench_function("nucleo", |b| {
        let mut matcher = Matcher::new(Config::DEFAULT);
        let atom = Atom::new(
            needle,
            CaseMatching::Ignore,
            Normalization::Never,
            AtomKind::Fuzzy,
            false,
        );
        b.iter(|| atom.match_list(black_box(haystack.iter()), &mut matcher))
    });

    c.bench_function("skim", |b| {
        let matcher = SkimMatcherV2::default();
        let haystack_str = haystack.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

        b.iter(|| {
            let mut matches = vec![];
            for item in black_box(haystack_str.iter()) {
                let score = matcher.fuzzy_match(needle, item);
                let _ = black_box(score);
                if let Some(score) = score {
                    matches.push((score, item.to_string()));
                }
            }
            matches.sort_by_key(|(score, _)| *score);
            matches
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_secs(1));
    targets = criterion_benchmark
}
criterion_main!(benches);
