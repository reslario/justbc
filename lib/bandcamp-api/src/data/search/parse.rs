use {
    super::*,
    snafu::{Snafu, OptionExt, ResultExt},
    select::{
        node::Node,
        predicate::Class,
    },
    crate::data::common::{
        Date,
        parse::*
    },
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("missing information: {}", info))]
    MissingInfo { info: &'static str },
    #[snafu(display("error parsing tags"))]
    ParseTags,
    #[snafu(display("error parsing date: {}", source))]
    ParseDate { source: DateParseError },
    #[snafu(display("error parsing length: {}", source))]
    ParseLength { source: LengthParseError }
}

pub(super) type Result<T> = std::result::Result<T, Error>;

pub(super) fn parse_result(node: Node) -> Option<Result<SearchResult>> {
    use SearchResult as R;

    div(node, "itemtype").map(|ty| match ty.trim() {
        "ALBUM" => parse(node, parse_album, R::Album).into(),
        "TRACK" => parse(node, parse_track, R::Track).into(),
        "LABEL" => parse(node, parse_label, R::Label).into(),
        "ARTIST" => parse(node, parse_artist, R::Artist).into(),
        _ => None
    })
    .transpose()
    .map(|res| res?)
}

fn parse<T>(
    node: Node,
    parse: impl Fn(Node) -> Result<T>,
    map: impl Fn(T) -> SearchResult
) -> Result<SearchResult> {
    parse(node).map(map)
}

fn parse_album(node: Node) -> Result<Album> {
    Ok(Album {
        heading: heading(node)?,
        by: by(node)?,
        length:  length(node)?,
        released:  release_date(node)?,
        tags: parse_tags(node).transpose()?
    })
}

fn parse_track(node: Node) -> Result<Track> {
    Ok(Track {
        heading: heading(node)?,
        source: source(node)?,
        released: release_date(node)?,
        tags: parse_tags(node).transpose()?
    })
}

fn parse_label(node: Node) -> Result<Label> {
    Ok(Label {
        heading: heading(node)?,
        sub_heading: sub_heading(node)?
    })
}

fn parse_artist(node: Node) -> Result<Artist> {
    Ok(Artist {
        heading: heading(node)?,
        sub_heading: sub_heading(node)?,
        tags:  parse_tags(node).transpose()?,
        genre: genre(node),
    })
}

fn missing(info: &'static str) -> MissingInfo<&'static str> {
    MissingInfo { info }
}

fn heading(node: Node) -> Result<Heading> {
    let node = div_node(node, "heading")?;
    let a = node
        .children()
        .nth(1)
        .context(missing("heading title"))?;

    Ok(Heading {
        title: a.text().trim().into(),
        url: a.attr("href")
            .context(missing("heading link"))?
            .into(),
    })
}

fn div<'n>(node: Node<'n>, class: &'static str) -> Result<&'n str> {
    div_text(div_node(node, class)?)
        .context(missing(class))
}

fn div_node<'n>(node: Node<'n>, class: &'static str) -> Result<Node<'n>> {
    node.find(Class(class))
        .next()
        .context(missing(class))
}

fn div_text(node: Node) -> Option<&str> {
    node.first_child()?
        .as_text()
        .map(str::trim)
}

fn sub_heading(node: Node) -> Result<Option<String>> {
    div(node, "subhead")
        .map(str::to_string)
        .map(|s| Some(s).filter(|s| !s.is_empty()))
}

fn parse_tags(node: Node) -> Option<Result<Tags>> {
    div(node, "tags")
        .ok()
        .map(str::parse)
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

fn release_date(node: Node) -> Result<Date> {
    div(node, "released")
        .and_then(parse_release_date)
}

fn parse_release_date(date: &str) -> Result<Date> {
    const START: &str = "released";

    date.trim()
        .strip_prefix(START)
        .context(MissingField { field: START })
        .context(ParseDate)?
        .trim_start()
        .parse()
        .context(ParseDate)
}

fn by(node: Node) -> Result<String> {
    div(node, "subhead")
        .and_then(parse_by)
}

fn parse_by(by: &str) -> Result<String> {
    const START: &str = "by ";

    by.trim()
        .strip_prefix(START)
        .context(missing(START))
        .map(<_>::into)
}

fn source(node: Node) -> Result<Source> {
    div(node, "subhead")
        .and_then(str::parse)
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

fn genre(node: Node) -> Option<String> {
    div(node, "genre")
        .ok()
        .and_then(|genre| genre.strip_prefix("genre: "))
        .map(<_>::into)
}

fn length(node: Node) -> Result<Length> {
    div(node, "length")?
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
