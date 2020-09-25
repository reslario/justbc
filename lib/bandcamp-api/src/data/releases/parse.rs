use {
    super::*,
    scrape::extract::attr,
    snafu::{Snafu, ResultExt}
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("release info not found"))]
    NoInfo,
    #[snafu(display("error extracting release information: {}", source))]
    Read { source: scrape::Error },
    #[snafu(display("missing field: {}", field))]
    MissingInfo { field: &'static str },
    #[snafu(display("error parsing release info: {}", source))]
    Serde { source: serde_json::Error }
}

pub(super) fn get_json(script: scrape::Event) -> Option<Result<String, scrape::Error>> {
    attr("data-tralbum")(script)
}

pub(super) fn parse_json(string: impl AsRef<str>) -> Result<Release, Error> {
    serde_json::from_str(string.as_ref())
        .context(Serde)
}
