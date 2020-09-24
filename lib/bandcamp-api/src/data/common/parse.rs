use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ParseError<E>
where E: std::fmt::Debug + snafu::Error + 'static {
    #[snafu(display("missing field: {}", field))]
    MissingField { field: &'static str },
    #[snafu(display("error parsing field ´{}´: {}", field, source))]
    Parse { source: E, field: &'static str }
}
