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
    max_typos: u16,

    has_sse2: bool,
    has_avx2: bool,
    has_neon: bool,
}

impl<'a, const W: usize> Prefilter<'a, W> {
    pub fn new(needle: &'a [u8], needle_cased: &'a [(u8, u8)], max_typos: u16) -> Self {
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
            max_typos,

            has_sse2,
            has_avx2,
            has_neon,
        }
    }

    pub fn case_needle(needle: &str) -> Vec<(u8, u8)> {
        needle
            .as_bytes()
            .iter()
            .map(|&c| (c.to_ascii_uppercase(), c.to_ascii_lowercase()))
            .collect()
    }

    pub fn match_haystack(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<true, true, false>(haystack)
    }

    pub fn match_haystack_insensitive(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<true, false, false>(haystack)
    }

    pub fn match_haystack_unordered(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<false, true, false>(haystack)
    }

    pub fn match_haystack_unordered_insensitive(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<false, false, false>(haystack)
    }

    pub fn match_haystack_unordered_typos(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<false, true, true>(haystack)
    }

    pub fn match_haystack_unordered_typos_insensitive(&self, haystack: &[u8]) -> bool {
        self.match_haystack_runtime_detection::<false, false, true>(haystack)
    }

    #[inline(always)]
    fn match_haystack_runtime_detection<
        const ORDERED: bool,
        const CASE_SENSITIVE: bool,
        const TYPOS: bool,
    >(
        &self,
        haystack: &[u8],
    ) -> bool {
        if W <= 16 {
            match haystack.len() {
                0 => return true,
                1..=7 => {
                    return self.match_haystack_scalar::<ORDERED, CASE_SENSITIVE, TYPOS>(haystack);
                }
                _ => {}
            }
        }

        match (self.has_avx2, self.has_sse2, self.has_neon) {
            #[cfg(target_arch = "x86_64")]
            (true, _, _) => unsafe {
                self.match_haystack_avx2::<ORDERED, CASE_SENSITIVE, TYPOS>(haystack)
            },
            #[cfg(target_arch = "x86_64")]
            (_, true, _) => unsafe {
                self.match_haystack_sse2::<ORDERED, CASE_SENSITIVE, TYPOS>(haystack)
            },
            #[cfg(target_arch = "aarch64")]
            (_, _, true) => self.match_haystack_neon::<ORDERED, CASE_SENSITIVE>(haystack),
            _ => self.match_haystack_simd::<ORDERED, CASE_SENSITIVE, TYPOS>(haystack),
        }
    }

    #[inline(always)]
    fn match_haystack_scalar<const ORDERED: bool, const CASE_SENSITIVE: bool, const TYPOS: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        match (TYPOS, CASE_SENSITIVE) {
            (true, true) => scalar::match_haystack_typos(self.needle, haystack, self.max_typos),
            (true, false) => scalar::match_haystack_typos_insensitive(
                self.needle_cased,
                haystack,
                self.max_typos,
            ),
            (false, true) => scalar::match_haystack(self.needle, haystack),
            (false, false) => scalar::match_haystack_insensitive(self.needle_cased, haystack),
        }
    }

    #[inline(always)]
    fn match_haystack_simd<const ORDERED: bool, const CASE_SENSITIVE: bool, const TYPOS: bool>(
        &self,
        haystack: &[u8],
    ) -> bool {
        match (ORDERED, CASE_SENSITIVE, TYPOS) {
            (true, _, true) => panic!("ordered typos implementations are not yet available"),
            (true, true, false) => simd::match_haystack::<W>(self.needle, haystack),
            (true, false, false) => {
                simd::match_haystack_insensitive::<W>(self.needle_cased, haystack)
            }

            (false, true, false) => simd::match_haystack_unordered::<W>(self.needle, haystack),
            (false, true, true) => {
                simd::match_haystack_unordered_typos::<W>(self.needle, haystack, self.max_typos)
            }
            (false, false, false) => {
                simd::match_haystack_unordered_insensitive::<W>(self.needle_cased, haystack)
            }
            (false, false, true) => simd::match_haystack_unordered_typos_insensitive::<W>(
                self.needle_cased,
                haystack,
                self.max_typos,
            ),
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn match_haystack_x86_64<
        const ORDERED: bool,
        const CASE_SENSITIVE: bool,
        const TYPOS: bool,
    >(
        &self,
        haystack: &[u8],
    ) -> bool {
        unsafe {
            match (ORDERED, CASE_SENSITIVE, TYPOS) {
                (true, _, true) => panic!("ordered typos implementations are not yet available"),
                (true, true, false) => x86_64::match_haystack::<W>(self.needle, haystack),
                (true, false, false) => {
                    x86_64::match_haystack_insensitive::<W>(self.needle_cased, haystack)
                }

                (false, true, false) => {
                    x86_64::match_haystack_unordered::<W>(self.needle, haystack)
                }
                (false, true, true) => x86_64::match_haystack_unordered_typos::<W>(
                    self.needle,
                    haystack,
                    self.max_typos,
                ),
                (false, false, false) => {
                    x86_64::match_haystack_unordered_insensitive::<W>(self.needle_cased, haystack)
                }
                (false, false, true) => x86_64::match_haystack_unordered_typos_insensitive::<W>(
                    self.needle_cased,
                    haystack,
                    self.max_typos,
                ),
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
    unsafe fn match_haystack_avx2<
        const ORDERED: bool,
        const CASE_SENSITIVE: bool,
        const TYPOS: bool,
    >(
        &self,
        haystack: &[u8],
    ) -> bool {
        unsafe { self.match_haystack_x86_64::<ORDERED, CASE_SENSITIVE, TYPOS>(haystack) }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn match_haystack_sse2<
        const ORDERED: bool,
        const CASE_SENSITIVE: bool,
        const TYPOS: bool,
    >(
        &self,
        haystack: &[u8],
    ) -> bool {
        unsafe { self.match_haystack_x86_64::<ORDERED, CASE_SENSITIVE, TYPOS>(haystack) }
    }
}

#[cfg(test)]
mod tests {
    use super::Prefilter;

    /// Ensures both the ordered and unordered implementations return the same result
    fn match_haystack<const W: usize>(needle: &str, haystack: &str) -> bool {
        let ordered = match_haystack_generic::<W, true, true>(needle, haystack, 0);
        let unordered = match_haystack_generic::<W, false, true>(needle, haystack, 0);
        assert_eq!(
            ordered, unordered,
            "ordered and unordered implementations produced different results for {needle} on {haystack}"
        );
        ordered
    }

    fn match_haystack_insensitive<const W: usize>(needle: &str, haystack: &str) -> bool {
        let ordered = match_haystack_generic::<W, true, false>(needle, haystack, 0);
        let unordered = match_haystack_generic::<W, false, false>(needle, haystack, 0);
        assert_eq!(
            ordered, unordered,
            "ordered and unordered implementations produced different results for {needle} on {haystack}"
        );
        ordered
    }

    fn match_haystack_ordered<const W: usize>(needle: &str, haystack: &str) -> bool {
        match_haystack_generic::<W, true, true>(needle, haystack, 0)
    }

    fn match_haystack_unordered<const W: usize>(needle: &str, haystack: &str) -> bool {
        match_haystack_generic::<W, false, true>(needle, haystack, 0)
    }

    fn match_haystack_unordered_typos<const W: usize>(
        needle: &str,
        haystack: &str,
        max_typos: u16,
    ) -> bool {
        match_haystack_generic::<W, false, true>(needle, haystack, max_typos)
    }

    fn match_haystack_unordered_typos_insensitive<const W: usize>(
        needle: &str,
        haystack: &str,
        max_typos: u16,
    ) -> bool {
        match_haystack_generic::<W, false, false>(needle, haystack, max_typos)
    }

    #[test]
    fn test_exact_match() {
        assert!(match_haystack::<16>("foo", "foo"));
        assert!(match_haystack::<16>("a", "a"));
        assert!(match_haystack::<16>("hello", "hello"));
    }

    #[test]
    fn test_fuzzy_match_with_gaps() {
        assert!(match_haystack::<16>("foo", "f_o_o"));
        assert!(match_haystack::<16>("foo", "f__o__o"));
        assert!(match_haystack::<16>("abc", "a_b_c"));
        assert!(match_haystack::<16>("test", "t_e_s_t"));
    }

    #[test]
    fn test_unordered_within_chunk() {
        assert!(match_haystack_unordered::<16>("foo", "oof"));
        assert!(!match_haystack_ordered::<16>("foo", "oof"));

        assert!(match_haystack_unordered::<16>("abc", "cba"));
        assert!(!match_haystack_ordered::<16>("abc", "cba"));

        assert!(match_haystack_unordered::<16>("test", "tset"));
        assert!(!match_haystack_ordered::<16>("test", "tset"));

        assert!(match_haystack_unordered::<16>("hello", "olleh"));
        assert!(!match_haystack_ordered::<16>("hello", "olleh"));
    }

    #[test]
    fn test_case_sensitivity() {
        assert!(!match_haystack::<16>("foo", "FOO"));
        assert!(match_haystack_insensitive::<16>("foo", "FOO"));

        assert!(!match_haystack::<16>("Foo", "foo"));
        assert!(match_haystack_insensitive::<16>("Foo", "foo"));

        assert!(!match_haystack::<16>("ABC", "abc"));
        assert!(match_haystack_insensitive::<16>("ABC", "abc"));
    }

    #[test]
    fn test_chunk_boundary() {
        // Characters must be within same 16-byte chunk
        let haystack = "oo______________f"; // 'f' is at position 16 (17th byte)
        assert!(!match_haystack::<16>("foo", haystack));

        // But if all within one chunk, should work
        let haystack = "oof_____________"; // All within first 16 bytes
        assert!(match_haystack_unordered::<16>("foo", haystack));
    }

    #[ignore = "overlapping loads are not yet masked on the ordered implementation"]
    #[test]
    fn test_overlapping_load() {
        // Because we load the last 16 bytes of the haystack in the final iteration,
        // when the haystack.len() % 16 != 0, we end up matching on the 'o' twice
        assert!(!match_haystack_ordered::<48>(
            "foo",
            "f_________________________o______"
        ));
    }

    #[test]
    fn test_multiple_chunks() {
        assert!(match_haystack::<48>(
            "foo",
            "f_______________o_______________o"
        ));
        assert!(match_haystack::<48>(
            "abc",
            "a_______________b_______________c_______________"
        ));
    }

    #[test]
    fn test_partial_matches() {
        assert!(!match_haystack::<16>("fob", "fo"));
        assert!(!match_haystack::<16>("test", "tet"));
        assert!(!match_haystack::<16>("abc", "a"));
    }

    #[test]
    fn test_duplicate_characters_in_needle() {
        assert!(match_haystack::<16>("foo", "foo"));
        assert!(match_haystack_unordered::<16>("foo", "ofo"));
        assert!(match_haystack_unordered::<16>("foo", "fo")); // Missing one 'o'

        assert!(match_haystack_unordered::<16>("aaa", "aaa"));
        assert!(match_haystack_unordered::<16>("aaa", "aa"));
    }

    #[test]
    fn test_haystack_with_extra_characters() {
        assert!(match_haystack::<16>("foo", "foobar"));
        assert!(match_haystack::<16>("foo", "prefoobar"));
        assert!(match_haystack::<16>("abc", "xaxbxcx"));
    }

    #[test]
    fn test_edge_cases_at_16_byte_boundary() {
        let haystack = "123456789012345f"; // 'f' at position 15 (last position in chunk)
        assert!(match_haystack::<16>("f", haystack));

        let haystack = "o_______________of"; // Two 'o's in first chunk, 'f' in second
        assert!(!match_haystack::<16>("foo", haystack));
    }

    #[test]
    #[should_panic]
    fn test_invalid_width_limit_panic() {
        // When W > 16, the function should panic when haystack.len() < 16
        assert!(match_haystack::<32>("abc", "cba"));
        assert!(match_haystack::<64>("abc", "cba"));
    }

    #[test]
    fn test_width_limit() {
        let haystack = "a_______________b_______________c";
        assert!(!match_haystack::<16>("abc", haystack));
        assert!(match_haystack::<48>("abc", haystack));
    }

    #[test]
    fn test_overlapping_chunks() {
        // The function uses overlapping loads, so test edge cases
        // where characters might be found in overlapping regions
        let haystack = "_______________fo"; // 'f' at position 15, 'o' at position 16
        assert!(match_haystack::<32>("fo", haystack));
    }

    #[test]
    fn test_single_character_needle() {
        // Single character needles
        assert!(match_haystack::<16>("a", "a"));
        assert!(match_haystack::<16>("a", "ba"));
        assert!(match_haystack::<16>("a", "_______________a"));
        assert!(!match_haystack::<16>("a", ""));
    }

    #[test]
    fn test_repeated_character_haystack() {
        // Haystack with repeated characters
        assert!(match_haystack::<16>("abc", "aaabbbccc"));
        assert!(match_haystack::<16>("foo", "fofofoooo"));
    }

    #[test]
    fn test_typos_single_missing_character() {
        // One character missing from haystack
        assert!(match_haystack_unordered_typos::<16>("bar", "ba", 1));
        assert!(match_haystack_unordered_typos::<16>("bar", "ar", 1));
        assert!(match_haystack_unordered_typos::<16>("hello", "hllo", 1));
        assert!(match_haystack_unordered_typos::<16>("test", "tst", 1));

        // Should fail with 0 typos allowed
        assert!(!match_haystack_unordered_typos::<16>("bar", "ba", 0));
        assert!(!match_haystack_unordered_typos::<16>("hello", "hllo", 0));
    }

    #[test]
    fn test_typos_multiple_missing_characters() {
        assert!(match_haystack_unordered_typos::<16>("hello", "hll", 2));
        assert!(match_haystack_unordered_typos::<16>("testing", "tstng", 2));
        assert!(match_haystack_unordered_typos::<16>("abcdef", "abdf", 2));

        assert!(!match_haystack_unordered_typos::<16>("hello", "hll", 1));
        assert!(!match_haystack_unordered_typos::<16>("testing", "tstng", 1));
    }

    #[test]
    fn test_typos_with_gaps() {
        assert!(match_haystack_unordered_typos::<16>("bar", "b_r", 1));
        assert!(match_haystack_unordered_typos::<16>("test", "t__s_t", 1));
        assert!(match_haystack_unordered_typos::<16>("helo", "h_l_", 2));
    }

    #[test]
    fn test_typos_unordered_permutations() {
        assert!(match_haystack_unordered_typos::<16>("bar", "rb", 1));
        assert!(match_haystack_unordered_typos::<16>("abcdef", "fcda", 2));
    }

    #[test]
    fn test_typos_case_insensitive() {
        // Case insensitive with typos
        assert!(match_haystack_unordered_typos_insensitive::<16>(
            "BAR", "ba", 1
        ));
        assert!(match_haystack_unordered_typos_insensitive::<16>(
            "Hello", "HLL", 2
        ));
        assert!(match_haystack_unordered_typos_insensitive::<16>(
            "TeSt", "ES", 2
        ));
        assert!(!match_haystack_unordered_typos_insensitive::<16>(
            "TeSt", "ES", 1
        ));
    }

    #[test]
    fn test_typos_edge_cases() {
        // All characters missing (typos == needle length)
        assert!(match_haystack_unordered_typos::<16>("abc", "", 3));

        // More typos allowed than necessary
        assert!(match_haystack_unordered_typos::<16>("foo", "fo", 5));
    }

    #[test]
    fn test_typos_across_chunks() {
        assert!(match_haystack_unordered_typos::<48>(
            "abc",
            "a_______________b",
            1
        ));

        assert!(match_haystack_unordered_typos::<48>(
            "test",
            "t_______________s_______________t",
            1
        ));
    }

    #[test]
    fn test_typos_single_character_needle() {
        assert!(match_haystack_unordered_typos::<16>("a", "a", 0));
        assert!(match_haystack_unordered_typos::<16>("a", "", 1));
        assert!(!match_haystack_unordered_typos::<16>("a", "", 0));
    }

    fn normalize_haystack(haystack: &str) -> String {
        if haystack.len() < 8 {
            "_".repeat(8 - haystack.len()) + haystack
        } else {
            haystack.to_string()
        }
    }

    fn match_haystack_generic<const W: usize, const ORDERED: bool, const CASE_SENSITIVE: bool>(
        needle: &str,
        haystack: &str,
        max_typos: u16,
    ) -> bool {
        let needle_cased = Prefilter::<W>::case_needle(needle);
        let needle = needle.as_bytes();
        let haystack = normalize_haystack(haystack);
        let haystack = haystack.as_bytes();

        let prefilter = Prefilter::<W>::new(needle, &needle_cased, max_typos);

        if max_typos > 0 {
            let typo_result =
                prefilter.match_haystack_simd::<ORDERED, CASE_SENSITIVE, true>(haystack);

            #[cfg(target_arch = "x86_64")]
            {
                let typo_result_x86_64 = unsafe {
                    prefilter.match_haystack_x86_64::<ORDERED, CASE_SENSITIVE, true>(haystack)
                };
                assert_eq!(
                    typo_result, typo_result_x86_64,
                    "x86_64 typos (set to 0) and simd implementations produced different results"
                );
            }

            return typo_result;
        }

        let result = prefilter.match_haystack_simd::<ORDERED, CASE_SENSITIVE, false>(haystack);

        if !ORDERED {
            let typo_result =
                prefilter.match_haystack_simd::<ORDERED, CASE_SENSITIVE, true>(haystack);
            assert_eq!(
                result, typo_result,
                "regular and typos implementations (set to 0) produced different results"
            );
        }

        #[cfg(target_arch = "x86_64")]
        {
            let result_x86_64 = unsafe {
                prefilter.match_haystack_x86_64::<ORDERED, CASE_SENSITIVE, false>(haystack)
            };
            assert_eq!(
                result, result_x86_64,
                "x86_64 and simd implementations produced different results"
            );

            if !ORDERED {
                let typo_result_x86_64 = unsafe {
                    prefilter.match_haystack_x86_64::<ORDERED, CASE_SENSITIVE, true>(haystack)
                };
                assert_eq!(
                    result, typo_result_x86_64,
                    "x86_64 typos (set to 0) and simd implementations produced different results"
                );
            }
        }

        result
    }
}
