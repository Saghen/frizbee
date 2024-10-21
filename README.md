# Frizbee

Frizbee is a SIMD fuzzy string matcher written in Rust. The core of the algorithm uses Smith-Waterman with affine gaps, similar to FZF, but with many of the scoring bonuses from FZY. In the included benchmark, with typo resistance disabled, it outperforms nucleo by 2.9x (1.3ms vs 3.8ms). However, it supports matching against ASCII only, with plans to support Unicode.

## Usage

```rust
use frizbee::*;

let needle = "banny";
let haystacks = [
    "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
    "print", "println", "prelude", "println!", "prefetch", "prefix", "prefix!", "print!",
];

let matches = match_list(needle, &haystacks, Options::default());
```

## Algorithm

The core of the algorithm is Smith-Waterman with affine gaps and inter-sequence many-to-one parallelism via SIMD ([ref](https://pmc.ncbi.nlm.nih.gov/articles/PMC8419822/#Sec13)). This is the basis of other popular fuzzy matching algorithms like FZF and Nucleo. The main properties of Smith-Waterman are:

- Always finds the best alignment 
- Supports insertion, deletion and substitution
- Does not support transposition (i.e. swapping two adjacent characters)

Due to the inter-sequence parallelism, the algorithm performs best when all the haystacks are the same length (i.e. length 8) for the given SIMD width (i.e. 16 for 128 bit SIMD with u8 scores). The `match_list` function handles this by grouping the haystacks by length into "buckets" of various sizes (`4`, `8`, `12`, ...). Any haystack with length larger than the largest bucket will be discarded, for now.

Many scoring ideas are ~~stolen~~ borrowed from FZY, but implemented in SIMD so the implementation may be slightly different. The scoring parameters are:

- `MATCH_SCORE`: Score for a match
- `MISMATCH_PENALTY`: Penalty for a mismatch (substitution)
- `GAP_OPEN_PENALTY`: Penalty for opening a gap (deletion/insertion)
- `GAP_EXTEND_PENALTY`: Penalty for extending a gap (deletion/insertion)
- `PREFIX_BONUS`: Bonus for matching the first character of the haystack
- `DELIMITER_BONUS`: Bonus for matching _after_ a delimiter character (e.g. "hw" on "hello_world", will give a bonus on "w")
- `MATCHING_CASE_BONUS`: Bonus for matching the case of the needle (e.g. "WorLd" on "WoRld" will receive a bonus on "W", "o", "d")
- `EXACT_MATCH_BONUS`: Bonus for matching the exact needle (e.g. "foo" on "foo" will receive the bonus)
