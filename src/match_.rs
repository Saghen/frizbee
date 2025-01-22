#[derive(Debug, Clone, Default)]
pub struct Match {
    /** Index of the match in the original list of haystacks */
    pub index_in_haystack: usize,
    pub score: u16,
    pub indices: Option<Vec<usize>>,
}
