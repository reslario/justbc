#[cfg(feature = "query")]
use {
    url::Url,
    serde::Deserialize,
    crate::{
        url::ApiUrl,
        data::Query
    }
};

use crate::data::{
    fans,
    releases,
    common::Id,
    outlets::{self, OutletKind}
};

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Search {
    pub results: Vec<SearchResult>
}

#[cfg(feature = "query")]
impl Query<str> for Search {
    fn url(q: &str) -> Url {
        ApiUrl::new("fuzzysearch")
            .version("1")
            .function("app_autocomplete")
            .query("q", q)
            .into()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
#[cfg_attr(feature = "query", serde(tag = "type"))]
pub enum SearchResult {
    #[cfg_attr(feature = "query", serde(rename = "b"))]
    Outlet(Outlet),
    #[cfg_attr(feature = "query", serde(rename = "a"))]
    Album(Album),
    #[cfg_attr(feature = "query", serde(rename = "t"))]
    Track(Track),
    #[cfg_attr(feature = "query", serde(rename = "f"))]
    Fan(Fan)
}

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Outlet {
    #[cfg_attr(feature = "query", serde(rename = "is_label", deserialize_with = "outlet_kind"))]
    pub kind: OutletKind,
    pub name: String,
    pub id: Id<outlets::Outlet>
}

#[cfg(feature = "query")]
fn outlet_kind<'de, D>(deserializer: D) -> Result<OutletKind, D::Error>
where D: serde::Deserializer<'de> {
    if <_>::deserialize(deserializer)? {
        Ok(OutletKind::Label)
    } else {
        Ok(OutletKind::Artist)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Album {
    pub name: String,
    #[cfg_attr(feature = "query", serde(rename = "band_name"))]
    pub artist: String,
    pub id: Id<releases::Release>,
    #[cfg_attr(feature = "query", serde(rename = "band_id"))]
    pub artist_id: Id<outlets::Outlet>
}

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Track {
    pub name: String,
    pub id: Id<releases::Release>,
    #[cfg_attr(feature = "query", serde(rename = "album_name"))]
    pub album: Option<String>,
    pub album_id: Option<u64>,
    #[cfg_attr(feature = "query", serde(rename = "band_name"))]
    pub artist: String,
    #[cfg_attr(feature = "query", serde(rename = "band_id"))]
    pub artist_id: Id<outlets::Outlet>
}

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Fan {
    pub name: String,
    pub id: Id<fans::Fan>
}
