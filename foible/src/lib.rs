use std::fmt::Debug;

pub trait AsSource {
    fn next_source(&self) -> Option<&dyn Debug>;
}

pub use foible_derive::AsSource;
