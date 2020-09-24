pub mod data;
pub mod pages;

use {
    pages::Page,
    scrape::Scraper,
    std::fmt::Display,
    snafu::{Snafu, ResultExt},
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

    pub async fn query<T, P, A>(&self, args: &A) -> QueryResult<T, T::Err>
    where 
        T: data::Query<P>,
        P: Page<A>,
        T::Err: snafu::Error + Display + 'static,
        for <'url> &'url <P as Page<A>>::Url: reqwest::IntoUrl
    {
        let resp = self
            .client
            .get(&P::url(args))
            .send()
            .await
            .context(Connection)?
            .text()
            .await
            .context(Connection)
            .map(std::io::Cursor::new)?;

        let page = Scraper::new(resp).into();

        T::query(page).context(Data)
    }
}
