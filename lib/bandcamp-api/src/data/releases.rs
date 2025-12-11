#[cfg(feature = "query")]
use {
    crate::{
        data::{
            common::{self, Id},
            outlets::Outlet,
            Query,
        },
        url::ApiUrl,
    },
    serde::Deserialize,
};

use {
    crate::data::common::Date,
    std::{fmt, time::Duration},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Release {
    #[cfg_attr(feature = "query", serde(flatten))]
    pub info: Info,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Copy)]
#[cfg(feature = "query")]
pub struct ReleaseArgs {
    pub id: Id<Release>,
    pub kind: ReleaseKind,
    pub outlet: Id<Outlet>,
}

#[cfg(feature = "query")]
impl Query<ReleaseArgs> for Release {
    fn url(args: &ReleaseArgs) -> url::Url {
        ApiUrl::mobile()
            .function("tralbum_details")
            .query("tralbum_id", args.id.to_string())
            .query("tralbum_type", args.kind.identifier())
            .query("band_id", args.outlet.to_string())
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "query",
    derive(Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum ReleaseKind {
    #[cfg_attr(feature = "query", serde(alias = "t"))]
    Track,
    #[cfg_attr(feature = "query", serde(alias = "a"))]
    Album,
}

#[cfg(feature = "query")]
impl ReleaseKind {
    fn identifier(&self) -> &'static str {
        match self {
            ReleaseKind::Track => "t",
            ReleaseKind::Album => "a",
        }
    }
}

impl fmt::Display for ReleaseKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Info {
    #[cfg_attr(feature = "query", serde(rename = "type"))]
    pub kind: ReleaseKind,
    #[cfg_attr(feature = "query", serde(rename = "tralbum_artist"))]
    pub artist: String,
    pub title: String,
    pub about: Option<String>,
    pub credits: Option<String>,
    #[cfg_attr(
        feature = "query",
        serde(deserialize_with = "Date::deserialize_unix_timestamp")
    )]
    pub release_date: Date,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Track {
    pub title: String,
    #[cfg_attr(feature = "query", serde(rename = "streaming_url"))]
    pub stream: Stream,
    #[cfg_attr(feature = "query", serde(deserialize_with = "f32_duration"))]
    pub duration: Duration,
}

#[cfg(feature = "query")]
pub(super) fn f32_duration<'de, D>(deserializer: D) -> Result<std::time::Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    <_>::deserialize(deserializer).map(Duration::from_secs_f32)
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "query", derive(Deserialize))]
#[cfg_attr(feature = "query", serde(rename_all = "kebab-case"))]
pub struct Stream {
    #[cfg_attr(
        feature = "query",
        serde(deserialize_with = "common::parse::deserialize_url")
    )]
    pub mp3_128: url::Url,
}
