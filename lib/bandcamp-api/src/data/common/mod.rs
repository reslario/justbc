#[cfg(feature = "query")]
pub(crate) mod parse;

pub mod date;

pub use date::{Date, Month};

use std::{fmt, marker::PhantomData};

#[cfg(feature = "query")]
use serde::Deserialize;

pub struct Id<T> {
    value: u64,
    _marker: PhantomData<T>,
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

#[cfg(feature = "query")]
impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <_>::deserialize(deserializer).map(|value| Id {
            value,
            _marker: <_>::default(),
        })
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
