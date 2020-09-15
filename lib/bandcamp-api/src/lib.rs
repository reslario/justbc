pub mod data;
pub mod pages;

use {
    pages::Page,
    std::fmt::Display,
    snafu::{Snafu, ResultExt},
    select::document::Document

};

#[derive(Debug, Snafu)]
pub enum QueryError<DE>
where DE: std::fmt::Display + snafu::Error + 'static {
    #[snafu(display("connection error: {}", source))]
    Connection { source: reqwest::Error },
    #[snafu(display("error retrieving data: {}", source))]
    Data { source: DE }
}

pub type QueryResult<T, DE> = Result<T, QueryError<DE>>;

pub struct Api {
    client: reqwest::Client
}

impl Api {
    pub fn new() -> Api {
        Api {
            client: reqwest::Client::new()
        }
    }

    pub async fn query<T>(&self, args: &<T::Page as Page>::Args) -> QueryResult<T, T::Err>
    where 
        T: data::Query,
        T::Page: Page,
        T::Err: snafu::Error + Display + 'static,
        for <'url> &'url <T::Page as Page>::Url: reqwest::IntoUrl
    {
        let resp = self
            .client
            .get(&T::Page::url(args))
            .send()
            .await
            .context(Connection)?
            .text()
            .await
            .context(Connection)?;

        let page = Document::from(resp.as_str()).into();

        T::query(&page).context(Data)
    }
}
