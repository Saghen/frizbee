# Frizbee

Frizbee is a SIMD fuzzy string matcher written in Rust. The core of the algorithm uses Smith-Waterman with affine gaps, similar to FZF, but with many of the scoring bonuses from FZY. In the included benchmark, with typo resistance disabled, it outperforms nucleo by ~2.3x (130.70us vs 302.35us). It matches against bytes directly, ignoring unicode. Used by [blink.cmp](https://github.com/saghen/blink.cmp) and [blink.pick](https://github.com/saghen/blink.pick).

## Usage

```rust
use frizbee::*;

let needle = "pri";
let haystacks = ["print", "println", "prelude", "println!"];

let matches = match_list(needle, &haystacks, Options::default());
```

## Benchmarks

Benchmarks were run on a Ryzen 9950X3D. Results with different needles, partial match percentage, match percentage, median length, and number of samples are in the works. You may test these cases yourself via the included benchmarks.

```rust
needle: "deadbe"
partial_match_percentage: 0.05
match_percentage: 0.05
median_length: 16
std_dev_length: 4
num_samples: 10000

// Gets the scores for all of the items without any filtering
frizbee                 time:   [367.25 µs 368.44 µs 369.87 µs]
// Performs the fastest prefilter since no typos are allowed
// Matches the behavior of fzf/nucleo, set via `max_typos: Some(0)`
frizbee_0_typos         time:   [130.14 µs 130.70 µs 131.33 µs]
// Performs a prefilter since a set number of typos are allowed,
// set via `max_typos: Some(1)`
frizbee_1_typos         time:   [280.29 µs 283.53 µs 286.87 µs]
frizbee_2_typos         time:   [481.09 µs 482.44 µs 484.03 µs]

nucleo                  time:   [301.74 µs 302.35 µs 303.10 µs]
```

## Algorithm

The core of the algorithm is Smith-Waterman with affine gaps and inter-sequence many-to-one parallelism via SIMD ([ref](https://pmc.ncbi.nlm.nih.gov/articles/PMC8419822/#Sec13)). This is the basis of other popular fuzzy matching algorithms like FZF and Nucleo. The main properties of Smith-Waterman are:

- Always finds the best alignment
- Supports insertion, deletion and substitution
- Does not support transposition (i.e. swapping two adjacent characters)

Due to the inter-sequence parallelism, the algorithm performs best when all the haystacks are the same length (i.e. length 8) for the given SIMD width (i.e. 16 for 256 bit SIMD with u16 scores). The `match_list` function handles this by grouping the haystacks by length into "buckets" of various sizes (`4`, `8`, `12`, ...). Any haystack with length larger than the largest bucket will be discarded, for now. 

The SIMD width will be chosen at runtime based on available instruction set extensions. Currently, only x86_64's AVX2 (256-bit) and AVX512 (512-bit) will be detected at runtime, falling back to 128-bit SIMD if neither is available.

Nucleo and FZF use a prefiltering step that removes any haystacks that do not include all of the characters in the needle. Frizbee supports this but disables it by default to allow for typos. You may control the maximum number of typos with the `max_typos` property.

- `MATCH_SCORE`: Score for a match
- `MISMATCH_PENALTY`: Penalty for a mismatch (substitution)
- `GAP_OPEN_PENALTY`: Penalty for opening a gap (deletion/insertion)
- `GAP_EXTEND_PENALTY`: Penalty for extending a gap (deletion/insertion)
- `PREFIX_BONUS`: Bonus for matching the first character of the haystack
- `DELIMITER_BONUS`: Bonus for matching _after_ a delimiter character (e.g. "hw" on "hello_world", will give a bonus on "w")
- `CAPITALIZATION_BONUS`: Bonus for matching a capital letter after a lowercase letter (e.g. "b" on "fooBar" will receive a bonus on "B")
- `MATCHING_CASE_BONUS`: Bonus for matching the case of the needle (e.g. "WorLd" on "WoRld" will receive a bonus on "W", "o", "d")
- `EXACT_MATCH_BONUS`: Bonus for matching the exact needle (e.g. "foo" on "foo" will receive the bonus)

## Ideas

- [x] Runtime instruction selection (512-bit and 256-bit SIMD)
- [ ] Calculate alignment directions during matrix building
  - It should be possible to calculate the backtrace direction for the alignment phase while we're building the matrix, which could speed up typo calculation
- [ ] Prefix matching
- [x] Drop u8 based scoring and double scoring to support longer fuzzy matches
  - Currently, alignment can be lost on longer matches causing us to mark them as having typos
- [x] Incremental matching
  - [ ] Runtime instruction selection (512-bit and 256-bit SIMD)
  - [ ] Prefiltering
  - [ ] Exact match bonus
