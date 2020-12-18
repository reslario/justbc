pub mod common;
pub mod releases;
pub mod search;
pub mod outlets;
pub mod fans;

#[cfg(feature = "query")]
pub trait Query<A: ?Sized>: serde::de::DeserializeOwned {
    fn url(args: &A) -> url::Url;
}
