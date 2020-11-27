#[cfg(feature = "query")]
mod parse;

#[cfg(feature = "query")]
use crate::{
    pages,
    data::Query
};

use {
    url::Url,
    crate::data::releases::ReleaseKind
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outlet {
    pub info: Info,
    pub featured: Vec<Featured>,
    pub releases: Vec<Release>
}

#[cfg(feature = "query")]
impl Query<pages::Outlet> for Outlet {
    type Err = parse::Error;

    fn query(mut page: pages::Outlet) -> Result<Self, Self::Err> {
        let mut buf = vec![];
        let info = parse::get_info(&mut *page, &mut buf)?;
        let (featured, releases) = parse::get_releases(&mut *page, &mut buf, &info)?;

        Ok(Outlet {
            info,
            featured,
            releases
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub kind: OutletKind,
    pub name: String,
    pub bio: Option<String>,
    pub url: Url,
}

#[derive(Debug, Copy, PartialEq, Eq, Clone)]
pub enum OutletKind {
    Artist,
    Label
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseInfo {
    pub url: Url,
    pub title: String,
    pub artist: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Featured(pub ReleaseInfo);

impl std::ops::Deref for Featured {
    type Target = ReleaseInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub kind: ReleaseKind,
    pub info: ReleaseInfo
}

impl std::ops::Deref for Release {
    type Target = ReleaseInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
