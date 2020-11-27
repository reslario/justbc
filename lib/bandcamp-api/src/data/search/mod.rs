#[cfg(feature = "query")]
mod parse;

#[cfg(feature = "query")]
use {
    snafu::ResultExt,
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
    url::Url,
    crate::data::common::Date
};

#[derive(Debug, PartialEq, Eq)]
pub struct Search {
    pub results: Vec<SearchResult>
}

#[cfg(feature = "query")]
impl Search {
    const PER_PAGE: usize = 18;
}

#[cfg(feature = "query")]
impl Query<pages::Search> for Search {
    type Err = parse::Error;

    fn query(mut page: pages::Search) -> Result<Self, parse::Error> {
        let mut results = Vec::with_capacity(Search::PER_PAGE);
        let mut infos = page.filter(div().class("result-info"));
        let mut buf = vec![];

        loop {
            let mut info = infos.take(1);
            if let scrape::Event::Eof = info.read_event(&mut buf).context(parse::Read)? {
                return Ok(Search { results })
            } else {
                if let Some(result) = parse::parse_result(info, &mut buf)? {
                    results.push(result)
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SearchResult {
    Artist(Artist),
    Label(Label),
    Album(Album),
    Track(Track)
}

impl SearchResult {
    pub fn heading(&self) -> &Heading {
        match self {
            SearchResult::Artist(a) => &a.heading,
            SearchResult::Label(l) => &l.heading,
            SearchResult::Album(a) => &a.heading,
            SearchResult::Track(t) => &t.heading
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Artist {
    pub heading: Heading,
    pub sub_heading: Option<String>,
    pub genre: Option<String>,
    pub tags: Option<Tags>
}

#[derive(Debug, PartialEq, Eq)]
pub struct Label {
    pub heading: Heading,
    pub sub_heading: Option<String>,
    pub tags: Option<Tags>
}

#[derive(Debug, PartialEq, Eq)]
pub struct Heading {
    pub title: String,
    pub url: Url
}

#[derive(Debug, PartialEq, Eq)]
pub struct Album {
    pub heading: Heading,
    pub by: String,
    pub length: Length,
    pub released: Date,
    pub tags: Option<Tags>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Length {
    tracks: u16,
    minutes: u16
}

#[derive(Debug, PartialEq, Eq)]
pub struct Track {
    pub heading: Heading,
    pub source: Source,
    pub released: Date,
    pub tags: Option<Tags>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Source {
    pub from: Option<String>,
    pub by: String
}

#[derive(PartialEq, Eq)]
pub struct Tags {
    pub string: String,
    pub indices: Vec<usize>
}

impl Tags {
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        std::iter::once(&0)
            .chain(&self.indices)
            .zip(self.indices.iter())
            .map(move |(start, end)| &self.string[*start..*end])
    }
}

use std::fmt;

impl fmt::Debug for Tags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tags = &mut self.iter();

        for tag in tags.take(self.indices.len() - 1) {
            write!(f, "{}, ", tag)?
        }

        if let Some(tag) = tags.next() {
            tag.fmt(f)?
        }

        Ok(())
    }
}
