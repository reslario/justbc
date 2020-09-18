mod parse;

use {
    snafu::OptionExt,
    serde::Deserialize,
    select::predicate::Name,
    crate::{
        pages,
        data::{
            Query,
            common::Date
        }
    }
};

#[derive(Debug, Clone)]
pub struct Album {
    pub info: Info,
    pub tracks: Vec<Track>
}

impl Query for Album {
    type Page = pages::Album;
    type Err = parse::Error;

    fn query(page: &Self::Page) -> Result<Self, Self::Err> {
        page.find(Name("script"))
            .filter_map(|script| script.first_child())
            .filter_map(|script| script.as_text())
            .find_map(parse::album_data_str)
            .context(parse::NoInfo)
            .and_then(parse::parse_album_data)
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
