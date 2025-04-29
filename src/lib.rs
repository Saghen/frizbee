#![feature(portable_simd)]
#![feature(avx512_target_feature)]
#![feature(get_mut_unchecked)]

use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod r#const;
pub mod incremental;
pub mod one_shot;
pub mod prefilter;
pub mod smith_waterman;

pub use incremental::IncrementalMatcher;
pub use one_shot::{match_indices, match_list, match_list_parallel};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Match {
    pub score: u16,
    /** Index of the match in the original list of haystacks */
    pub index_in_haystack: u32,
    pub exact: bool,
}

impl PartialOrd for Match {
    fn partial_cmp(&self, other: &Match) -> Option<Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}
impl Ord for Match {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .reverse()
            .then_with(|| self.index_in_haystack.cmp(&other.index_in_haystack))
    }
}
impl PartialEq for Match {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.index_in_haystack == other.index_in_haystack
    }
}
impl Eq for Match {}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MatchIndices {
    pub score: u16,
    pub indices: Vec<usize>,
    pub exact: bool,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    pub sort: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            prefilter: true,
            min_score: 0,
            max_typos: Some(0),
            sort: true,
        }
    }
}
