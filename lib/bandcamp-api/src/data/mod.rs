pub mod common;
pub mod albums;

pub trait Query: Sized {
    type Page;
    type Err;

    fn query(page: &Self::Page) -> Result<Self, Self::Err>;
}
