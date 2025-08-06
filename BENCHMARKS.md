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

NOTE: The nucleo parallel benchmark is not included since I haven't discovered a way to ensure the matcher has finished running.

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

|           | `Nucleo`                 | `Frizbee`                      | `Frizbee: All Scores`           | `Frizbee: 1 Typo`              | `Frizbee (Parallel)`              | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:-------------------------------|:--------------------------------|:-------------------------------|:----------------------------------|:------------------------------------------ |
| **`16`**  | `3.73 ms` (✅ **1.00x**)  | `1.39 ms` (🚀 **2.68x faster**) | `4.00 ms` (✅ **1.07x slower**)  | `1.52 ms` (🚀 **2.45x faster**) | `277.27 us` (🚀 **13.46x faster**) | `1.09 ms` (🚀 **3.43x faster**)             |
| **`32`**  | `5.60 ms` (✅ **1.00x**)  | `2.00 ms` (🚀 **2.81x faster**) | `6.64 ms` (❌ *1.19x slower*)    | `2.29 ms` (🚀 **2.45x faster**) | `388.80 us` (🚀 **14.40x faster**) | `1.52 ms` (🚀 **3.68x faster**)             |
| **`64`**  | `8.77 ms` (✅ **1.00x**)  | `3.00 ms` (🚀 **2.92x faster**) | `11.16 ms` (❌ *1.27x slower*)   | `3.31 ms` (🚀 **2.65x faster**) | `532.22 us` (🚀 **16.48x faster**) | `2.11 ms` (🚀 **4.15x faster**)             |
| **`128`** | `25.33 ms` (✅ **1.00x**) | `5.74 ms` (🚀 **4.41x faster**) | `19.80 ms` (✅ **1.28x faster**) | `6.22 ms` (🚀 **4.07x faster**) | `925.09 us` (🚀 **27.38x faster**) | `3.24 ms` (🚀 **7.82x faster**)             |

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

|           | `Nucleo`                  | `Frizbee`                       | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`              | `Frizbee: All Scores (Parallel)`           |
|:----------|:--------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:----------------------------------|:------------------------------------------ |
| **`16`**  | `23.05 ms` (✅ **1.00x**)  | `4.62 ms` (🚀 **4.99x faster**)  | `4.03 ms` (🚀 **5.72x faster**)  | `5.86 ms` (🚀 **3.94x faster**)  | `818.43 us` (🚀 **28.16x faster**) | `1.10 ms` (🚀 **21.03x faster**)            |
| **`32`**  | `39.26 ms` (✅ **1.00x**)  | `9.57 ms` (🚀 **4.10x faster**)  | `6.62 ms` (🚀 **5.93x faster**)  | `12.38 ms` (🚀 **3.17x faster**) | `1.68 ms` (🚀 **23.31x faster**)   | `1.48 ms` (🚀 **26.52x faster**)            |
| **`64`**  | `63.49 ms` (✅ **1.00x**)  | `15.44 ms` (🚀 **4.11x faster**) | `11.16 ms` (🚀 **5.69x faster**) | `19.17 ms` (🚀 **3.31x faster**) | `2.41 ms` (🚀 **26.34x faster**)   | `2.09 ms` (🚀 **30.39x faster**)            |
| **`128`** | `118.12 ms` (✅ **1.00x**) | `25.43 ms` (🚀 **4.65x faster**) | `19.63 ms` (🚀 **6.02x faster**) | `29.41 ms` (🚀 **4.02x faster**) | `3.57 ms` (🚀 **33.13x faster**)   | `3.27 ms` (🚀 **36.17x faster**)            |

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

|           | `Nucleo`                 | `Frizbee`                        | `Frizbee: All Scores`           | `Frizbee: 1 Typo`                | `Frizbee (Parallel)`              | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:---------------------------------|:--------------------------------|:---------------------------------|:----------------------------------|:------------------------------------------ |
| **`16`**  | `2.35 ms` (✅ **1.00x**)  | `789.98 us` (🚀 **2.97x faster**) | `3.98 ms` (❌ *1.70x slower*)    | `837.13 us` (🚀 **2.80x faster**) | `181.84 us` (🚀 **12.90x faster**) | `1.07 ms` (🚀 **2.20x faster**)             |
| **`32`**  | `3.32 ms` (✅ **1.00x**)  | `1.03 ms` (🚀 **3.21x faster**)   | `6.55 ms` (❌ *1.97x slower*)    | `1.05 ms` (🚀 **3.15x faster**)   | `227.10 us` (🚀 **14.61x faster**) | `1.46 ms` (🚀 **2.27x faster**)             |
| **`64`**  | `4.76 ms` (✅ **1.00x**)  | `1.61 ms` (🚀 **2.96x faster**)   | `11.17 ms` (❌ *2.35x slower*)   | `1.61 ms` (🚀 **2.95x faster**)   | `332.81 us` (🚀 **14.30x faster**) | `2.08 ms` (🚀 **2.29x faster**)             |
| **`128`** | `16.37 ms` (✅ **1.00x**) | `3.35 ms` (🚀 **4.89x faster**)   | `19.99 ms` (❌ *1.22x slower*)   | `3.41 ms` (🚀 **4.80x faster**)   | `625.24 us` (🚀 **26.18x faster**) | `3.27 ms` (🚀 **5.01x faster**)             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

