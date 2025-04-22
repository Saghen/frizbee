pub mod bucket;
pub mod matcher;
pub mod matcher_parallel;
mod vec;

pub use matcher::match_list;
pub use matcher_parallel::match_list_parallel;
