//! A fast xml and web scraping library that uses [quick-xml](https://crates.io/crates/quick-xml)
//! to provide a pull-based reading interface based on events.
//!
//! On top of that, it provides ergonomic tools to filter documents and extract
//! information from them.

mod scraper;
pub mod filter;
pub mod extract;

pub use{
    scraper::*,
    quick_xml::events::Event
};

/// The error type used by this crate.
pub type Error = quick_xml::Error;

/// The result type used by this crate.
pub type Result<T> = quick_xml::Result<T>;
