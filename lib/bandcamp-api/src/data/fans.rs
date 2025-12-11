use {
    crate::{
        data::{
            common::Id,
            outlets,
            releases::{self, ReleaseKind},
            Query,
        },
        url::ApiUrl,
    },
    serde::Deserialize,
};

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Fan {
    pub name: String,
    pub id: Id<Fan>,
    pub location: String,
    pub bio: String,
    pub collection: Vec<Collected>,
}

#[derive(Debug, Copy, Clone)]
pub struct FanArgs {
    pub id: Id<Fan>,
    pub start: u16,
    pub count: u16,
}

impl FanArgs {
    pub const DEFAULT_COUNT: u16 = 25;
}

impl Query<FanArgs> for Fan {
    fn url(args: &FanArgs) -> url::Url {
        ApiUrl::mobile()
            .function("fan_details")
            .query("fan_id", args.id.to_string())
            .query("start", args.start.to_string())
            .query("count", args.count.to_string())
            .into()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "query", derive(Deserialize))]
pub struct Collected {
    #[cfg_attr(feature = "query", serde(rename = "tralbum_type"))]
    pub kind: ReleaseKind,
    #[cfg_attr(feature = "query", serde(rename = "item_title"))]
    pub title: String,
    #[cfg_attr(feature = "query", serde(rename = "item_id"))]
    pub id: Id<releases::Release>,
    #[cfg_attr(feature = "query", serde(rename = "band_name"))]
    pub artist: String,
    #[cfg_attr(feature = "query", serde(rename = "band_id"))]
    pub artist_id: Id<outlets::Outlet>,
}
