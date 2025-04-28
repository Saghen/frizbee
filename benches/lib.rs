use std::{sync::Arc, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use frizbee::{incremental::IncrementalMatcher, one_shot::*, *};
use generate::generate_haystack;
use nucleo::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher as NucleoMatcher, Nucleo,
};

mod generate;

fn criterion_benchmark(c: &mut Criterion) {
    // TODO: vary needle, partial match percent, match percent, median length and num samples
    let needle = "deadbeef";
    let haystack = generate_haystack(
        needle,
        generate::HaystackGenerationOptions {
            seed: 12345,
            partial_match_percentage: 0.05,
            match_percentage: 0.05,
            median_length: 32,
            std_dev_length: 16,
            num_samples: 1000000,
        },
    );
    let haystack_ref = haystack.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    // Typical case
    c.bench_function("frizbee_all_scores_parallel", |b| {
        b.iter(|| {
            match_list_parallel(
                black_box(needle),
                black_box(&haystack_ref),
                Options {
                    max_typos: None,
                    ..Default::default()
                },
                16,
            )
        })
    });
    c.bench_function("frizbee_parallel", |b| {
        b.iter(|| {
            match_list_parallel(
                black_box(needle),
                black_box(&haystack_ref),
                Options::default(),
                16,
            )
        })
    });
    c.bench_function("frizbee", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options::default(),
            )
        })
    });

    // Other fuzzy matchers
    c.bench_function("nucleo_parallel", |b| {
        b.iter(|| {
            let mut matcher = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), Some(16), 1);
            let injector = matcher.injector();
            for item in haystack_ref.iter() {
                let static_item: &&str = unsafe { std::mem::transmute::<&_, &'static _>(item) };
                injector.push(static_item, |_, _| {});
            }
            while matcher.tick(10).running {}
        })
    });
    c.bench_function("nucleo", |b| {
        let mut matcher = NucleoMatcher::new(Config::DEFAULT);
        let atom = Atom::new(
            needle,
            CaseMatching::Ignore,
            Normalization::Never,
            AtomKind::Fuzzy,
            false,
        );
        b.iter(|| atom.match_list(black_box(haystack.iter()), &mut matcher))
    });

    // Score all matches
    c.bench_function("frizbee_all_scores", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options {
                    max_typos: None,
                    ..Default::default()
                },
            )
        })
    });

    // Fixed number of typos
    c.bench_function("frizbee_parallel_1_typos", |b| {
        b.iter(|| {
            match_list_parallel(
                black_box(needle),
                black_box(&haystack_ref),
                Options {
                    max_typos: Some(1),
                    ..Default::default()
                },
                16,
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

    // Incremental
    c.bench_function("frizbee_incremental", |b| {
        b.iter(|| {
            IncrementalMatcher::new(black_box(&haystack_ref))
                .match_needle(black_box(needle), Options::default())
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
