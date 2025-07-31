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

|           | `Frizbee`                | `Nucleo`                        | `Frizbee: All Scores`           | `Frizbee: 1 Typo`               | `Frizbee (Parallel)`             | `Frizbee: All Scores (Parallel)`           |
|:----------|:-------------------------|:--------------------------------|:--------------------------------|:--------------------------------|:---------------------------------|:------------------------------------------ |
| **`16`**  | `1.75 ms` (✅ **1.00x**)  | `3.31 ms` (❌ *1.89x slower*)    | `5.52 ms` (❌ *3.16x slower*)    | `3.02 ms` (❌ *1.73x slower*)    | `329.42 us` (🚀 **5.31x faster**) | `1.28 ms` (✅ **1.36x faster**)             |
| **`32`**  | `3.16 ms` (✅ **1.00x**)  | `5.04 ms` (❌ *1.59x slower*)    | `11.01 ms` (❌ *3.48x slower*)   | `5.04 ms` (❌ *1.59x slower*)    | `601.72 us` (🚀 **5.26x faster**) | `2.01 ms` (✅ **1.57x faster**)             |
| **`64`**  | `4.88 ms` (✅ **1.00x**)  | `8.11 ms` (❌ *1.66x slower*)    | `19.57 ms` (❌ *4.01x slower*)   | `7.60 ms` (❌ *1.56x slower*)    | `874.62 us` (🚀 **5.58x faster**) | `3.13 ms` (✅ **1.56x faster**)             |
| **`128`** | `11.74 ms` (✅ **1.00x**) | `24.75 ms` (❌ *2.11x slower*)   | `35.68 ms` (❌ *3.04x slower*)   | `19.38 ms` (❌ *1.65x slower*)   | `1.94 ms` (🚀 **6.06x faster**)   | `5.24 ms` (🚀 **2.24x faster**)             |

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
| **`16`**  | `6.77 ms` (✅ **1.00x**)  | `21.92 ms` (❌ *3.24x slower*)    | `5.58 ms` (✅ **1.21x faster**)  | `11.09 ms` (❌ *1.64x slower*)   | `1.28 ms` (🚀 **5.29x faster**) | `1.30 ms` (🚀 **5.22x faster**)             |
| **`32`**  | `16.18 ms` (✅ **1.00x**) | `38.45 ms` (❌ *2.38x slower*)    | `10.91 ms` (✅ **1.48x faster**) | `19.57 ms` (❌ *1.21x slower*)   | `2.77 ms` (🚀 **5.83x faster**) | `2.03 ms` (🚀 **7.96x faster**)             |
| **`64`**  | `24.68 ms` (✅ **1.00x**) | `63.17 ms` (❌ *2.56x slower*)    | `19.44 ms` (✅ **1.27x faster**) | `27.92 ms` (❌ *1.13x slower*)   | `3.90 ms` (🚀 **6.32x faster**) | `3.15 ms` (🚀 **7.83x faster**)             |
| **`128`** | `41.43 ms` (✅ **1.00x**) | `119.46 ms` (❌ *2.88x slower*)   | `35.51 ms` (✅ **1.17x faster**) | `44.76 ms` (✅ **1.08x slower**) | `5.97 ms` (🚀 **6.94x faster**) | `5.22 ms` (🚀 **7.93x faster**)             |

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
| **`16`**  | `1.19 ms` (✅ **1.00x**) | `1.94 ms` (❌ *1.63x slower*)    | `5.49 ms` (❌ *4.61x slower*)    | `1.94 ms` (❌ *1.63x slower*)    | `242.04 us` (🚀 **4.92x faster**) | `1.29 ms` (✅ **1.08x slower**)             |
| **`32`**  | `2.05 ms` (✅ **1.00x**) | `2.81 ms` (❌ *1.37x slower*)    | `10.86 ms` (❌ *5.29x slower*)   | `3.28 ms` (❌ *1.60x slower*)    | `346.97 us` (🚀 **5.92x faster**) | `2.07 ms` (✅ **1.01x slower**)             |
| **`64`**  | `3.24 ms` (✅ **1.00x**) | `4.23 ms` (❌ *1.30x slower*)    | `19.54 ms` (❌ *6.03x slower*)   | `5.09 ms` (❌ *1.57x slower*)    | `535.61 us` (🚀 **6.05x faster**) | `3.16 ms` (✅ **1.03x faster**)             |
| **`128`** | `8.49 ms` (✅ **1.00x**) | `15.72 ms` (❌ *1.85x slower*)   | `35.49 ms` (❌ *4.18x slower*)   | `15.85 ms` (❌ *1.87x slower*)   | `1.38 ms` (🚀 **6.18x faster**)   | `5.24 ms` (✅ **1.62x faster**)             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

