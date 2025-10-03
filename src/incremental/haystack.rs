use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct IncrementalHaystack<const W: usize> {
    pub data: [u8; W],
    pub score_matrix: RefCell<Vec<[u16; W]>>,
    pub index: u32,
    pub score: RefCell<u16>,
    /// Index of the needle character that caused the item to be filtered out
    pub filtered_at: u32,
}

impl<const W: usize> IncrementalHaystack<W> {
    pub fn new(index: u32, haystack: &str) -> Self {
        let mut data = [0u8; W];
        data[0..haystack.len()].copy_from_slice(haystack.as_bytes());

        Self {
            data,
            score_matrix: RefCell::new(vec![]),
            index,
            score: RefCell::new(0),
            filtered_at: u32::MAX,
        }
    }

    pub fn truncate_to(&self, idx: u32) {
        self.score_matrix.borrow_mut().truncate(idx as usize);
    }

    pub fn push_scores(&self, scores: [u16; W]) {
        self.score_matrix.borrow_mut().push(scores);
        self.score
            .replace_with(|score| (*score).max(scores.into_iter().max().unwrap()));
    }

    pub fn typos(&self, max_typos: u16) -> u16 {
        let score_matrix = self.score_matrix.borrow();
        if score_matrix.is_empty() {
            return 0;
        }

        let last_col = score_matrix.last().unwrap();
        let mut col_idx = score_matrix.len() - 1;
        let mut row_idx = last_col
            .iter()
            .enumerate()
            .max_by_key(|(_, score)| *score)
            .map(|(index, _)| index)
            .unwrap();

        let mut typos = 0;

        while col_idx > 0 {
            if typos >= max_typos {
                break;
            }

            // Must be moving left
            if row_idx == 0 {
                typos += 1;
                col_idx -= 1;
                continue;
            }

            // Gather up the scores for all possible paths
            let diag = score_matrix[col_idx - 1][row_idx - 1];
            let left = score_matrix[col_idx - 1][row_idx];
            let up = score_matrix[col_idx][row_idx - 1];

            // Match or mismatch
            if diag >= left && diag >= up {
                // Must be a mismatch
                if diag >= last_col[row_idx] {
                    typos += 1;
                }
                row_idx -= 1;
                col_idx -= 1;
            // Skipped character in needle
            } else if left >= up {
                typos += 1;
                col_idx -= 1;
            // Skipped character in haystack
            } else {
                row_idx -= 1;
            }
        }

        // HACK: Compensate for the last column being a typo
        if col_idx == 0 && last_col[row_idx] == 0 {
            typos += 1;
        }

        typos
    }
}
