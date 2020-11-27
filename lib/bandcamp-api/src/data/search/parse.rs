use {
    super::*,
    snafu::{Snafu, OptionExt, ResultExt},
    scrape::{
        BufMut,
        Scrape,
        filter,
        extract::*
    },
    crate::data::common::{
        date,
        Date,
        parse::*
    },
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("error extracting data: {}", source))]
    Read { source: scrape::Error },
    #[snafu(display("missing information: {}", info))]
    MissingInfo { info: &'static str },
    #[snafu(display("error parsing tags"))]
    ParseTags,
    #[snafu(display("error parsing date: {}", source))]
    ParseDate { source: date::parse::DateParseError },
    #[snafu(display("error parsing length: {}", source))]
    ParseLength { source: LengthParseError },
    #[snafu(display("error parsing url: {}", source))]
    ParseUrl { source: url::ParseError }
}

pub(super) type Result<T> = std::result::Result<T, Error>;

pub(super) fn parse_result(mut scraper: impl Scrape, buf: BufMut) -> Result<Option<SearchResult>> {
    use SearchResult as R;

    let result = match div(&mut scraper, "itemtype", buf)?.as_str() {
        "ALBUM" => parse(&mut scraper, parse_album, R::Album, buf).into(),
        "TRACK" => parse(&mut scraper, parse_track, R::Track, buf).into(),
        "LABEL" => parse(&mut scraper, parse_label, R::Label, buf).into(),
        "ARTIST" => parse(&mut scraper, parse_artist, R::Artist, buf).into(),
        _ => None
    }.transpose()?;

    scraper.read_to_end(buf).context(Read)?;

    Ok(result)
}

fn div(scraper: impl Scrape, class: &'static str, buf: BufMut) -> Result<String> {
    scraper
        .into_filter(filter::div().class(class))
        .step(buf)
        .context(Read)?
        .extract(text, buf)
        .context(Read)?
        .context(missing(class))
}

fn parse<T, S>(
    scraper: S,
    parse: impl Fn(S, BufMut) -> Result<T>,
    map: impl Fn(T) -> SearchResult,
    buf: BufMut
) -> Result<SearchResult> {
    parse(scraper, buf).map(map)
}

fn parse_album(mut scraper: impl Scrape, buf: BufMut) -> Result<Album> {
    Ok(Album {
        heading: heading(&mut scraper, buf)?,
        by: by(&mut scraper, buf)?,
        length:  length(&mut scraper, buf)?,
        released:  release_date(&mut scraper, buf)?,
        tags: tags(scraper, buf).transpose()?
    })
}

fn parse_track(mut scraper: impl Scrape, buf: BufMut) -> Result<Track> {
    Ok(Track {
        heading: heading(&mut scraper, buf)?,
        source: source(&mut scraper, buf)?,
        released: release_date(&mut scraper, buf)?,
        tags: tags(scraper, buf).transpose()?
    })
}

fn parse_label(mut scraper: impl Scrape, buf: BufMut) -> Result<Label> {
    Ok(Label {
        heading: heading(&mut scraper, buf)?,
        sub_heading: sub_heading(&mut scraper, buf),
        tags:  tags(scraper, buf).transpose()?,
    })
}

fn parse_artist(mut scraper: impl Scrape, buf: BufMut) -> Result<Artist> {
    Ok(Artist {
        heading: heading(&mut scraper, buf)?,
        sub_heading: sub_heading(&mut scraper, buf),
        genre: genre(&mut scraper, buf),
        tags:  tags(scraper, buf).transpose()?,
    })
}

fn missing(info: &'static str) -> MissingInfo<&'static str> {
    MissingInfo { info }
}

fn heading(scraper: impl Scrape, buf: BufMut) -> Result<Heading> {
    let mut a = scraper
        .into_filter(filter::div().class("heading"))
        .into_filter(tag("a"));

    Ok(Heading {
        url: a.extract(attr("href"), buf)
            .context(Read)?
            .context(missing("heading link"))?
            .parse()
            .context(ParseUrl)?,
        title: a.extract(text, buf)
            .context(Read)?
            .context(missing("heading title"))?
    })
}

fn sub_heading(scraper: impl Scrape, buf: BufMut) -> Option<String> {
    div(scraper, "subhead", buf).ok()
}

fn tags(scraper: impl Scrape, buf: BufMut) -> Option<Result<Tags>> {
    div(scraper, "tags", buf)
        .ok()?
        .parse()
        .into()
}

impl std::str::FromStr for Tags {
    type Err = Error;

    fn from_str(tags: &str) -> Result<Self> {
        let (string, indices) = tags
            .trim()
            .strip_prefix("tags:")
            .context(ParseTags)?
            .split(',')
            .map(str::trim)
            .scan(0, |idx, tag| {
                *idx += tag.len();
                (tag, *idx).into()
            })
            .unzip();

        Ok(Tags {
            string,
            indices
        })
    }
}

fn release_date(scraper: impl Scrape, buf: BufMut) -> Result<Date> {
    div(scraper, "released", buf)
        .and_then(parse_release_date)
}

fn parse_release_date(date: impl AsRef<str>) -> Result<Date> {
    const START: &str = "released";

    date.as_ref()
        .trim()
        .strip_prefix(START)
        .context(MissingField { field: START })
        .context(date::parse::Parse)
        .context(ParseDate)?
        .trim_start()
        .parse()
        .context(ParseDate)
}

fn by(scraper: impl Scrape, buf: BufMut) -> Result<String> {
    div(scraper, "subhead", buf)
        .and_then(parse_by)
}

fn parse_by(by: impl AsRef<str>) -> Result<String> {
    const START: &str = "by ";

    by.as_ref()
        .trim()
        .strip_prefix(START)
        .context(missing(START))
        .map(<_>::into)
}

fn source(scraper: impl Scrape, buf: BufMut) -> Result<Source> {
    div(scraper, "subhead", buf)
        .and_then(|s| s.parse())
}

impl std::str::FromStr for Source {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self> {
        let (from, rest) = parse_from(source);
        let by = parse_by(rest)?;

        Ok(Source {
            from,
            by
        })
    }
}

fn parse_from(from: &str) -> (Option<String>, &str) {
    let from = from.trim();
    from.strip_prefix("from ")
        .and_then(|from| from
            .find('\n')
            .map(|nl| from.split_at(nl))
        ).map(|(from, rest)| (from.to_string().into(), rest))
        .unwrap_or((None, from))
}

fn genre(scraper: impl Scrape, buf: BufMut) -> Option<String> {
    div(scraper, "genre", buf)
        .ok()
        .and_then(|genre| genre
            .strip_prefix("genre: ")
            .map(<_>::into)
        )
}

fn length(scraper: impl Scrape, buf: BufMut) -> Result<Length> {
    div(scraper, "length", buf)?
        .parse()
        .context(ParseLength)
}

pub type LengthParseError = ParseError<std::num::ParseIntError>;

type LengthResult<T> = std::result::Result<T, LengthParseError>;

impl std::str::FromStr for Length {
    type Err = LengthParseError;

    fn from_str(length: &str) -> LengthResult<Self> {
        let mut fields = length.split(", ");
        
        Ok(Length {
            tracks: parse_field(&mut fields, "tracks")?,
            minutes: parse_field(&mut fields, "minutes")?
        })
    }
}

fn parse_field<'f>(mut iter: impl Iterator<Item = &'f str>, field: &'static str) -> LengthResult<u16> {
    iter.next()
        .context(MissingField { field })?
        .split_whitespace()
        .next()
        .context(MissingField { field })?
        .parse()
        .context(Parse { field })
}
