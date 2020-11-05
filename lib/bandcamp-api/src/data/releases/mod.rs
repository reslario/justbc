#[cfg(feature = "query")]
mod parse;

#[cfg(feature = "query")]
use {
    serde::Deserialize,
    snafu::{OptionExt, ResultExt},
    scrape::{
        Scrape,
        filter::*
    },
    crate::{
        pages,
        data::Query
    }
};

use {
    crate::data::common::Date,
    std::{
        fmt,
        time::Duration
    }
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Release {
    pub artist: String,
    #[cfg_attr(feature = "query", serde(rename = "current"))]
    pub info: Info,
    #[cfg_attr(feature = "query", serde(rename = "trackinfo"))]
    pub tracks: Vec<Track>
}

#[cfg(feature = "query")]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
#[cfg_attr(feature = "query", serde(rename_all = "kebab-case"))]
pub enum ReleaseKind {
    Track,
    Album
}

#[cfg(feature = "query")]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Info {
    #[cfg_attr(feature = "query", serde(rename = "type"))]
    pub kind: ReleaseKind,
    pub title: String,
    pub about: Option<String>,
    pub credits: Option<String>,
    pub release_date: Date
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Track {
    pub title: String,
    pub file: File,
    #[cfg_attr(feature = "query", serde(deserialize_with = "parse::f32_duration"))]
    pub duration: Duration
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
#[cfg_attr(feature = "query", serde(rename_all = "kebab-case"))]
pub struct File {
    pub mp3_128: String
}
