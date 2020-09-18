use {
    super::*,
    serde::Deserialize,
    snafu::{Snafu, OptionExt, ResultExt}
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ParseError<E>
where E: std::fmt::Debug + snafu::Error + 'static {
    #[snafu(display("missing field: {}", field))]
    MissingField { field: &'static str },
    #[snafu(display("error parsing field ´{}´", field))]
    Parse { source: E, field: &'static str }
}

pub type DateParseError = ParseError<std::num::ParseIntError>;

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
    iter.next().context(MissingField { field: field })
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
