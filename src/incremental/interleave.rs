use std::arch::x86_64::*;
use std::mem::transmute;
use std::simd::Simd;
use std::simd::num::SimdUint;

pub fn interleave<const W: usize>(strs: [[u8; W]; 16]) -> [Simd<u16, 16>; W] {
    let mut interleaved: [Simd<u16, 16>; W] = [Simd::splat(0); W];

    for offset in (0..W).step_by(16) {
        let simds = to_simd::<W>(strs, offset);
        let interleaved_chunk = interleave_chunk(simds);

        let copy_len = (W - offset).min(16);
        interleaved[offset..(offset + copy_len)].copy_from_slice(&interleaved_chunk[0..copy_len]);
    }

    interleaved
}

pub fn interleave_u16<const W: usize>(strs: [[u16; W]; 16]) -> [Simd<u16, 16>; W] {
    let mut interleaved: [Simd<u16, 16>; W] = [Simd::splat(0); W];

    for offset in (0..W).step_by(16) {
        let simds = to_simd_u16::<W>(strs, offset);
        let interleaved_chunk = interleave_chunk_u16(simds);

        let copy_len = (W - offset).min(16);
        interleaved[offset..(offset + copy_len)].copy_from_slice(&interleaved_chunk[0..copy_len]);
    }

    interleaved
}

pub fn deinterleave_u16<const W: usize>(simds: [Simd<u16, 16>; W]) -> [[u16; W]; 16] {
    let mut deinterleaved = [[0u16; W]; 16];

    for offset in (0..W).step_by(16) {
        // TODO:
        if offset + 16 > W {
            break;
        }

        let simds: [Simd<u16, 16>; 16] = simds[offset..(offset + 16)].try_into().unwrap();
        let simds = unsafe { transmute::<[Simd<u16, 16>; 16], [__m256i; 16]>(simds) };
        let deinterleaved_chunk = interleave_chunk_u16(simds);

        let copy_len = (W - offset).min(16);
        for i in 0..16 {
            deinterleaved[i][offset..(offset + copy_len)]
                .copy_from_slice(&deinterleaved_chunk[i].to_array()[0..copy_len]);
        }
    }

    deinterleaved
}

#[inline]
fn to_simd<const W: usize>(str_bytes: [[u8; W]; 16], offset: usize) -> [__m128i; 16] {
    unsafe {
        std::array::from_fn(|i| _mm_loadu_si128(str_bytes[i][offset..].as_ptr() as *const __m128i))
    }
}

#[inline]
fn to_simd_u16<const W: usize>(str_bytes: [[u16; W]; 16], offset: usize) -> [__m256i; 16] {
    unsafe {
        std::array::from_fn(|i| _mm256_loadu_epi16(str_bytes[i][offset..].as_ptr() as *const i16))
    }
}

#[inline]
fn interleave_chunk(mut simds: [__m128i; 16]) -> [Simd<u16, 16>; 16] {
    unsafe {
        // distance = 8
        for i in 0..8 {
            let (lo, hi) = interleave_u8x16(simds[i], simds[i + 8]);
            simds[i] = lo;
            simds[i + 8] = hi;
        }

        // distance = 4
        for base in (0..16).step_by(8) {
            for i in 0..4 {
                let (lo, hi) = interleave_u8x16(simds[base + i], simds[base + i + 4]);
                simds[base + i] = lo;
                simds[base + i + 4] = hi;
            }
        }

        // distance = 2
        for base in (0..16).step_by(4) {
            for i in 0..2 {
                let (lo, hi) = interleave_u8x16(simds[base + i], simds[base + i + 2]);
                simds[base + i] = lo;
                simds[base + i + 2] = hi;
            }
        }

        // distance = 1
        for base in (0..16).step_by(2) {
            let (lo, hi) = interleave_u8x16(simds[base], simds[base + 1]);
            simds[base] = lo;
            simds[base + 1] = hi;
        }

        let simds = std::mem::transmute::<[__m128i; 16], [Simd<u8, 16>; 16]>(simds);

        // Convert u8x16 to u16x16
        simds.map(|s| s.cast::<u16>())
    }
}

#[inline]
fn interleave_chunk_u16(mut simds: [__m256i; 16]) -> [Simd<u16, 16>; 16] {
    unsafe {
        // distance = 8
        for i in 0..8 {
            let (lo, hi) = interleave_u16x16(simds[i], simds[i + 8]);
            simds[i] = lo;
            simds[i + 8] = hi;
        }

        // distance = 4
        for base in (0..16).step_by(8) {
            for i in 0..4 {
                let (lo, hi) = interleave_u16x16(simds[base + i], simds[base + i + 4]);
                simds[base + i] = lo;
                simds[base + i + 4] = hi;
            }
        }

        // distance = 2
        for base in (0..16).step_by(4) {
            for i in 0..2 {
                let (lo, hi) = interleave_u16x16(simds[base + i], simds[base + i + 2]);
                simds[base + i] = lo;
                simds[base + i + 2] = hi;
            }
        }

        // distance = 1
        for base in (0..16).step_by(2) {
            let (lo, hi) = interleave_u16x16(simds[base], simds[base + 1]);
            simds[base] = lo;
            simds[base + 1] = hi;
        }

        let simds = std::mem::transmute::<[__m256i; 16], [Simd<u16, 16>; 16]>(simds);

        simds
    }
}

#[inline]
unsafe fn interleave_u8x16(a: __m128i, b: __m128i) -> (__m128i, __m128i) {
    unsafe {
        let low = _mm_unpacklo_epi8(a, b); // Interleave low 8 bytes
        let high = _mm_unpackhi_epi8(a, b); // Interleave high 8 bytes
        (low, high)
    }
}

#[inline]
unsafe fn interleave_u16x16(a: __m256i, b: __m256i) -> (__m256i, __m256i) {
    unsafe {
        // Use vpunpcklwd and vpunpckhwd for 16-bit interleaving
        let lo = _mm256_unpacklo_epi16(a, b);
        let hi = _mm256_unpackhi_epi16(a, b);

        // Fix the lane crossing issue in AVX2
        // TODO: incorrect for u16?
        let lo_fixed = _mm256_permute4x64_epi64(lo, 0b11011000); // 0xD8
        let hi_fixed = _mm256_permute4x64_epi64(hi, 0b11011000);

        (lo_fixed, hi_fixed)
    }
}
