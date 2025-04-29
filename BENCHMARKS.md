# Benchmarks

## Table of Contents

- [Environment](#environment)
- [Explanation](#explanation)
- [Benchmark Results](#benchmark-results)
    - [Partial Match](#partial-match)
    - [All Match](#all-match)
    - [No Match](#no-match)

## Environment

You may test these cases yourself via the included benchmarks. Benchmarks were run on a Ryzen 9950x3D and the following environment:

```bash
$ cargo version -v
cargo 1.87.0-nightly (6cf826701 2025-03-14)
release: 1.87.0-nightly
commit-hash: 6cf8267012570f63d6b86e85a2ae5627de52df9e
commit-date: 2025-03-14
host: x86_64-unknown-linux-gnu
libgit2: 1.9.0 (sys:0.20.0 vendored)
libcurl: 8.12.1-DEV (sys:0.4.80+curl-8.12.1 vendored ssl:OpenSSL/3.4.1)
ssl: OpenSSL 3.4.1 11 Feb 2025
os: NixOS 25.5.0 [64-bit]
```

## Explanation

In each of the benchmarks, the median length of the haystacks is varied from 8 to 128.

- **Frizbee**: Uses the `Options::default()`, where we perform the fastest prefilter since no typos are allowed
- **All Scores**: Set via `max_typos: None`, gets the scores for all of the items without any filtering
- **1 Typo**: Set via `max_typos: Some(1)`, performs a slower, but still effective prefilter since a set number of typos are allowed
- **Nucleo**: Runs with normalization disabled, case insentivity enabled and fuzzy matching enabled
- **\$BENCH (Parallel)**: Same as $BENCH, but uses 8 threads to perform the matching in parallel

NOTE: The nucleo parallel benchmark is not included since I haven't discovered a way to wait ensure the matcher has finished running at the moment.

## Benchmark Results

### Partial Match

What I would consider the typical case, where 5% of the haystack matches the needle and 20% of the haystack includes characters from the needle, but doesn't fully match.

```rust
needle: "deadbeef"
partial_match_percentage: 0.20
match_percentage: 0.05
median_length: varies
std_dev_length: median_length / 4
num_samples: 100000
```

|           | `Frizbee`                | `Nucleo`                        | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`             | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:---------------------------------|:------------------------------------------ |
| **`16`**  | `2.23 ms` (✅ **1.00x**)  | `3.62 ms` (❌ *1.62x slower*)    | `5.30 ms` (❌ *2.37x slower*)    | `3.89 ms` (❌ *1.74x slower*)    | `457.09 us` (🚀 **4.88x faster**) | `1.22 ms` (🚀 **1.84x faster**)             |
| **`32`**  | `3.73 ms` (✅ **1.00x**)  | `5.63 ms` (❌ *1.51x slower*)    | `9.32 ms` (❌ *2.50x slower*)    | `5.73 ms` (❌ *1.54x slower*)    | `647.59 us` (🚀 **5.76x faster**) | `1.73 ms` (🚀 **2.16x faster**)             |
| **`64`**  | `5.34 ms` (✅ **1.00x**)  | `8.51 ms` (❌ *1.59x slower*)    | `17.04 ms` (❌ *3.19x slower*)   | `8.22 ms` (❌ *1.54x slower*)    | `898.02 us` (🚀 **5.95x faster**) | `2.79 ms` (🚀 **1.92x faster**)             |
| **`128`** | `11.43 ms` (✅ **1.00x**) | `24.74 ms` (❌ *2.16x slower*)   | `31.00 ms` (❌ *2.71x slower*)   | `18.92 ms` (❌ *1.65x slower*)   | `1.92 ms` (🚀 **5.94x faster**)   | `4.67 ms` (🚀 **2.45x faster**)             |

### All Match

All of the haystacks match the needle. The "All Scores" case will always be the fastest since it skips the prefiltering step, which no longer filters any of the items out.

```rust
needle: "deadbeef"
match_percentage: 1.0
partial_match_percentage: 0.0
median_length: varies
std_dev_length: median_length / 4
num_samples: 100000
```

|           | `Frizbee`                | `Nucleo`                         | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`           | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:---------------------------------|:--------------------------------|:--------------------------------|:-------------------------------|:------------------------------------------ |
| **`16`**  | `8.85 ms` (✅ **1.00x**)  | `22.82 ms` (❌ *2.58x slower*)    | `4.97 ms` (✅ **1.78x faster**)  | `11.97 ms` (❌ *1.35x slower*)   | `1.77 ms` (🚀 **5.01x faster**) | `1.17 ms` (🚀 **7.57x faster**)             |
| **`32`**  | `17.43 ms` (✅ **1.00x**) | `38.52 ms` (❌ *2.21x slower*)    | `9.01 ms` (🚀 **1.93x faster**)  | `20.31 ms` (❌ *1.17x slower*)   | `2.99 ms` (🚀 **5.83x faster**) | `1.90 ms` (🚀 **9.18x faster**)             |
| **`64`**  | `27.49 ms` (✅ **1.00x**) | `64.16 ms` (❌ *2.33x slower*)    | `18.36 ms` (✅ **1.50x faster**) | `29.23 ms` (✅ **1.06x slower**) | `3.91 ms` (🚀 **7.03x faster**) | `2.80 ms` (🚀 **9.81x faster**)             |
| **`128`** | `38.44 ms` (✅ **1.00x**) | `118.63 ms` (❌ *3.09x slower*)   | `30.91 ms` (✅ **1.24x faster**) | `42.34 ms` (✅ **1.10x slower**) | `5.72 ms` (🚀 **6.72x faster**) | `4.64 ms` (🚀 **8.28x faster**)             |

### No Match

None of the haystacks partially or fully match the needle, meaning none of the characters in the needle are present in the haystack.

```rust
needle: "deadbeef"
match_percentage: 0.0
partial_match_percentage: 0.0
median_length: varies
std_dev_length: median_length / 4
num_samples: 100000
```

|           | `Frizbee`               | `Nucleo`                        | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`             | `Frizbee: All Scores (Parallel)`           |
|:----------|:------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:---------------------------------|:------------------------------------------ |
| **`16`**  | `1.53 ms` (✅ **1.00x**) | `2.22 ms` (❌ *1.45x slower*)    | `4.96 ms` (❌ *3.24x slower*)    | `2.48 ms` (❌ *1.62x slower*)    | `277.76 us` (🚀 **5.52x faster**) | `1.18 ms` (✅ **1.30x faster**)             |
| **`32`**  | `2.43 ms` (✅ **1.00x**) | `3.27 ms` (❌ *1.35x slower*)    | `8.95 ms` (❌ *3.68x slower*)    | `3.84 ms` (❌ *1.58x slower*)    | `389.18 us` (🚀 **6.24x faster**) | `1.73 ms` (✅ **1.40x faster**)             |
| **`64`**  | `3.63 ms` (✅ **1.00x**) | `4.67 ms` (❌ *1.29x slower*)    | `16.90 ms` (❌ *4.65x slower*)   | `5.70 ms` (❌ *1.57x slower*)    | `574.03 us` (🚀 **6.33x faster**) | `2.85 ms` (✅ **1.28x faster**)             |
| **`128`** | `8.62 ms` (✅ **1.00x**) | `16.26 ms` (❌ *1.89x slower*)   | `31.16 ms` (❌ *3.61x slower*)   | `15.61 ms` (❌ *1.81x slower*)   | `1.38 ms` (🚀 **6.23x faster**)   | `4.73 ms` (🚀 **1.82x faster**)             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

