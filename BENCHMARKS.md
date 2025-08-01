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

The bench compiles with `-C target-cpu=x86-64-v3` which supports the last 10 years of CPUs. `x86-64-v2` performs roughly the same (~2% slower). Ideally, we'd see the same performance using `x86-64` but this requires more work in runtime instruction detection.

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

|           | `Nucleo`                 | `Frizbee`                      | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`             | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:-------------------------------|:--------------------------------|:--------------------------------|:---------------------------------|:------------------------------------------ |
| **`16`**  | `3.30 ms` (✅ **1.00x**)  | `1.84 ms` (✅ **1.79x faster**) | `3.91 ms` (❌ *1.18x slower*)    | `3.10 ms` (✅ **1.07x faster**)  | `405.64 us` (🚀 **8.14x faster**) | `1.12 ms` (🚀 **2.95x faster**)             |
| **`32`**  | `5.01 ms` (✅ **1.00x**)  | `3.13 ms` (✅ **1.60x faster**) | `6.17 ms` (❌ *1.23x slower*)    | `4.86 ms` (✅ **1.03x faster**)  | `610.39 us` (🚀 **8.21x faster**) | `1.41 ms` (🚀 **3.56x faster**)             |
| **`64`**  | `8.20 ms` (✅ **1.00x**)  | `4.52 ms` (🚀 **1.82x faster**) | `10.99 ms` (❌ *1.34x slower*)   | `6.94 ms` (✅ **1.18x faster**)  | `860.52 us` (🚀 **9.53x faster**) | `2.08 ms` (🚀 **3.95x faster**)             |
| **`128`** | `24.89 ms` (✅ **1.00x**) | `9.11 ms` (🚀 **2.73x faster**) | `20.00 ms` (✅ **1.24x faster**) | `15.00 ms` (✅ **1.66x faster**) | `1.67 ms` (🚀 **14.91x faster**)  | `3.33 ms` (🚀 **7.47x faster**)             |

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

|           | `Nucleo`                  | `Frizbee`                       | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`            | `Frizbee: All Scores (Parallel)`           |
|:----------|:--------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:------------------------------------------ |
| **`16`**  | `22.06 ms` (✅ **1.00x**)  | `7.91 ms` (🚀 **2.79x faster**)  | `3.70 ms` (🚀 **5.97x faster**)  | `10.48 ms` (🚀 **2.10x faster**) | `1.68 ms` (🚀 **13.15x faster**) | `1.09 ms` (🚀 **20.23x faster**)            |
| **`32`**  | `38.83 ms` (✅ **1.00x**)  | `14.65 ms` (🚀 **2.65x faster**) | `6.41 ms` (🚀 **6.06x faster**)  | `17.27 ms` (🚀 **2.25x faster**) | `2.66 ms` (🚀 **14.62x faster**) | `1.41 ms` (🚀 **27.46x faster**)            |
| **`64`**  | `64.21 ms` (✅ **1.00x**)  | `19.63 ms` (🚀 **3.27x faster**) | `11.04 ms` (🚀 **5.82x faster**) | `22.78 ms` (🚀 **2.82x faster**) | `3.21 ms` (🚀 **20.00x faster**) | `2.11 ms` (🚀 **30.49x faster**)            |
| **`128`** | `119.92 ms` (✅ **1.00x**) | `27.63 ms` (🚀 **4.34x faster**) | `19.93 ms` (🚀 **6.02x faster**) | `31.61 ms` (🚀 **3.79x faster**) | `4.31 ms` (🚀 **27.82x faster**) | `3.29 ms` (🚀 **36.48x faster**)            |

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

|           | `Nucleo`                 | `Frizbee`                      | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`             | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:-------------------------------|:--------------------------------|:--------------------------------|:---------------------------------|:------------------------------------------ |
| **`16`**  | `1.96 ms` (✅ **1.00x**)  | `1.22 ms` (✅ **1.61x faster**) | `3.82 ms` (❌ *1.95x slower*)    | `1.89 ms` (✅ **1.04x faster**)  | `255.90 us` (🚀 **7.67x faster**) | `1.05 ms` (🚀 **1.86x faster**)             |
| **`32`**  | `2.80 ms` (✅ **1.00x**)  | `2.05 ms` (✅ **1.37x faster**) | `6.15 ms` (❌ *2.20x slower*)    | `3.29 ms` (❌ *1.17x slower*)    | `354.36 us` (🚀 **7.90x faster**) | `1.42 ms` (🚀 **1.97x faster**)             |
| **`64`**  | `4.26 ms` (✅ **1.00x**)  | `3.18 ms` (✅ **1.34x faster**) | `11.02 ms` (❌ *2.59x slower*)   | `4.99 ms` (❌ *1.17x slower*)    | `569.16 us` (🚀 **7.48x faster**) | `2.09 ms` (🚀 **2.03x faster**)             |
| **`128`** | `15.73 ms` (✅ **1.00x**) | `7.03 ms` (🚀 **2.24x faster**) | `19.90 ms` (❌ *1.27x slower*)   | `12.10 ms` (✅ **1.30x faster**) | `1.25 ms` (🚀 **12.58x faster**)  | `3.37 ms` (🚀 **4.67x faster**)             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

