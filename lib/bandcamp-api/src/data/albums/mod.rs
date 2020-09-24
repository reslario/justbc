mod parse;

use {
    snafu::OptionExt,
    serde::Deserialize,
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
pub struct Album {
    #[serde(rename = "current")]
    pub info: Info,
    #[serde(rename = "trackinfo")]
    pub tracks: Vec<Track>
}

impl Query<pages::Album> for Album {
    type Err = parse::Error;

    fn query(mut page: pages::Album) -> Result<Self, Self::Err> {
        let mut scripts = page.filter(tag("script"));
        let mut buf = vec![];

        std::iter::repeat(())
            .find_map(|_| parse::get_json(&mut scripts, &mut buf))
            .context(parse::NoInfo)
            .and_then(parse::parse_json)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Info {
    pub title: String,
    pub about: String,
    pub credits: String,
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
