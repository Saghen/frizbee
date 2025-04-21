# Frizbee

Frizbee is a SIMD fuzzy string matcher written in Rust. The core of the algorithm uses Smith-Waterman with affine gaps, similar to FZF, but with many of the scoring bonuses from FZY. In the included benchmark, with typo resistance disabled, it outperforms nucleo by ~2.5x (116.73us vs 302.35us). It matches against bytes directly, ignoring unicode. Used by [blink.cmp](https://github.com/saghen/blink.cmp) and [blink.pick](https://github.com/saghen/blink.pick).

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
frizbee_0_typos         time:   [116.47 µs 116.73 µs 117.05 µs]
// Performs a prefilter since a set number of typos are allowed,
// set via `max_typos: Some(1)`
frizbee_1_typos         time:   [279.38 µs 281.56 µs 284.10 µs]
frizbee_2_typos         time:   [478.87 µs 479.54 µs 480.29 µs]

nucleo                  time:   [301.74 µs 302.35 µs 303.10 µs]
```

## Algorithm

The core of the algorithm is Smith-Waterman with affine gaps and inter-sequence many-to-one parallelism via SIMD ([ref](https://pmc.ncbi.nlm.nih.gov/articles/PMC8419822/#Sec13)). Besides the parallelism, this is the basis of other popular fuzzy matching algorithms like FZF and Nucleo. The main properties of Smith-Waterman are:

- Always finds the best alignment
- Supports insertion, deletion and substitution
- Does not support transposition (i.e. swapping two adjacent characters)

Due to the inter-sequence parallelism, the algorithm performs best when all the haystacks are the same length (i.e. length 8) for the given SIMD width (i.e. 16 for 256 bit SIMD with u16 scores), see the [implementation](#implementation) section for more details.

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

### Implementation

1. **Prefiltering**: When `max_typos = Some(x)`, perform a fast prefiltering step on the haystacks to exclude any items that can't match the needle
    - **Bitmask:** if `haystack.len() < 24`, creates a u64 bitmask where each bit represents the existence of a character in the range `[33, 90]`. `XOR` the bitmask of the needle and haystack together, to get the number of characters from the needle missing in the haystack. Doesn't check order but combined with the SIMD smith waterman, it's faster to perform a rougher prefiltering step when `haystack.len() < 24`
    - **Memchr:** if `haystack.len() >= 24 && max_typos < 2`, uses the `memchr` crate to ensure the haystack contains the entire needle with a tolerance of `max_typos` missing
    - if neither of the above apply, no prefiltering will be applied
2. **Bucketing**: Group the haystacks by length into buckets of various haystack lengths (`4`, `8`, `12`, ...) until the bucket reaches `$LANES` items, where `$LANES` is the number of available SIMD lanes
    - If the item would cause excessive memory usage, or we don't have a bucket big enough for the haystack (currently max bucket size is `1024`), fallback to a greedy matcher. As a result, it's possible for some items to not appear in the final list, even when `max_typos = None`
3. **Smith Waterman Forward Pass**: When a bucket is full, perform SIMD smith waterman on `$LANES` items at a time
4. **Smith Waterman Backward Pass**: If `max_typos != None` and we didn't use the `memchr` prefilter method, perform a backward (alignment) pass to find the number of typos in the haystack
5. **Finalize:** Optionally sort (`opts.stable_sort | unstable_sort`) and return the matches

## Ideas

- [x] Runtime instruction selection (512-bit and 256-bit SIMD)
- [ ] Calculate alignment directions during matrix building
  - Might speed up typo calculation
- [ ] Prefix matching
- [x] Drop u8 based scoring and double scoring to support longer fuzzy matches
  - Currently, alignment can be lost on longer matches causing us to mark them as having typos
- [x] Incremental matching
  - [ ] Runtime instruction selection (512-bit and 256-bit SIMD)
  - [ ] Prefiltering
  - [ ] Exact match bonus
