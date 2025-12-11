use {
    super::{super::parse, Date, Month},
    practicaltimestamp::UnixTimestamp,
    serde::Deserialize,
    snafu::{OptionExt, ResultExt, Snafu},
    std::{num::NonZeroU8, str::FromStr},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DateParseError {
    #[snafu(display("parse error: {}", source))]
    Parse {
        source: parse::ParseError<std::num::ParseIntError>,
    },
    #[snafu(display("invalid month"))]
    InvalidMonth,
}

impl FromStr for Date {
    type Err = DateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split(' ');

        Ok(Date {
            day: next_parsed(&mut fields, "day")?,
            month: next(&mut fields, "month")?.parse()?,
            year: next_parsed(&mut fields, "year")?,
        })
    }
}

fn next<'a>(
    mut iter: impl Iterator<Item = &'a str>,
    field: &'static str,
) -> Result<&'a str, DateParseError> {
    iter.next()
        .context(parse::MissingField { field })
        .context(Parse)
}

fn next_parsed<'a, T>(
    iter: impl Iterator<Item = &'a str>,
    field: &'static str,
) -> Result<T, DateParseError>
where
    T: std::str::FromStr<Err = std::num::ParseIntError>,
{
    next(iter, field)?
        .parse()
        .context(parse::Parse { field })
        .context(Parse)
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl FromStr for Month {
    type Err = DateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Month::iter()
            .find(|month| month.matches_str(s))
            .context(InvalidMonth)
    }
}

impl Month {
    fn matches_str(self, s: &str) -> bool {
        if let Some(rest) = s.strip_prefix(self.short()) {
            rest.is_empty() || rest == &self.long()[Month::SHORT_LEN..]
        } else {
            false
        }
    }

    fn from_n(n: u8) -> Month {
        Month::ALL[n as usize - 1]
    }
}

impl Date {
    fn from_unix_timestamp(timestamp: u64) -> Date {
        let (year, month, day) = UnixTimestamp::from_unix_timestamp(timestamp as _)
            .unwrap()
            .to_year_month_day();

        Date {
            year,
            month: Month::from_n(month),
            day: NonZeroU8::new(day).unwrap(),
        }
    }

    pub fn deserialize_unix_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <_>::deserialize(deserializer).map(Date::from_unix_timestamp)
    }
}
