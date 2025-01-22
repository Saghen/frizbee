# Frizbee

Frizbee is a SIMD fuzzy string matcher written in Rust. The core of the algorithm uses Smith-Waterman with affine gaps, similar to FZF, but with many of the scoring bonuses from FZY. In the included benchmark, with typo resistance disabled, it outperforms nucleo by 1.65x (23.2us vs 38.3us). It supports matching against ASCII only, with plans to support Unicode.

## Usage

```rust
use frizbee::*;

let needle = "pri";
let haystacks = [
    "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
    "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
];

let matches = match_list(needle, &haystacks, Options::default());
```

## Benchmarks

Benchmarks were run on a Ryzen 7 3700X, with `-C target-cpu=native`. Results with different needle, partial match percentage, match percentage, median length, and number of samples are in the works. You may test these cases yourself via the included benchmarks.

```rust
needle: "deadbe"
partial_match_percentage: 0.05
match_percentage: 0.05
median_length: 16
std_dev_length: 4
num_samples: 1000

// Gets the scores for all of the items without any filtering
frizbee                 time:   [55.135 µs 55.233 µs 55.358 µs]
// Performs a fast prefilter since no typos are allowed
// Matches the behavior of fzf/nucleo, set via `max_typos: Some(0)`
frizbee_0_typos         time:   [21.178 µs 21.290 µs 21.464 µs]
// Performs a prefilter since 1 typo is allowed, set via `max_typos: Some(1)`
frizbee_1_typos         time:   [39.701 µs 39.796 µs 39.912 µs]
// Performs no prefiltering, and calculates the number of typos
// from the smith waterman score matrix
frizbee_2_typos         time:   [61.491 µs 61.657 µs 61.851 µs]

nucleo                  time:   [38.105 µs 38.338 µs 38.657 µs]
```

## Algorithm

The core of the algorithm is Smith-Waterman with affine gaps and inter-sequence many-to-one parallelism via SIMD ([ref](https://pmc.ncbi.nlm.nih.gov/articles/PMC8419822/#Sec13)). This is the basis of other popular fuzzy matching algorithms like FZF and Nucleo. The main properties of Smith-Waterman are:

- Always finds the best alignment 
- Supports insertion, deletion and substitution
- Does not support transposition (i.e. swapping two adjacent characters)

Due to the inter-sequence parallelism, the algorithm performs best when all the haystacks are the same length (i.e. length 8) for the given SIMD width (i.e. 16 for 128 bit SIMD with u8 scores). The `match_list` function handles this by grouping the haystacks by length into "buckets" of various sizes (`4`, `8`, `12`, ...). Any haystack with length larger than the largest bucket will be discarded, for now.

Nucleo and FZF use a prefiltering step that removes any haystacks that do not include all of the characters in the needle. Frizbee supports this but disables it by default to allow for typos. You may play with the `min_score` property to control how many typos you allow. A good default may be `6 * needle.len()`.

Many scoring ideas are ~~stolen~~ borrowed from FZY, but implemented in SIMD so the implementation may be slightly different. The scoring parameters are:

- `MATCH_SCORE`: Score for a match
- `MISMATCH_PENALTY`: Penalty for a mismatch (substitution)
- `GAP_OPEN_PENALTY`: Penalty for opening a gap (deletion/insertion)
- `GAP_EXTEND_PENALTY`: Penalty for extending a gap (deletion/insertion)
- `PREFIX_BONUS`: Bonus for matching the first character of the haystack
- `DELIMITER_BONUS`: Bonus for matching _after_ a delimiter character (e.g. "hw" on "hello_world", will give a bonus on "w")
- `MATCHING_CASE_BONUS`: Bonus for matching the case of the needle (e.g. "WorLd" on "WoRld" will receive a bonus on "W", "o", "d")
- `EXACT_MATCH_BONUS`: Bonus for matching the exact needle (e.g. "foo" on "foo" will receive the bonus)
