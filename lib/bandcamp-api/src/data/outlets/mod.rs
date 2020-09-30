pub mod parse;

use {
    crate::{
        pages,
        data::{
            Query,
            releases::ReleaseKind
        }
    },
    
};

#[derive(Debug, Clone)]
pub struct Outlet {
    pub info: Info,
    pub featured: Vec<Featured>,
    pub releases: Vec<Release>
}

impl Query<pages::Outlet> for Outlet {
    type Err = parse::Error;

    fn query(mut page: pages::Outlet) -> Result<Self, Self::Err> {
        let mut buf = vec![];
        let info = parse::get_info(&mut *page, &mut buf)?;
        let (featured, releases) = parse::get_releases(&mut *page, &mut buf)?;

        Ok(Outlet {
            info,
            featured,
            releases
        })
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub kind: OutletKind,
    pub name: String,
    pub bio: String,
}

#[derive(Debug, Copy, PartialEq, Eq, Clone)]
pub enum OutletKind {
    Artist,
    Label
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub url: String,
    pub title: String,
    pub artist: Option<String>
}

#[derive(Debug, Clone)]
pub struct Featured(pub ReleaseInfo);

impl std::ops::Deref for Featured {
    type Target = ReleaseInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
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
