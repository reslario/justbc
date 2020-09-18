use {
    super::*,
    serde::Deserialize,
    snafu::{Snafu, OptionExt, ResultExt}
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("album info not found"))]
    NoInfo,
    #[snafu(display("missing field: {}", field))]
    MissingInfo { field: &'static str },
    #[snafu(display("error parsing field {}: {}", field, source))]
    Serde { source: serde_json::Error, field: &'static str }
}

pub (super) fn album_data_str(script: &str) -> Option<&str> {
    const VAR: &str = "var TralbumData = {";

    let start = script.find(VAR)?
        + VAR.len();

    let end = script[start..]
        .find("};")?;

    script[start..][..end].into()
}

pub (super) fn parse_album_data(string: &str) -> Result<Album, Error> {
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
