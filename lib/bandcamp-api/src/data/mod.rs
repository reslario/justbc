pub mod common;
pub mod fans;
pub mod outlets;
pub mod releases;
pub mod search;

#[cfg(feature = "query")]
pub trait Query<A: ?Sized>: serde::de::DeserializeOwned {
    fn url(args: &A) -> url::Url;
}
