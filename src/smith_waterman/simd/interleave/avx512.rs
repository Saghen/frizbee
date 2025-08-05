use std::{
    arch::x86_64::*,
    simd::{num::SimdUint, Simd},
};

#[target_feature(enable = "avx2", enable = "avx512f", enable = "avx512bw")]
pub fn interleave<const W: usize>(strs: [&str; 32]) -> [Simd<u16, 32>; W] {
    let str_bytes: [&[u8]; 32] = std::array::from_fn(|i| strs[i].as_bytes());
    let str_lens: [usize; 32] = std::array::from_fn(|i| str_bytes[i].len());

    let mut interleaved = [Simd::splat(0); W];
    for offset in (0..W).step_by(32) {
        let simds = to_simd(str_bytes, str_lens, offset);
        let interleaved_chunk = interleave_chunk(simds).map(|s| s.cast::<u16>());

        let copy_len = (W - offset).min(32);
        interleaved[offset..offset + copy_len].copy_from_slice(&interleaved_chunk[0..copy_len]);
    }

    interleaved
}

#[inline(always)]
fn to_simd(str_bytes: [&[u8]; 32], str_lens: [usize; 32], offset: usize) -> [__m256i; 32] {
    unsafe {
        std::array::from_fn(|i| {
            let len = str_lens[i];
            if offset >= len {
                // Beyond string length - return zeros
                return _mm256_setzero_si256();
            }

            let remaining = len - offset;
            let load_len = remaining.min(32);

            if load_len == 32 {
                _mm256_loadu_si256(str_bytes[i][offset..].as_ptr() as *const __m256i)
            } else {
                let mut temp = _mm256_setzero_si256();
                std::ptr::copy_nonoverlapping(
                    str_bytes[i][offset..].as_ptr(),
                    &mut temp as *mut __m256i as *mut u8,
                    load_len,
                );
                temp
            }
        })
    }
}

#[inline(always)]
pub fn interleave_chunk(mut simds: [__m256i; 32]) -> [Simd<u8, 32>; 32] {
    unsafe {
        // distance = 16
        for i in 0..16 {
            let (lo, hi) = interleave_u8x32(simds[i], simds[i + 16]);
            simds[i] = lo;
            simds[i + 16] = hi;
        }

        // distance = 8
        for base in (0..32).step_by(16) {
            for i in 0..8 {
                let (lo, hi) = interleave_u8x32(simds[base + i], simds[base + i + 8]);
                simds[base + i] = lo;
                simds[base + i + 8] = hi;
            }
        }

        // distance = 4
        for base in (0..32).step_by(8) {
            for i in 0..4 {
                let (lo, hi) = interleave_u8x32(simds[base + i], simds[base + i + 4]);
                simds[base + i] = lo;
                simds[base + i + 4] = hi;
            }
        }

        // distance = 2
        for base in (0..32).step_by(4) {
            for i in 0..2 {
                let (lo, hi) = interleave_u8x32(simds[base + i], simds[base + i + 2]);
                simds[base + i] = lo;
                simds[base + i + 2] = hi;
            }
        }

        // distance = 1
        for base in (0..32).step_by(2) {
            let (lo, hi) = interleave_u8x32(simds[base], simds[base + 1]);
            simds[base] = lo;
            simds[base + 1] = hi;
        }

        std::mem::transmute::<[__m256i; 32], [Simd<u8, 32>; 32]>(simds)
    }
}

#[inline(always)]
unsafe fn interleave_u8x32(a: __m256i, b: __m256i) -> (__m256i, __m256i) { unsafe {
    // Use vpunpcklwd and vpunpckhwd for 16-bit interleaving
    let lo = _mm256_unpacklo_epi8(a, b);
    let hi = _mm256_unpackhi_epi8(a, b);

    // Fix the lane crossing issue in AVX2
    let lo_fixed = _mm256_permute4x64_epi64(lo, 0b11011000); // 0xD8
    let hi_fixed = _mm256_permute4x64_epi64(hi, 0b11011000);

    (lo_fixed, hi_fixed)
}}
