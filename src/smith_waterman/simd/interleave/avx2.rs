use std::{
    arch::x86_64::*,
    simd::{Simd, num::SimdUint},
};

#[target_feature(enable = "avx2")]
pub fn interleave<const W: usize>(strs: [&str; 16]) -> [Simd<u16, 16>; W] {
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

#[inline]
fn to_simd(str_bytes: [&[u8]; 16], str_lens: [usize; 16], offset: usize) -> [__m128i; 16] {
    unsafe {
        std::array::from_fn(|i| {
            let len = str_lens[i];
            // beyond length
            // NOTE: we could optimize this away if (max_length - min_length) < 16
            if offset >= len {
                return _mm_setzero_si128();
            }

            let remaining = len - offset;
            let load_len = remaining.min(16);

            if load_len == 16 {
                _mm_loadu_si128(str_bytes[i][offset..].as_ptr() as *const __m128i)
            } else {
                let mut data = _mm_setzero_si128();
                std::ptr::copy_nonoverlapping(
                    str_bytes[i][offset..].as_ptr(),
                    &mut data as *mut __m128i as *mut u8,
                    load_len,
                );
                data
            }
        })
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
unsafe fn interleave_u8x16(a: __m128i, b: __m128i) -> (__m128i, __m128i) {
    unsafe {
        let low = _mm_unpacklo_epi8(a, b); // Interleave low 8 bytes
        let high = _mm_unpackhi_epi8(a, b); // Interleave high 8 bytes
        (low, high)
    }
}

#[cfg(test)]
mod tests {
    use super::interleave;

    #[test]
    fn test_interleave_avx2() {
        // TODO: what the fuck
        let strings_owned: [String; 16] =
            std::array::from_fn(|i| -> [u8; 32] { std::array::from_fn(|j| (i * 16 + j) as u8) })
                .map(|str| unsafe { String::from_utf8_unchecked(str.to_vec()) });
        let strings = strings_owned.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        let strings: &[&str; 16] = strings.as_slice().try_into().unwrap();

        let transposed = unsafe { interleave::<32>(*strings) };

        let expected: [[u16; 16]; 32] =
            std::array::from_fn(|i| std::array::from_fn(|j| ((j * 16 + i) % 256) as u16));

        assert_eq!(transposed.map(|simd| simd.to_array()), expected);
    }
}
