use {
    super::*,
    snafu::{Snafu, ResultExt},
    scrape::{
        Scrape,
        BufMut,
        extract::attr
    },
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("album info not found"))]
    NoInfo,
    #[snafu(display("missing field: {}", field))]
    MissingInfo { field: &'static str },
    #[snafu(display("error parsing album info: {}", source))]
    Serde { source: serde_json::Error }
}

type Result<T> = std::result::Result<T, Error>;

pub(super) fn get_json(mut scraper: impl Scrape, buf: BufMut) -> Option<String> {
    scraper
        .extract(attr("data-tralbum"), buf)
        .ok()?
}

pub(super) fn parse_json(string: impl AsRef<str>) -> Result<Album> {
    serde_json::from_str(string.as_ref())
        .context(Serde)
}
