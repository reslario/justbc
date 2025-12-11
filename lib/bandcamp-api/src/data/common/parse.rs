use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ParseError<E>
where
    E: std::fmt::Debug + snafu::Error + 'static,
{
    #[snafu(display("missing field: {}", field))]
    MissingField { field: &'static str },
    #[snafu(display("error parsing field ´{}´: {}", field, source))]
    Parse { source: E, field: &'static str },
}

pub fn deserialize_url<'de, D>(deserializer: D) -> Result<url::Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    <&str>::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}
