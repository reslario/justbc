mod parse;

use {
    std::fmt,
    serde::Deserialize,
    snafu::{OptionExt, ResultExt},
    scrape::{
        Scrape,
        filter::*
    },
    crate::{
        pages,
        data::{
            Query,
            common::Date
        }
    }
};

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    #[serde(rename = "current")]
    pub info: Info,
    #[serde(rename = "trackinfo")]
    pub tracks: Vec<Track>
}

impl Query<pages::Album> for Release {
    type Err = parse::Error;

    fn query(mut page: pages::Album) -> Result<Self, Self::Err> {
        page.filter(tag("script"))
            .find_extract(parse::get_json, &mut vec![])
            .context(parse::Read)?
            .context(parse::NoInfo)
            .and_then(parse::parse_json)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseKind {
    Track,
    Album
}

impl ReleaseKind {
    pub(crate) fn url_segment(&self) -> &'static str {
        match self {
            ReleaseKind::Track => "track",
            ReleaseKind::Album => "album"
        }
    }
}

impl fmt::Display for ReleaseKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Info {
    #[serde(rename = "type")]
    pub kind: ReleaseKind,
    pub title: String,
    pub about: Option<String>,
    pub credits: Option<String>,
    pub release_date: Date
}

#[derive(Deserialize, Debug, Clone)]
pub struct Track {
    title: String,
    file: File,
    duration: f32
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct File {
    mp3_128: String
}
