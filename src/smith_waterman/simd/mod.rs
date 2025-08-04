//! SIMD implementation of the smith waterman algorithm with inter-sequence many-to-one
//! parallelism. AVX512 and AVX2 will be detected at runtime, falling back to 128-bit SIMD. You
//! should use a maximum of 32 lanes, as this relates to 512-bit wide SIMD registers.

mod algorithm;
mod indices;
mod interleave;
mod types;
mod typos;

pub use algorithm::*;
pub use indices::*;
pub use interleave::*;
pub(crate) use types::*;
pub use typos::*;
