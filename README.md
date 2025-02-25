# Frizbee

Frizbee is a dependency free SIMD fuzzy string matcher written in Rust. The core of the algorithm uses Smith-Waterman with affine gaps, similar to FZF, but with many of the scoring bonuses from FZY. In the included benchmark, with typo resistance disabled, it outperforms nucleo by ~2x (18.4us vs 35.8us). It matches against characters directly, ignoring unicode.

## Usage

```rust
use frizbee::*;

let needle = "pri";
let haystacks = ["print", "println", "prelude", "println!"];

let matches = match_list(needle, &haystacks, Options::default());
```

## Benchmarks

Benchmarks were run on a Ryzen 7 3700X, with `-C target-cpu=native`. Results with different needles, partial match percentage, match percentage, median length, and number of samples are in the works. You may test these cases yourself via the included benchmarks.

```rust
needle: "deadbe"
partial_match_percentage: 0.05
match_percentage: 0.05
median_length: 16
std_dev_length: 4
num_samples: 1000

// Gets the scores for all of the items without any filtering
frizbee                 time:   [54.892 µs 55.033 µs 55.209 µs]
// Performs the fastest prefilter since no typos are allowed
// Matches the behavior of fzf/nucleo, set via `max_typos: Some(0)`
frizbee_0_typos         time:   [18.283 µs 18.373 µs 18.482 µs]
// Performs a prefilter since a set number of typos are allowed,
// set via `max_typos: Some(1)`
frizbee_1_typos         time:   [27.963 µs 28.049 µs 28.143 µs]
frizbee_2_typos         time:   [49.100 µs 49.177 µs 49.271 µs]

nucleo                  time:   [35.686 µs 35.765 µs 35.848 µs]
```

## Algorithm

The core of the algorithm is Smith-Waterman with affine gaps and inter-sequence many-to-one parallelism via SIMD ([ref](https://pmc.ncbi.nlm.nih.gov/articles/PMC8419822/#Sec13)). This is the basis of other popular fuzzy matching algorithms like FZF and Nucleo. The main properties of Smith-Waterman are:

- Always finds the best alignment 
- Supports insertion, deletion and substitution
- Does not support transposition (i.e. swapping two adjacent characters)

Due to the inter-sequence parallelism, the algorithm performs best when all the haystacks are the same length (i.e. length 8) for the given SIMD width (i.e. 16 for 128 bit SIMD with u8 scores). The `match_list` function handles this by grouping the haystacks by length into "buckets" of various sizes (`4`, `8`, `12`, ...). Any haystack with length larger than the largest bucket will be discarded, for now.

Nucleo and FZF use a prefiltering step that removes any haystacks that do not include all of the characters in the needle. Frizbee supports this but disables it by default to allow for typos. You may play with the `max_typos` property to control how many typos you allow.

- `MATCH_SCORE`: Score for a match
- `MISMATCH_PENALTY`: Penalty for a mismatch (substitution)
- `GAP_OPEN_PENALTY`: Penalty for opening a gap (deletion/insertion)
- `GAP_EXTEND_PENALTY`: Penalty for extending a gap (deletion/insertion)
- `PREFIX_BONUS`: Bonus for matching the first character of the haystack
- `DELIMITER_BONUS`: Bonus for matching _after_ a delimiter character (e.g. "hw" on "hello_world", will give a bonus on "w")
- `MATCHING_CASE_BONUS`: Bonus for matching the case of the needle (e.g. "WorLd" on "WoRld" will receive a bonus on "W", "o", "d")
- `EXACT_MATCH_BONUS`: Bonus for matching the exact needle (e.g. "foo" on "foo" will receive the bonus)
