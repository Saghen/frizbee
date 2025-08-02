use std::simd::{LaneCount, Simd, SupportedLaneCount};

use crate::smith_waterman::simd::{
    avx2::interleave_simd_avx2, avx512::interleave_simd_avx512,
    interleave::generic::interleave_simd_generic,
};

pub mod avx2;
pub mod avx512;
pub mod generic;

#[inline(never)]
pub fn interleave_simd<const W: usize, const L: usize>(strs: [&str; L]) -> [Simd<u16, L>; W]
where
    LaneCount<L>: SupportedLaneCount,
{
    if L == 32
        && is_x86_feature_detected!("avx512f")
        && is_x86_feature_detected!("avx512bw")
        && is_x86_feature_detected!("avx2")
    {
        return unsafe {
            let strs_ptr = &strs as *const [&str; L] as *const [&str; 32];
            let result = interleave_simd_avx512::<W>(*strs_ptr);
            let result_ptr = &result as *const [Simd<u16, 32>; W] as *const [Simd<u16, L>; W];
            std::ptr::read(result_ptr)
        };
    } else if L == 16 && is_x86_feature_detected!("avx2") && is_x86_feature_detected!("sse2") {
        return unsafe {
            let strs_ptr = &strs as *const [&str; L] as *const [&str; 16];
            let result = interleave_simd_avx2::<W>(*strs_ptr);
            let result_ptr = &result as *const [Simd<u16, 16>; W] as *const [Simd<u16, L>; W];
            std::ptr::read(result_ptr)
        };
    }

    interleave_simd_generic::<W, L>(strs)
}
