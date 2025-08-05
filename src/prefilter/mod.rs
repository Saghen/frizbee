//! Fast prefiltering algorithms, which run before Smith Waterman since in the typical case,
//! a small percentage of the haystack will match the needle. Automatically used by the Matcher
//! and match_list APIs.
//!
//! Unordered algorithms are much faster than ordered algorithms, but don't guarantee that the
//! needle is contained in the haystack, unlike ordered algorithms. As a result, a backwards
//! pass must be performed after Smith Waterman to verify the number of typos. But the faster
//! prefilter generally seems to outweigh this extra cost.
//!
//! The `Prefilter` struct chooses the fastest algorithm via runtime feature detection.
//!
//! All algorithms, except scalar, assume that needle.len() > 0 && haystack.len() >= 8

pub mod bitmask;
pub mod scalar;
pub mod simd;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[derive(Clone, Debug)]
pub struct Prefilter<'a, const W: usize> {
    needle: &'a [u8],
    needle_cased: &'a [(u8, u8)],
    has_sse2: bool,
    has_avx2: bool,
    has_neon: bool,
}

impl<'a, const W: usize> Prefilter<'a, W> {
    pub fn new(needle: &'a [u8], needle_cased: &'a [(u8, u8)]) -> Self {
        #[cfg(target_arch = "x86_64")]
        let has_sse2 = is_x86_feature_detected!("sse2");
        #[cfg(not(target_arch = "x86_64"))]
        let has_sse2 = false;

        #[cfg(target_arch = "x86_64")]
        let has_avx2 =
            has_sse2 && is_x86_feature_detected!("avx2") && is_x86_feature_detected!("avx");
        #[cfg(not(target_arch = "x86_64"))]
        let has_avx2 = false;

        #[cfg(target_arch = "aarch64")]
        let has_neon = true;
        #[cfg(not(target_arch = "aarch64"))]
        let has_neon = false;

        Prefilter {
            needle,
            needle_cased,
            has_sse2,
            has_avx2,
            has_neon,
        }
    }

    pub fn case_needle(needle: &str) -> Vec<(u8, u8)> {
        needle
            .as_bytes()
            .iter()
            .map(|&c| (c.to_ascii_uppercase(), c.to_ascii_uppercase()))
            .collect()
    }

    pub fn match_haystack(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<true, true>(haystack)
    }

    pub fn match_haystack_insensitive(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<true, false>(haystack)
    }

    pub fn match_haystack_unordered(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<false, true>(haystack)
    }

    pub fn match_haystack_unordered_insensitive(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<false, false>(haystack)
    }

    #[inline(always)]
    fn match_haystack_runtime_detection<const ORDERED: bool, const CASE_SENSITIVE: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        if W <= 16 {
            match haystack.len() {
                0 => return true,
                1..=7 => return self.match_haystack_scalar::<ORDERED, CASE_SENSITIVE>(haystack),
                _ => {}
            }
        }

