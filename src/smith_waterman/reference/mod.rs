mod algorithm;
mod indices;
mod typos;

pub use algorithm::smith_waterman;
pub use indices::char_indices_from_score_matrix;
pub use typos::typos_from_score_matrix;
