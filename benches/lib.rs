use criterion::{black_box, criterion_group, criterion_main, Criterion};
use frizbee::*;
use generate::generate_haystack;
use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher,
};

mod generate;

fn criterion_benchmark(c: &mut Criterion) {
    let needle = "deadbeef";
    let haystack = generate_haystack(
        needle,
        generate::HaystackGenerationOptions {
            seed: 12345,
            partial_match_percentage: 0.33,
            match_percentage: 0.33,
            median_length: 16,
            std_dev_length: 4,
            num_samples: 1000,
        },
    );
    let haystack_ref = haystack.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    c.bench_function("match_list", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options::default(),
            );
        })
    });
    c.bench_function("match_list_prefilter", |b| {
        b.iter(|| {
            match_list(
                black_box(needle),
                black_box(&haystack_ref),
                Options {
                    prefilter: true,
                    ..Default::default()
                },
            );
        })
    });
    c.bench_function("nucleo_match_list", |b| {
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