        match (self.has_avx2, self.has_sse2, self.has_neon) {
            #[cfg(target_arch = "x86_64")]
            (true, _, _) => unsafe {
                self.match_haystack_avx2::<ORDERED, CASE_SENSITIVE>(haystack)
            },
            #[cfg(target_arch = "x86_64")]
            (_, true, _) => unsafe {
                self.match_haystack_sse2::<ORDERED, CASE_SENSITIVE>(haystack)
            },
            #[cfg(target_arch = "aarch64")]
            (_, _, true) => self.match_haystack_neon::<ORDERED, CASE_SENSITIVE>(haystack),
            _ => self.match_haystack_simd::<ORDERED, CASE_SENSITIVE>(haystack),
        }
    }

    #[inline(always)]
    fn match_haystack_scalar<const ORDERED: bool, const CASE_SENSITIVE: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        if CASE_SENSITIVE {
            scalar::match_haystack(self.needle, haystack)
        } else {
            scalar::match_haystack_insensitive(self.needle_cased, haystack)
        }
    }

    #[inline(always)]
    fn match_haystack_simd<const ORDERED: bool, const CASE_SENSITIVE: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        match (ORDERED, CASE_SENSITIVE) {
            (true, true) => simd::match_haystack::<W>(self.needle, haystack),
            (true, false) => simd::match_haystack_insensitive::<W>(self.needle_cased, haystack),
            (false, true) => simd::match_haystack_unordered::<W>(self.needle, haystack),
            (false, false) => {
                simd::match_haystack_unordered_insensitive::<W>(self.needle_cased, haystack)
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn match_haystack_x86_64<const ORDERED: bool, const CASE_SENSITIVE: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        match (ORDERED, CASE_SENSITIVE) {
            (true, true) => x86_64::match_haystack::<W>(self.needle, haystack),
            (true, false) => x86_64::match_haystack_insensitive::<W>(self.needle_cased, haystack),
            (false, true) => x86_64::match_haystack_unordered::<W>(self.needle, haystack),
            (false, false) => {
                x86_64::match_haystack_unordered_insensitive::<W>(self.needle_cased, haystack)
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn match_haystack_neon<const Ordered: bool, const CaseSensitive: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        self.match_haystack_simd::<Ordered, CaseSensitive>(haystack)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2,avx,avx2")]
    unsafe fn match_haystack_avx2<const ORDERED: bool, const CASE_SENSITIVE: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        self.match_haystack_x86_64::<ORDERED, CASE_SENSITIVE>(haystack)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn match_haystack_sse2<const ORDERED: bool, const CASE_SENSITIVE: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        self.match_haystack_x86_64::<ORDERED, CASE_SENSITIVE>(haystack)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::simd::prefilter_simd as _prefilter_simd;
//     #[cfg(all(
//         target_feature = "sse2",
//         target_feature = "bmi1",
//         target_arch = "x86_64"
//     ))]
//     use super::ordered::x86_64::prefilter_sse2;
//
//     fn prefilter_simd(needle: &str, haystack: &str) -> bool {
//         let needle_str = needle;
//         let haystack_str = haystack;
//
//         let needle = needle.as_bytes();
//         let haystack = haystack.as_bytes();
//
//         let lane_8 = _prefilter_simd::<8, 64>(needle, haystack);
//         let lane_16 = _prefilter_simd::<16, 128>(needle, haystack);
//         let lane_32 = _prefilter_simd::<32, 256>(needle, haystack);
//         let lane_64 = _prefilter_simd::<64, 512>(needle, haystack);
//         #[cfg(all(
//             target_feature = "sse2",
//             target_feature = "bmi1",
//             target_arch = "x86_64"
//         ))]
//         unsafe {
//             let sse2 = prefilter_sse2::<64>(needle, haystack);
//             assert_eq!(
//                 lane_8, sse2,
//                 "({needle_str}, {haystack_str}) 8 lane implementation produced different results than SSE2"
//             );
//         };
//
//         assert_eq!(
//             lane_8, lane_16,
//             "({needle_str}, {haystack_str}) 8 lane implementation produced different results than 16 lane"
//         );
//         assert_eq!(
//             lane_8, lane_32,
//             "8 lane implementation produced different results than 32 lane"
//         );
//         assert_eq!(
//             lane_8, lane_64,
//             "8 lane implementation produced different results than 64 lane"
//         );
//
//         lane_8
//     }
//
//     #[ignore = "We assume that the bucket has already checked this case"]
//     #[test]
//     fn test_empty_needle() {
//         assert!(prefilter_simd("", "haystack"));
//         assert!(prefilter_simd("", ""));
//     }
//
//     #[ignore = "We assume that the bucket has already checked this case"]
//     #[test]
//     fn test_empty_haystack() {
//         assert!(!prefilter_simd("needle", ""));
//         assert!(prefilter_simd("", ""));
//     }
//
//     #[test]
//     fn test_exact_match() {
//         assert!(prefilter_simd("hello", "hello"));
//         assert!(prefilter_simd("h", "h"));
//         assert!(prefilter_simd(
//             "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
//             "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
//         ));
//     }
//
//     #[test]
//     fn test_case_insensitive() {
//         assert!(prefilter_simd("hello", "HELLO"));
//         assert!(prefilter_simd("HELLO", "hello"));
//         assert!(prefilter_simd("HeLLo", "hEllO"));
//         assert!(prefilter_simd("abc", "ABC"));
//         assert!(prefilter_simd("ABC", "abc"));
//     }
//
//     #[test]
//     fn test_fuzzy_match() {
//         assert!(prefilter_simd("ab", "a_b"));
//         assert!(prefilter_simd("abc", "_a___ab__ac_"));
//         assert!(!prefilter_simd("abc", "_____ba__ac_"));
//     }
//
//     #[test]
//     fn test_needle_in_middle() {
//         assert!(prefilter_simd("world", "hello world!"));
//         assert!(prefilter_simd("orl", "hello world!"));
//     }
//
//     #[test]
//     fn test_needle_at_start() {
//         assert!(prefilter_simd("h", "hello world"));
//         // assert!(prefilter_simd("hello", "hello world"));
//     }
//
//     #[test]
//     fn test_needle_at_end() {
//         assert!(prefilter_simd("d", "hello world"));
//         assert!(prefilter_simd("world", "hello world"));
//     }
//
//     #[test]
//     fn test_needle_not_found() {
//         assert!(!prefilter_simd("xyz", "hello world"));
//         assert!(!prefilter_simd("XYZ", "hello world"));
//         assert!(!prefilter_simd("abc", "def ghi jkl"));
//     }
//
//     #[test]
//     fn test_needle_longer_than_haystack() {
//         assert!(!prefilter_simd("hello world", "hello"));
//         assert!(!prefilter_simd("abcdef", "abc"));
//         assert!(!prefilter_simd("a", ""));
//     }
// }
