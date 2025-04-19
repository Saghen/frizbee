use crate::r#const::*;
use std::simd::{cmp::*, SimdElement};
use std::simd::{Mask, Simd};

pub trait SimdNum<const L: usize>:
    Sized
    + Copy
    + std::fmt::Debug
    + std::simd::SimdElement
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::convert::From<u8>
    + std::convert::Into<u16>
    + std::convert::Into<usize>
    + std::cmp::PartialEq
    + std::cmp::PartialOrd
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    const ZERO: Self;
    const ZERO_VEC: Simd<Self, L>;

    // Delimiters
    const SPACE_DELIMITER: Simd<Self, L>;
    const SLASH_DELIMITER: Simd<Self, L>;
    const DOT_DELIMITER: Simd<Self, L>;
    const COMMA_DELIMITER: Simd<Self, L>;
    const UNDERSCORE_DELIMITER: Simd<Self, L>;
    const DASH_DELIMITER: Simd<Self, L>;
    const COLON_DELIMITER: Simd<Self, L>;
    const DELIMITER_BONUS: Simd<Self, L>;

    // Capitalization
    const CAPITAL_START: Simd<Self, L>;
    const CAPITAL_END: Simd<Self, L>;
    const LOWER_START: Simd<Self, L>;
    const LOWER_END: Simd<Self, L>;
    const TO_LOWERCASE_MASK: Simd<Self, L>;

    // Scoring Params
    const CAPITALIZATION_BONUS: Simd<Self, L>;
    const MATCHING_CASE_BONUS: Simd<Self, L>;

    const GAP_OPEN_PENALTY: Simd<Self, L>;
    const GAP_EXTEND_PENALTY: Simd<Self, L>;
    const MATCH_SCORE: Simd<Self, L>;
    const MISMATCH_PENALTY: Simd<Self, L>;
    const PREFIX_MATCH_SCORE: Simd<Self, L>;

    fn from_usize(n: usize) -> Self;
}

pub trait SimdVec<N: SimdNum<L>, const L: usize>:
    Sized
    + Copy
    + std::ops::Add<Output = Simd<N, L>>
    + std::ops::BitOr<Output = Simd<N, L>>
    + std::simd::cmp::SimdPartialEq<Mask = Mask<<N as SimdElement>::Mask, L>>
    + std::simd::cmp::SimdOrd
    + std::simd::num::SimdUint
where
    N: SimdNum<L>,
    N::Mask: std::simd::MaskElement,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
}

pub trait SimdMask<N: SimdNum<L>, const L: usize>:
    Sized
    + Copy
    + std::ops::Not<Output = Mask<N::Mask, L>>
    + std::ops::BitAnd<Output = Mask<N::Mask, L>>
    + std::ops::BitOr<Output = Mask<N::Mask, L>>
    + std::simd::cmp::SimdPartialEq<Mask = Mask<<N as SimdElement>::Mask, L>>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
}

macro_rules! simd_num_impl {
    ($type:ident,$($lanes:literal),+) => {
        $(
            impl SimdNum<$lanes> for $type {
                const ZERO: Self = 0;
                const ZERO_VEC: Simd<Self, $lanes> = Simd::from_array([0; $lanes]);

                const SPACE_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b' ' as $type; $lanes]);
                const SLASH_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'/' as $type; $lanes]);
                const DOT_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'.' as $type; $lanes]);
                const COMMA_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b',' as $type; $lanes]);
                const UNDERSCORE_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'_' as $type; $lanes]);
                const DASH_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'-' as $type; $lanes]);
                const COLON_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b':' as $type; $lanes]);
                const DELIMITER_BONUS: Simd<Self, $lanes> = Simd::from_array([DELIMITER_BONUS as $type; $lanes]);

                const CAPITAL_START: Simd<Self, $lanes> = Simd::from_array([b'A' as $type; $lanes]);
                const CAPITAL_END: Simd<Self, $lanes> = Simd::from_array([b'Z' as $type; $lanes]);
                const LOWER_START: Simd<Self, $lanes> = Simd::from_array([b'a' as $type; $lanes]);
                const LOWER_END: Simd<Self, $lanes> = Simd::from_array([b'z' as $type; $lanes]);
                const TO_LOWERCASE_MASK: Simd<Self, $lanes> = Simd::from_array([0x20; $lanes]);

                const CAPITALIZATION_BONUS: Simd<Self, $lanes> = Simd::from_array([CAPITALIZATION_BONUS as $type; $lanes]);
                const MATCHING_CASE_BONUS: Simd<Self, $lanes> = Simd::from_array([MATCHING_CASE_BONUS as $type; $lanes]);

                const GAP_OPEN_PENALTY: Simd<Self, $lanes> = Simd::from_array([GAP_OPEN_PENALTY as $type; $lanes]);
                const GAP_EXTEND_PENALTY: Simd<Self, $lanes> = Simd::from_array([GAP_EXTEND_PENALTY as $type; $lanes]);
                const MATCH_SCORE: Simd<Self, $lanes> = Simd::from_array([MATCH_SCORE as $type; $lanes]);
                const MISMATCH_PENALTY: Simd<Self, $lanes> = Simd::from_array([MISMATCH_PENALTY as $type; $lanes]);
                const PREFIX_MATCH_SCORE: Simd<Self, $lanes> = Simd::from_array([(MATCH_SCORE + PREFIX_BONUS) as $type; $lanes]);

                #[inline(always)]
                fn from_usize(n: usize) -> Self {
                    n as $type
                }
            }
            impl SimdVec<$type, $lanes> for Simd<$type, $lanes> {}
            impl SimdMask<$type, $lanes> for Mask<<$type as SimdElement>::Mask, $lanes> {}
        )+
    };
}
simd_num_impl!(u16, 1, 2, 4, 8, 16, 32);

