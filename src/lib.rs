#![feature(portable_simd)]

pub mod r#const;
pub mod incremental;
pub mod one_shot;
pub mod prefilter;
pub mod smith_waterman;

#[derive(Debug, Clone, Default)]
pub struct Match {
    /** Index of the match in the original list of haystacks */
    pub index_in_haystack: usize,
    pub indices: Option<Vec<usize>>,
    pub score: u16,
    pub exact: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// May perform prefiltering, depending on haystack length and max number of typos,
    /// which drastically improves performance when most of the haystack does not match
    pub prefilter: bool,
    /// Minimum score of an item to return a result. Generally, needle.len() * 6 will be a good
    /// default
    pub min_score: u16,
    /// The maximum number of characters missing from the needle, before an item in the
    /// haystack is filtered out
    pub max_typos: Option<u16>,
    /// Sort the results while maintaining the original order of the haystacks
    pub stable_sort: bool,
    /// Sort the results without maintaining the original order of the haystacks (much faster on
    /// long lists)
    pub unstable_sort: bool,
    /// Calculate and include an array of matched indices for each haystack
    pub matched_indices: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            prefilter: true,
            min_score: 0,
            max_typos: None,
            stable_sort: true,
            unstable_sort: false,
            matched_indices: false,
        }
    }
}
