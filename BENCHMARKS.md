# Benchmarks

## Table of Contents

- [Environment](#environment)
- [Explanation](#explanation)
- [Benchmark Results](#benchmark-results)
    - [Chromium](#chromium)
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

### Chromium

List of all file paths in the chromium repository, with a median length of 67 characters.
```rust
needle: "linux"
match_percentage: 0.08
partial_match_percentage: unknown
median_length: 67
std_dev_length: unknown
num_samples: 1406941
```

|          | `Nucleo`                 | `Frizbee`                       | `Frizbee: All Scores`            | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`            | `Frizbee: All Scores (Parallel)`           |
|:---------|:-------------------------|:--------------------------------|:---------------------------------|:--------------------------------|:--------------------------------|:------------------------------------------ |
| **`67`** | `93.70 ms` (✅ **1.00x**) | `38.40 ms` (🚀 **2.44x faster**) | `118.30 ms` (❌ *1.26x slower*)   | `39.53 ms` (🚀 **2.37x faster**) | `7.92 ms` (🚀 **11.83x faster**) | `22.74 ms` (🚀 **4.12x faster**)            |

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

|           | `Nucleo`                 | `Frizbee`                       | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`              | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:----------------------------------|:------------------------------------------ |
| **`16`**  | `3.63 ms` (✅ **1.00x**)  | `1.59 ms` (🚀 **2.28x faster**)  | `4.01 ms` (✅ **1.10x slower**)  | `1.78 ms` (🚀 **2.03x faster**)  | `323.03 us` (🚀 **11.23x faster**) | `1.08 ms` (🚀 **3.34x faster**)             |
| **`32`**  | `5.62 ms` (✅ **1.00x**)  | `2.70 ms` (🚀 **2.08x faster**)  | `6.30 ms` (❌ *1.12x slower*)    | `3.07 ms` (🚀 **1.83x faster**)  | `487.56 us` (🚀 **11.52x faster**) | `1.41 ms` (🚀 **3.97x faster**)             |
| **`64`**  | `8.91 ms` (✅ **1.00x**)  | `5.17 ms` (✅ **1.72x faster**)  | `11.12 ms` (❌ *1.25x slower*)   | `5.74 ms` (✅ **1.55x faster**)  | `836.33 us` (🚀 **10.65x faster**) | `2.07 ms` (🚀 **4.29x faster**)             |
| **`128`** | `26.31 ms` (✅ **1.00x**) | `13.53 ms` (🚀 **1.94x faster**) | `19.67 ms` (✅ **1.34x faster**) | `14.67 ms` (✅ **1.79x faster**) | `1.97 ms` (🚀 **13.35x faster**)   | `3.25 ms` (🚀 **8.08x faster**)             |

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
| **`16`**  | `23.64 ms` (✅ **1.00x**)  | `4.68 ms` (🚀 **5.05x faster**)  | `3.97 ms` (🚀 **5.95x faster**)  | `5.92 ms` (🚀 **3.99x faster**)  | `829.58 us` (🚀 **28.50x faster**) | `1.11 ms` (🚀 **21.38x faster**)            |
| **`32`**  | `40.54 ms` (✅ **1.00x**)  | `9.27 ms` (🚀 **4.37x faster**)  | `6.27 ms` (🚀 **6.46x faster**)  | `12.17 ms` (🚀 **3.33x faster**) | `1.64 ms` (🚀 **24.66x faster**)   | `1.42 ms` (🚀 **28.58x faster**)            |
| **`64`**  | `66.34 ms` (✅ **1.00x**)  | `15.46 ms` (🚀 **4.29x faster**) | `11.20 ms` (🚀 **5.92x faster**) | `19.42 ms` (🚀 **3.42x faster**) | `2.43 ms` (🚀 **27.25x faster**)   | `2.09 ms` (🚀 **31.69x faster**)            |
| **`128`** | `124.55 ms` (✅ **1.00x**) | `25.00 ms` (🚀 **4.98x faster**) | `19.58 ms` (🚀 **6.36x faster**) | `28.95 ms` (🚀 **4.30x faster**) | `3.55 ms` (🚀 **35.11x faster**)   | `3.23 ms` (🚀 **38.55x faster**)            |

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

|           | `Nucleo`                 | `Frizbee`                       | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`             | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:---------------------------------|:------------------------------------------ |
| **`16`**  | `2.09 ms` (✅ **1.00x**)  | `1.04 ms` (🚀 **2.02x faster**)  | `3.94 ms` (❌ *1.88x slower*)    | `1.05 ms` (🚀 **2.00x faster**)  | `229.62 us` (🚀 **9.12x faster**) | `1.06 ms` (🚀 **1.97x faster**)             |
| **`32`**  | `3.26 ms` (✅ **1.00x**)  | `1.75 ms` (🚀 **1.86x faster**)  | `6.25 ms` (❌ *1.92x slower*)    | `1.79 ms` (🚀 **1.83x faster**)  | `339.89 us` (🚀 **9.60x faster**) | `1.40 ms` (🚀 **2.33x faster**)             |
| **`64`**  | `4.79 ms` (✅ **1.00x**)  | `3.81 ms` (✅ **1.26x faster**)  | `11.28 ms` (❌ *2.35x slower*)   | `3.99 ms` (✅ **1.20x faster**)  | `637.96 us` (🚀 **7.51x faster**) | `2.12 ms` (🚀 **2.26x faster**)             |
| **`128`** | `16.90 ms` (✅ **1.00x**) | `11.52 ms` (✅ **1.47x faster**) | `19.97 ms` (❌ *1.18x slower*)   | `12.36 ms` (✅ **1.37x faster**) | `1.73 ms` (🚀 **9.77x faster**)   | `3.25 ms` (🚀 **5.20x faster**)             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