#[inline(always)]
pub(crate) fn simd_to_lowercase_with_mask<N, const L: usize>(
    data: Simd<N, L>,
) -> (Mask<N::Mask, L>, Mask<N::Mask, L>, Simd<N, L>)
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    let is_capital_mask: Mask<N::Mask, L> =
        data.simd_ge(N::CAPITAL_START) & data.simd_le(N::CAPITAL_END);
    let is_lower_mask: Mask<N::Mask, L> = data.simd_ge(N::LOWER_START) & data.simd_le(N::LOWER_END);
    let lowercase = data | is_capital_mask.select(N::TO_LOWERCASE_MASK, N::ZERO_VEC);
    (is_capital_mask, is_lower_mask, lowercase)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NeedleChar<N, const L: usize>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    pub(crate) lowercase: Simd<N, L>,
    pub(crate) is_capital_mask: Mask<N::Mask, L>,
}
impl<N, const L: usize> NeedleChar<N, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    #[inline(always)]
    pub(crate) fn new(char: N) -> Self {
        let (is_capital_mask, _, lowercase) =
            simd_to_lowercase_with_mask::<N, L>(Simd::splat(char));
        Self {
            lowercase,
            is_capital_mask,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HaystackChar<N, const L: usize>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    pub(crate) lowercase: Simd<N, L>,
    pub(crate) is_lower_mask: Mask<N::Mask, L>,
    pub(crate) is_capital_mask: Mask<N::Mask, L>,
    pub(crate) is_delimiter_mask: Mask<N::Mask, L>,
}
impl<N, const L: usize> HaystackChar<N, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    #[inline(always)]
    pub(crate) fn new(chars: Simd<N, L>) -> Self {
        let (is_capital_mask, is_lower_mask, lowercase) =
            simd_to_lowercase_with_mask::<N, L>(chars);
        let is_delimiter_mask: Mask<N::Mask, L> = N::SPACE_DELIMITER.simd_eq(lowercase)
            | N::SLASH_DELIMITER.simd_eq(lowercase)
            | N::DOT_DELIMITER.simd_eq(lowercase)
            | N::COMMA_DELIMITER.simd_eq(lowercase)
            | N::UNDERSCORE_DELIMITER.simd_eq(lowercase)
            | N::DASH_DELIMITER.simd_eq(lowercase)
            | N::SPACE_DELIMITER.simd_eq(lowercase);
        Self {
            lowercase,
            is_lower_mask,
            is_capital_mask,
            is_delimiter_mask,
        }
    }

    #[inline(always)]
    pub(crate) fn from_haystacks(haystacks: &[&str; L], i: usize) -> Self {
        // Convert haystacks to a static array of bytes chunked for SIMD
        let chars = std::array::from_fn(|j| {
            N::from(*haystacks[j].as_bytes().get(i).to_owned().unwrap_or(&0))
        });
        // pre-compute haystack case mask, delimiter mask, and lowercase
        HaystackChar::new(Simd::from_array(chars))
    }
}

impl<N, const L: usize> Default for HaystackChar<N, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    fn default() -> Self {
        Self {
            lowercase: N::ZERO_VEC,
            is_lower_mask: Mask::splat(false),
            is_capital_mask: Mask::splat(false),
            is_delimiter_mask: Mask::splat(false),
        }
    }
}
