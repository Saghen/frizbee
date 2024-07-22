const MATCH_SCORE: i16 = 2;
const MISMATCH_SCORE: i16 = -1;
const GAP_OPEN_PENALTY: i16 = -2;
const GAP_EXTEND_PENALTY: i16 = -1;

pub fn smith_waterman_reference(query: &[u8], target: &[u8]) -> (i16, Vec<Vec<i16>>) {
    let query_len = query.len();
    let target_len = target.len();
    let mut m = vec![vec![0i16; target_len + 1]; query_len + 1];
    let mut i = vec![vec![0i16; target_len + 1]; query_len + 1];
    let mut d = vec![vec![0i16; target_len + 1]; query_len + 1];

    let mut max_score = 0;

    for qi in 1..=query_len {
        for ti in 1..=target_len {
            // Match/Mismatch
            let match_score = if query[qi - 1] == target[ti - 1] {
                MATCH_SCORE
            } else {
                MISMATCH_SCORE
            };
            m[qi][ti] = (m[qi - 1][ti - 1]
                .max(i[qi - 1][ti - 1])
                .max(d[qi - 1][ti - 1])
                + match_score)
                .max(0);

            // Insertion (gap in target)
            i[qi][ti] = (m[qi - 1][ti] + GAP_OPEN_PENALTY)
                .max(i[qi - 1][ti] + GAP_EXTEND_PENALTY)
                .max(d[qi - 1][ti] + GAP_OPEN_PENALTY)
                .max(0);

            // Deletion (gap in query)
            d[qi][ti] = (m[qi][ti - 1] + GAP_OPEN_PENALTY)
                .max(d[qi][ti - 1] + GAP_EXTEND_PENALTY)
                .max(i[qi][ti - 1] + GAP_OPEN_PENALTY)
                .max(0);

            // Update max score
            max_score = max_score.max(m[qi][ti]).max(i[qi][ti]).max(d[qi][ti]);
        }
    }

    let mut score_matrix = vec![vec![0i16; target_len + 1]; query_len + 1];
    for x in 0..=query_len {
        for y in 0..=target_len {
            score_matrix[x][y] = m[x][y].max(i[x][y]).max(d[x][y]);
        }
    }

    (max_score, score_matrix)
}
