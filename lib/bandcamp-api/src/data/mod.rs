pub mod common;
pub mod releases;
pub mod search;

pub trait Query<P>: Sized {
    type Err;

    fn query(page: P) -> Result<Self, Self::Err>;
}
