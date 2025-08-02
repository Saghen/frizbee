use std::{arch::x86_64::*, simd::Simd};

#[inline(never)]
pub fn interleave_simd_avx2<const W: usize>(strs: [&str; 16]) -> [Simd<u16, 16>; W] {
    let str_bytes: [&[u8]; 16] = std::array::from_fn(|i| strs[i].as_bytes());
    let str_lens: [usize; 16] = std::array::from_fn(|i| str_bytes[i].len());

    let mut interleaved = [Simd::splat(0); W];
    for offset in (0..W).step_by(16) {
        let simds = to_simd(str_bytes, str_lens, offset);
        let interleaved_chunk = interleave_chunk(simds);

        let copy_len = (W - offset).min(16);
        interleaved[offset..offset + copy_len].copy_from_slice(&interleaved_chunk[0..copy_len]);
    }

    interleaved
}

#[inline(always)]
fn to_simd(str_bytes: [&[u8]; 16], str_lens: [usize; 16], offset: usize) -> [__m256i; 16] {
    unsafe {
        std::array::from_fn(|i| {
            let len = str_lens[i];

            if offset >= len {
                // Beyond string length - return zeros
                return _mm256_setzero_si256();
            }

            let remaining = len - offset;
            let load_len = remaining.min(16);

            if load_len == 16 {
                // Full load - most common case
                // u8x16 = 128 bit
                let bytes = _mm_loadu_si128(str_bytes[i][offset..].as_ptr() as *const __m128i);
                // u8x16 -> u16x16
                _mm256_cvtepu8_epi16(bytes)
            } else {
                // Partial load - use masked load if available
                let mut temp = _mm_setzero_si128();
                std::ptr::copy_nonoverlapping(
                    str_bytes[i][offset..].as_ptr(),
                    &mut temp as *mut __m128i as *mut u8,
                    load_len,
                );
                _mm256_cvtepu8_epi16(temp)
            }
        })
    }
}

#[inline(always)]
pub fn interleave_chunk(mut simds: [__m256i; 16]) -> [Simd<u16, 16>; 16] {
    unsafe {
        // Stage 1: distance = 8
        for i in 0..8 {
            let (lo, hi) = interleave_u16x16(simds[i], simds[i + 8]);
            simds[i] = lo;
            simds[i + 8] = hi;
        }

        // Stage 2: distance = 4
        for base in (0..16).step_by(8) {
            for i in 0..4 {
                let (lo, hi) = interleave_u16x16(simds[base + i], simds[base + i + 4]);
                simds[base + i] = lo;
                simds[base + i + 4] = hi;
            }
        }

        // Stage 3: distance = 2
        for base in (0..16).step_by(4) {
            for i in 0..2 {
                let (lo, hi) = interleave_u16x16(simds[base + i], simds[base + i + 2]);
                simds[base + i] = lo;
                simds[base + i + 2] = hi;
            }
        }

        // Stage 4: distance = 1
        for base in (0..16).step_by(2) {
            let (lo, hi) = interleave_u16x16(simds[base], simds[base + 1]);
            simds[base] = lo;
            simds[base + 1] = hi;
        }

        std::mem::transmute::<[__m256i; 16], [Simd<u16, 16>; 16]>(simds)
    }
}

unsafe fn interleave_u16x16(a: __m256i, b: __m256i) -> (__m256i, __m256i) {
    // Use vpunpcklwd and vpunpckhwd for 16-bit interleaving
    let lo = _mm256_unpacklo_epi16(a, b);
    let hi = _mm256_unpackhi_epi16(a, b);

    // Fix the lane crossing issue in AVX2
    let lo_fixed = _mm256_permute4x64_epi64(lo, 0b11011000); // 0xD8
    let hi_fixed = _mm256_permute4x64_epi64(hi, 0b11011000);

    (lo_fixed, hi_fixed)
}

#[cfg(test)]
mod tests {
    use std::{
        arch::x86_64::{__m256i, _mm256_loadu_si256},
        simd::Simd,
    };

    use super::interleave_u16x16;

    #[test]
    fn test_interleave_avx2() {
        let a = unsafe { _mm256_loadu_si256([65u16; 16].as_ptr() as *const __m256i) };
        let b = unsafe { _mm256_loadu_si256([66u16; 16].as_ptr() as *const __m256i) };
        let (a, b) = unsafe { interleave_u16x16(a, b) };
        let a = unsafe { std::mem::transmute::<__m256i, Simd<u16, 16>>(a) };
        let b = unsafe { std::mem::transmute::<__m256i, Simd<u16, 16>>(b) };
        assert_eq!(
            a.to_array(),
            [65, 66, 65, 66, 65, 66, 65, 66, 65, 66, 65, 66, 65, 66, 65, 66]
        );
        assert_eq!(
            b.to_array(),
            [65, 66, 65, 66, 65, 66, 65, 66, 65, 66, 65, 66, 65, 66, 65, 66]
        );
    }
}
