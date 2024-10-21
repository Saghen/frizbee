use criterion::{black_box, criterion_group, criterion_main, Criterion};
use frizbee::*;
use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher,
};

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn get_data() -> Vec<String> {
    let path = Path::new("/home/saghen/downloads/title.basics.tsv");
    let file = File::open(path).unwrap();
    let reader = io::BufReader::new(file);

    let mut unique_values = HashSet::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let columns: Vec<&str> = line.split('\t').collect();

        if columns.len() >= 3 {
            let third_column = columns[2].trim();
            unique_values.insert(third_column.to_string().to_ascii_lowercase());
        }
    }

    unique_values
        .iter()
        .map(|x| x.to_string())
        .filter(|x| x.len() < 50)
        .collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    let query = "banny";
    let targets = [
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
        "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
    ];

    let mut targets_large = targets.to_vec().clone();
    targets_large.append(&mut targets_large.clone()); // 32
    targets_large.append(&mut targets_large.clone()); // 64
    targets_large.append(&mut targets_large.clone()); // 128
    targets_large.append(&mut targets_large.clone()); // 256
    targets_large.append(&mut targets_large.clone()); // 512
    targets_large.append(&mut targets_large.clone()); // 1024
    targets_large.append(&mut targets_large.clone()); // 2048
    targets_large.append(&mut targets_large.clone()); // 4096
    targets_large.append(&mut targets_large.clone()); // 8192
    targets_large.append(&mut targets_large.clone()); // 16384
    targets_large.append(&mut targets_large.clone()); // 32768
    targets_large.append(&mut targets_large.clone()); // 65536

    let data = get_data();
    let data = data
        .iter()
        .take(50_000)
        .map(|x| x.as_str())
        .collect::<Vec<&str>>();

    c.bench_function("match_list", |b| {
        b.iter(|| {
            match_list(black_box(query), black_box(&data), Options::default());
        })
    });
    c.bench_function("match_list_prefilter", |b| {
        b.iter(|| {
            match_list(
                black_box(query),
                black_box(&data),
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
            query,
            CaseMatching::Respect,
            Normalization::Never,
            AtomKind::Fuzzy,
            false,
        );
        b.iter(|| {
            atom.match_list(data.iter(), &mut matcher);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
