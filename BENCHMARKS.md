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

|           | `Nucleo`                 | `Frizbee`                      | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`              | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:-------------------------------|:--------------------------------|:--------------------------------|:----------------------------------|:------------------------------------------ |
| **`16`**  | `3.54 ms` (✅ **1.00x**)  | `1.46 ms` (🚀 **2.42x faster**) | `3.94 ms` (❌ *1.11x slower*)    | `2.92 ms` (✅ **1.21x faster**)  | `303.43 us` (🚀 **11.68x faster**) | `1.08 ms` (🚀 **3.27x faster**)             |
| **`32`**  | `5.45 ms` (✅ **1.00x**)  | `3.40 ms` (✅ **1.60x faster**) | `6.18 ms` (❌ *1.13x slower*)    | `5.25 ms` (✅ **1.04x faster**)  | `648.31 us` (🚀 **8.40x faster**)  | `1.40 ms` (🚀 **3.89x faster**)             |
| **`64`**  | `8.56 ms` (✅ **1.00x**)  | `4.94 ms` (✅ **1.73x faster**) | `11.16 ms` (❌ *1.30x slower*)   | `7.41 ms` (✅ **1.15x faster**)  | `885.91 us` (🚀 **9.66x faster**)  | `2.06 ms` (🚀 **4.15x faster**)             |
| **`128`** | `25.08 ms` (✅ **1.00x**) | `9.75 ms` (🚀 **2.57x faster**) | `19.78 ms` (✅ **1.27x faster**) | `15.07 ms` (✅ **1.66x faster**) | `1.72 ms` (🚀 **14.55x faster**)   | `3.27 ms` (🚀 **7.67x faster**)             |

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
| **`16`**  | `22.83 ms` (✅ **1.00x**)  | `5.48 ms` (🚀 **4.17x faster**)  | `4.05 ms` (🚀 **5.64x faster**)  | `8.98 ms` (🚀 **2.54x faster**)  | `1.11 ms` (🚀 **20.65x faster**) | `1.13 ms` (🚀 **20.23x faster**)            |
| **`32`**  | `38.62 ms` (✅ **1.00x**)  | `13.60 ms` (🚀 **2.84x faster**) | `6.36 ms` (🚀 **6.08x faster**)  | `16.66 ms` (🚀 **2.32x faster**) | `2.54 ms` (🚀 **15.23x faster**) | `1.43 ms` (🚀 **26.96x faster**)            |
| **`64`**  | `62.80 ms` (✅ **1.00x**)  | `18.59 ms` (🚀 **3.38x faster**) | `10.99 ms` (🚀 **5.72x faster**) | `22.97 ms` (🚀 **2.73x faster**) | `3.30 ms` (🚀 **19.05x faster**) | `2.13 ms` (🚀 **29.42x faster**)            |
| **`128`** | `117.87 ms` (✅ **1.00x**) | `28.51 ms` (🚀 **4.13x faster**) | `19.88 ms` (🚀 **5.93x faster**) | `31.23 ms` (🚀 **3.77x faster**) | `4.27 ms` (🚀 **27.62x faster**) | `3.22 ms` (🚀 **36.60x faster**)            |

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

|           | `Nucleo`                 | `Frizbee`                        | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`             | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:---------------------------------|:--------------------------------|:--------------------------------|:---------------------------------|:------------------------------------------ |
| **`16`**  | `2.07 ms` (✅ **1.00x**)  | `972.98 us` (🚀 **2.13x faster**) | `3.94 ms` (❌ *1.90x slower*)    | `1.96 ms` (✅ **1.06x faster**)  | `212.16 us` (🚀 **9.77x faster**) | `1.07 ms` (🚀 **1.94x faster**)             |
| **`32`**  | `3.21 ms` (✅ **1.00x**)  | `2.36 ms` (✅ **1.36x faster**)   | `6.22 ms` (❌ *1.94x slower*)    | `3.78 ms` (❌ *1.18x slower*)    | `398.60 us` (🚀 **8.05x faster**) | `1.41 ms` (🚀 **2.28x faster**)             |
| **`64`**  | `4.72 ms` (✅ **1.00x**)  | `3.69 ms` (✅ **1.28x faster**)   | `11.03 ms` (❌ *2.33x slower*)   | `5.56 ms` (❌ *1.18x slower*)    | `606.87 us` (🚀 **7.78x faster**) | `2.07 ms` (🚀 **2.28x faster**)             |
| **`128`** | `16.23 ms` (✅ **1.00x**) | `7.65 ms` (🚀 **2.12x faster**)   | `19.93 ms` (❌ *1.23x slower*)   | `12.67 ms` (✅ **1.28x faster**) | `1.29 ms` (🚀 **12.63x faster**)  | `3.25 ms` (🚀 **5.00x faster**)             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

