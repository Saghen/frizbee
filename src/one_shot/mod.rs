pub mod bucket;
pub mod matcher;
pub mod parallel;

pub use matcher::match_list;
pub use parallel::match_list_parallel;

pub(crate) trait Appendable<T> {
    fn append(&mut self, value: T);
}

impl<T> Appendable<T> for Vec<T> {
    fn append(&mut self, value: T) {
        self.push(value);
    }
}
