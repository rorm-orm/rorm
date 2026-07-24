pub mod db_enum;
pub mod field_type;
pub mod model;
pub mod patch;
mod utils;

pub trait SliceExt {
    type Item;

    fn map_collect<'a, U>(&'a self, f: impl FnMut(&'a Self::Item) -> U) -> Vec<U>;
}
impl<T> SliceExt for [T] {
    type Item = T;

    fn map_collect<'a, U>(&'a self, f: impl FnMut(&'a Self::Item) -> U) -> Vec<U> {
        self.iter().map(f).collect()
    }
}
