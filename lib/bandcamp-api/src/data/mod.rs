pub mod common;
pub mod albums;
pub mod search;

pub trait Query<P>: Sized {
    type Err;

    fn query(page: P) -> Result<Self, Self::Err>;
}
