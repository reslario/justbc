use {
    serde::Deserialize,
    select::predicate::Name,
    snafu::{Snafu, OptionExt, ResultExt},
    crate::{
        pages,
        data::Query
    }
};


#[derive(Debug, Clone)]
pub struct Album {
    pub info: Info,
    pub tracks: Vec<Track>
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("album info not found"))]
    NoInfo,
    #[snafu(display("missing field: {}", field))]
    MissingInfo { field: &'static str },
    #[snafu(display("error parsing field {}: {}", field, source))]
    Serde { source: serde_json::Error, field: &'static str }
}

impl Query for Album {
    type Page = pages::Album;
    type Err = Error;

    fn query(page: &Self::Page) -> Result<Self, Self::Err> {
        page.find(Name("script"))
            .filter_map(|script| script.first_child())
            .filter_map(|script| script.as_text())
            .find_map(album_data_str)
            .context(NoInfo)
            .and_then(parse_album_data)
    }
}

fn album_data_str(script: &str) -> Option<&str> {
    const VAR: &str = "var TralbumData = {";

    let start = script.find(VAR)?
        + VAR.len();

    let end = script[start..]
        .find("};")?;

    script[start..][..end].into()
}

fn parse_album_data(string: &str) -> Result<Album, Error> {
    Ok(Album {
        info: get_field("current", string)?,
        tracks: get_field("trackinfo", string)?
    })
}

fn get_field<'de, T: Deserialize<'de>>(field: &'static str, from: &'de str) -> Result<T, Error> {
    from.split_terminator('\n')
        .map(str::trim)
        .find(|prop| prop.starts_with(field))
        .and_then(get_json)
        .map(serde_json::from_str)
        .context(MissingInfo { field })?
        .context(Serde { field })
}

fn get_json(prop: &str) -> Option<&str> {
    let colon = prop.find(':')?;
    let json = &prop[colon..][1..];

    json.rfind(',')
        .map(|comma| &json[..comma])
        .unwrap_or(json)
        .into()
}

#[derive(Deserialize, Debug, Clone)]
pub struct Info {
    pub title: String,
    pub about: String,
    pub credits: String,
    pub release_date: Date
}

#[derive(Debug, Clone)]
pub struct Date {
    pub day: u8,
    pub month: String,
    pub year: u16
}

#[derive(Debug, Snafu)]
pub enum DateParseError {
    #[snafu(display("missing field: {}", which))]
    MissingField { which: &'static str },
    #[snafu(display("error parsing field {}", field))]
    Parse { source: std::num::ParseIntError, field: &'static str }
}

impl std::str::FromStr for Date {
    type Err = DateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split(' ');

        Ok(Date {
            day: next_parsed(&mut fields, "day")?,
            month: next(&mut fields, "month")?.into(),
            year: next_parsed(&mut fields, "year")?
        })
    }
}

fn next<'a>(mut iter: impl Iterator<Item = &'a str>, field: &'static str) -> Result<&'a str, DateParseError> {
    iter.next().context(MissingField { which: field })
}

fn next_parsed<'a, T>(iter: impl Iterator<Item = &'a str>, field: &'static str) -> Result<T, DateParseError>
where T: std::str::FromStr<Err = std::num::ParseIntError> {
    next(iter, field)?.parse().context(Parse { field })
}

impl <'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        <&str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
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
