#[cfg(feature = "query")]
use {
    crate::{data::Query, url::ApiUrl},
    serde::Deserialize,
    url::Url,
};

use crate::data::{
    common::Id,
    releases::{self, ReleaseKind},
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Outlet {
    #[cfg_attr(feature = "query", serde(flatten))]
    pub info: Info,
    pub discography: Vec<Release>,
}

#[cfg(feature = "query")]
impl Query<Id<Outlet>> for Outlet {
    fn url(id: &Id<Outlet>) -> Url {
        ApiUrl::mobile()
            .function("band_details")
            .query("band_id", id.to_string())
            .into()
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Info {
    #[cfg_attr(
        feature = "query",
        serde(rename = "artists", deserialize_with = "guess_outlet_kind")
    )]
    pub kind: OutletKind,
    pub name: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub id: Id<Outlet>,
}

#[derive(Debug, Copy, PartialEq, Eq, Clone)]
pub enum OutletKind {
    Artist,
    Label,
}

#[cfg(feature = "query")]
#[allow(clippy::unnecessary_wraps)]
fn guess_outlet_kind<'de, D>(deserializer: D) -> Result<OutletKind, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // there's no proper way to find out whether an outlet
    // is a label or an artist, so we just assume that if
    // its array of artists is empty, it's probably an artist
    // itself
    if <[(); 0]>::deserialize(deserializer).is_ok() {
        Ok(OutletKind::Artist)
    } else {
        Ok(OutletKind::Label)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Release {
    #[cfg_attr(feature = "query", serde(rename = "item_type"))]
    pub kind: ReleaseKind,
    #[cfg_attr(feature = "query", serde(rename = "item_id"))]
    pub id: Id<releases::Release>,
    pub title: String,
    #[cfg_attr(feature = "query", serde(rename = "artist_name"))]
    pub artist: Option<String>,
}
